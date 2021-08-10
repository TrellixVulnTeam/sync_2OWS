#![allow(dead_code, unused_macros)]

use indyrs::IndyError;
use serde_json::Value;

use crate::utils::{cheqd_keys, cheqd_ledger, cheqd_ledger::auth, cheqd_pool, environment};

use super::{logger, wallet, WalletHandle};
use super::test;

fn setup() -> String {
    let name = crate::utils::rand_utils::get_rand_string(10);
    test::cleanup_storage(&name);
    logger::set_default_logger();
    name
}

fn tear_down(name: &str, wallet_handle: WalletHandle) {
    wallet::close_wallet(wallet_handle).unwrap();
    test::cleanup_storage(name);
}

fn wallet_config(name: &str) -> String {
    json!({ "id": name }).to_string()
}

pub struct CheqdSetup {
    pub name: String,
    pub pool_alias: String,
    pub key_alias: String,
    pub account_id: String,
    pub pub_key: String,
    pub wallet_handle: WalletHandle,
}

impl CheqdSetup {
    pub fn new() -> CheqdSetup {
        let name = setup();

        // Wallet
        let wallet_config = wallet_config(&name);
        let (wallet_handle, _) = wallet::create_and_open_default_wallet(&wallet_config).unwrap();

        // Account
        let key_alias = "alice";
        let mnemonic = "alice";
        let (account_id, pub_key) = CheqdSetup::create_key(wallet_handle, key_alias, mnemonic).unwrap();

        // Pool
        let cheqd_test_pool_ip = environment::cheqd_test_pool_ip();
        let cheqd_test_chain_id = environment::cheqd_test_chain_id();
        cheqd_pool::add(&name, &cheqd_test_pool_ip, &cheqd_test_chain_id).unwrap();

        let setup = CheqdSetup {
            name: name.clone(),
            pool_alias: name,
            key_alias: key_alias.to_string(),
            account_id,
            pub_key,
            wallet_handle,
        };

        setup
    }

    pub fn create_key(wallet_handle: WalletHandle, alias: &str, mnemonic: &str) -> Result<(String, String), IndyError> {
        let key = cheqd_keys::add_from_mnemonic(wallet_handle, alias, mnemonic).unwrap();
        let key: Value = serde_json::from_str(&key).unwrap();
        println!("Cheqd setup. Create key: {:?}", key);

        let account_id = key["account_id"].as_str().unwrap().to_string();
        let pub_key = key["pub_key"].as_str().unwrap().to_string();
        Ok((account_id, pub_key))
    }

    pub fn get_base_account_number_and_sequence(&self, account_id: &str) -> Result<(u64, u64), IndyError> {
        let req = auth::build_query_account(account_id).unwrap();
        let resp = cheqd_pool::abci_query(&self.pool_alias, &req).unwrap();
        let resp = auth::parse_query_account_resp(&resp).unwrap();
        println!("Cheqd setup. Get account: {:?}", resp);

        let resp: Value = serde_json::from_str(&resp).unwrap();
        let base_account = resp["base_account"].as_object().unwrap();
        let account_number = base_account["account_number"].as_u64().unwrap();
        let account_sequence = base_account["sequence"].as_u64().unwrap();

        Ok((account_number, account_sequence))
    }

    pub fn build_and_sign_and_broadcast_tx(&self, msg: &[u8]) -> Result<String, IndyError> {
        // Get account info
        let (account_number, account_sequence) = self.get_base_account_number_and_sequence(&self.account_id)?;

        // Tx
        let tx = cheqd_ledger::auth::build_tx(
            &self.pool_alias, &self.pub_key, &msg, account_number, account_sequence, 300000, 0u64, "cheq", self.get_timeout_height(), "memo",
        )?;

        // Sign
        let signed = cheqd_keys::sign(self.wallet_handle, &self.key_alias, &tx)?;

        // Broadcast
        let resp = cheqd_pool::broadcast_tx_commit(&self.pool_alias, &signed)?;

        Ok(resp)
    }

    pub fn get_timeout_height(&self) -> u64 {
        const TIMEOUT: u64 = 20;
        let info: String = cheqd_pool::abci_info(&self.pool_alias).unwrap();
        let info: Value = serde_json::from_str(&info).unwrap();
        let current_height = info["response"]["last_block_height"].as_str().unwrap().parse::<u64>().unwrap();
        println!("Cheqd setup. Last block height: {:?}", current_height);

        return current_height + TIMEOUT;
    }
}

impl Drop for CheqdSetup {
    fn drop(&mut self) {
        tear_down(&self.name, self.wallet_handle);
    }
}
