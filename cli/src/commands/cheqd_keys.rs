extern crate regex;
extern crate chrono;

use crate::command_executor::{Command, CommandContext, CommandMetadata, CommandParams, CommandGroup, CommandGroupMetadata};
use crate::commands::*;

use crate::libindy::cheqd_keys::CheqdKeys;

pub mod group {
    use super::*;

    command_group!(CommandGroupMetadata::new("cheqd-keys", "Cheqd keys management commands"));
}

pub mod add_command {
    use super::*;

    command!(CommandMetadata::build("add", "Add key to wallet.")
                .add_required_param("alias", "Alias of key.")
                .add_optional_param("mnemonic", "Mnemonic phrase for creation key.")
                .add_example("cheqd-keys add-random alias=my_key")
                .add_example("cheqd-keys add-random alias=my_key mnemonic=my_mnemonic")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;
        let alias = get_str_param("alias", params).map_err(error_err!())?;
        let mnemonic = get_opt_str_param("mnemonic", params).map_err(error_err!())?;

        let res = if let Some(mnemonic)  = mnemonic {
            match CheqdKeys::add_from_mnemonic(wallet_handle, alias, mnemonic) {
                Ok(resp) => {
                    println_succ!("Key has been added from mnemonic \"{}\" .", resp);
                    Ok(())
                },
                Err(err) => {
                    handle_indy_error(err, None, None, None);
                    Err(())
                },
            }
        } else {
            match CheqdKeys::add_random(wallet_handle, alias) {
                Ok(resp) => {
                    println_succ!("Random key has been added \"{}\".", resp);
                    Ok(())
                },
                Err(err) => {
                    handle_indy_error(err, None, None, None);
                    Err(())
                },
            }
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod get_info_command {
    use super::*;

    command!(CommandMetadata::build("get-info", "Get info about key.")
                .add_required_param("alias", "Alias of key.")
                .add_example("cheqd-keys get-info alias=my_key")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;
        let alias = get_str_param("alias", params).map_err(error_err!())?;

        let res = match CheqdKeys::get_info(wallet_handle, alias) {
            Ok(resp) => {
                println_succ!("Get follow info \"{}\" ", resp);
                Ok(())
            },
            Err(err) => {
                handle_indy_error(err, None, None, None);
                Err(())
            },
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod get_list_keys_command {
    use super::*;
    use crate::utils::table::print_list_table;

    command!(CommandMetadata::build("get-list-keys", "Get list keys of current wallet.")
                .add_example("cheqd-keys get-list-keys")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

        let res = match CheqdKeys::get_list_keys(wallet_handle) {
            Ok(resp) => {
                let mut resp: Vec<serde_json::Value> = serde_json::from_str(&resp)
                    .map_err(|_| println_err!("Wrong data has been received"))?;

                print_list_table(&resp,
                                 &[("account_id", "Account id"),
                                     ("alias", "Alias"),
                                     ("pub_key", "Public key")],
                                 "There are no configs");
                Ok(())
            },
            Err(err) => {
                handle_indy_error(err, None, None, None);
                Err(())
            },
        };

        trace!("execute << {:?}", res);
        res
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const KEY_ALIAS: &str = "key_alias";
    const MNEMONIC: &str = "mnemonic";
    pub(crate) const KEY_ALIAS_WITH_BALANCE: &str = "alice";
    pub(crate) const MNEMONIC_WITH_BALANCE: &str = "alice";

    mod cheqd_keys {
        use super::*;

        #[test]
        pub fn add_random() {
            let ctx = setup_with_wallet();
            {
                let cmd = add_command::new();
                let mut params = CommandParams::new();
                params.insert("alias", KEY_ALIAS.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn add_from_mnemonic() {
            let ctx = setup_with_wallet();
            {
                let cmd = add_command::new();
                let mut params = CommandParams::new();
                params.insert("alias", KEY_ALIAS.to_string());
                params.insert("mnemonic", MNEMONIC.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn get_info() {
            let ctx = setup_with_wallet_and_cheqd_pool();
            {
                let cmd = get_info_command::new();
                let mut params = CommandParams::new();
                params.insert("alias", KEY_ALIAS_WITH_BALANCE.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn get_list_keys() {
            let ctx = setup_with_wallet_and_cheqd_pool();
            {
                let cmd = get_list_keys_command::new();
                let mut params = CommandParams::new();
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }
    }

    pub fn add(ctx: &CommandContext) {
        {
            let cmd = add_command::new();
            let mut params = CommandParams::new();
            params.insert("alias", KEY_ALIAS_WITH_BALANCE.to_string());
            params.insert("mnemonic", MNEMONIC_WITH_BALANCE.to_string());
            cmd.execute(&ctx, &params).unwrap();
        }
    }

    pub fn get_key(ctx: &CommandContext) -> serde_json::Value {
        let wallet_handle = ensure_opened_wallet_handle(ctx).unwrap();
        let key = CheqdKeys::get_info(wallet_handle, KEY_ALIAS_WITH_BALANCE).unwrap();
        serde_json::from_str(&key).unwrap()
    }
}