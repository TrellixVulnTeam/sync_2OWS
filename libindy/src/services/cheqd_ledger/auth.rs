use std::convert::TryInto;
use std::str::FromStr;

use cosmrs::{Coin, tx, AccountId};
use cosmrs::crypto::PublicKey;
use cosmrs::rpc::endpoint::abci_query;
use cosmrs::tendermint::block::Height;
use cosmrs::tx::{AuthInfo, Fee, Msg, SignDoc, SignerInfo};
use indy_api_types::errors::{IndyErrorKind, IndyResult, IndyResultExt};
use crate::domain::cheqd_ledger::auth::{QueryAccountRequest, QueryAccountResponse, Account};
use crate::domain::cheqd_ledger::CheqdProto;
use crate::services::CheqdLedgerService;
use prost::bytes::Buf;
use crate::utils::cheqd_crypto::check_proofs;

impl CheqdLedgerService {
    pub(crate) async fn auth_build_tx(
        &self,
        chain_id: &str,
        sender_public_key: &str,
        msg: Msg,
        account_number: u64,
        sequence_number: u64,
        max_gas: u64,
        max_coin_amount: u64,
        max_coin_denom: &str,
        timeout_height: u64,
        memo: &str,
    ) -> IndyResult<SignDoc> {
        let timeout_height: Height = timeout_height.try_into()?;

        let tx_body = tx::Body::new(vec![msg], memo, timeout_height);

        let signer_info = Self::build_signer_info(sender_public_key, sequence_number)?;

        let auth_info =
            Self::build_auth_info(max_gas, max_coin_amount, max_coin_denom, signer_info)?;

        let chain_id = chain_id.try_into()?;

        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)?;

        Ok(sign_doc)
    }

    fn build_auth_info(
        max_gas: u64,
        max_coin: u64,
        max_coin_denom: &str,
        signer_info: SignerInfo,
    ) -> IndyResult<AuthInfo> {
        let amount = Coin {
            denom: max_coin_denom.parse()?,
            amount: max_coin.into(),
        };

        let auth_info = signer_info.auth_info(Fee::from_amount_and_gas(amount, max_gas));

        Ok(auth_info)
    }

    fn build_signer_info(public_key: &str, sequence_number: u64) -> IndyResult<SignerInfo> {
        let public_key = rust_base58::FromBase58::from_base58(public_key)?;
        let public_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&public_key)?;
        let public_key: PublicKey = public_key.into();

        let signer_info = SignerInfo::single_direct(Some(public_key), sequence_number);
        Ok(signer_info)
    }

    pub(crate) fn auth_build_query_account_without_proof(
        &self,
        address: &str,
    ) -> IndyResult<abci_query::Request> {
        let query_data = QueryAccountRequest::new(address.to_string());
        let path = format!("/cosmos.auth.v1beta1.Query/Account");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;
        let req =
            abci_query::Request::new(Some(path), query_data.to_proto_bytes()?, None, true);
        Ok(req)
    }

    pub(crate) fn auth_build_query_account(
        &self,
        address: &str,
    ) -> IndyResult<abci_query::Request> {
        // let mut encoded_path = 0x01.to_bytes()?;
        // encoded_path.push_str(address);
        let mut query_data = vec!(0x01_u8);
        let acc = AccountId::from_str(address)?;
        query_data.append(acc.to_bytes().to_vec().as_mut());
        let path = format!("/store/acc/key");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;
        let req = abci_query::Request::new(Some(path), query_data, None, true);
        Ok(req)
    }

    pub(crate) fn auth_parse_query_account_resp(
        &self,
        resp: &abci_query::Response,
    ) -> IndyResult<QueryAccountResponse> {
        let result = if !resp.response.value.is_empty() {
            Some(Account::from_proto_bytes(&resp.response.value)?)
        } else { None };
        check_proofs(resp.clone())?;
        Ok(QueryAccountResponse::new(result))
    }

    pub(crate) fn auth_parse_query_account_resp_without_proof(
        &self,
        resp: &abci_query::Response,
    ) -> IndyResult<QueryAccountResponse> {
        let result = QueryAccountResponse::from_proto_bytes(&resp.response.value)?;
        return Ok(result);
    }
}
