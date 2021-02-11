
use indy_api_types::{ErrorCode, CommandHandle, PoolHandle, INVALID_POOL_HANDLE};
use crate::commands::Locator;
use crate::domain::pool::{PoolConfig, PoolOpenConfig};
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;
use indy_api_types::validation::Validatable;

use serde_json;
use libc::c_char;
use crate::services::metrics::command_metrics::CommandMetric;

/// Creates a new local pool ledger configuration that can be used later to connect pool nodes.
///
/// #Params
/// config_name: Name of the pool ledger configuration.
/// config (optional): Pool configuration json. if NULL, then default config will be used. Example:
/// {
///     "genesis_txn": string (optional), A path to genesis transaction file. If NULL, then a default one will be used.
///                    If file doesn't exists default one will be created.
/// }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_create_pool_ledger_config(
    command_handle: CommandHandle,
    config_name: *const c_char,
    config: *const c_char,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_create_pool_ledger_config > config_name {:?}, config {:?}", config_name, config);

    check_useful_c_str!(config_name, ErrorCode::CommonInvalidParam2);
    check_useful_opt_json!(config, ErrorCode::CommonInvalidParam3, PoolConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_create_pool_ledger_config ? config_name {:?}, config: {:?}", config_name, config);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .create(config_name, config);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_create_pool_ledger_config ? err {:?}", err);

        cb(command_handle, err)
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandCreate, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_create_pool_ledger_config < {:?}", res);

    res
}

/// Opens pool ledger and performs connecting to pool nodes.
///
/// Pool ledger configuration with corresponded name must be previously created
/// with indy_create_pool_ledger_config method.
/// It is impossible to open pool with the same name more than once.
///
/// config_name: Name of the pool ledger configuration.
/// config (optional): Runtime pool configuration json.
///                         if NULL, then default config will be used. Example:
/// {
///     "timeout": int (optional), timeout for network request (in sec).
///     "extended_timeout": int (optional), extended timeout for network request (in sec).
///     "preordered_nodes": array<string> -  (optional), names of nodes which will have a priority during request sending:
///         ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
///         This can be useful if a user prefers querying specific nodes.
///         Assume that `Node1` and `Node2` nodes reply faster.
///         If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
///         Note: Nodes not specified will be placed randomly.
///     "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
///         By default Libindy sends a read requests to 2 nodes in the pool.
///         If response isn't received or `state proof` is invalid Libindy sends the request again but to 2 (`number_read_nodes`) * 2 = 4 nodes and so far until completion.
/// }
///
/// #Returns
/// Handle to opened pool to use in methods that require pool connection.
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_open_pool_ledger(
    command_handle: CommandHandle,
    config_name: *const c_char,
    config: *const c_char,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode,
                         pool_handle: PoolHandle)>) -> ErrorCode {
    trace!("indy_open_pool_ledger > config_name {:?} config {:?}", config_name, config);

    check_useful_c_str!(config_name, ErrorCode::CommonInvalidParam2);
    check_useful_opt_validatable_json!(config, ErrorCode::CommonInvalidParam3, PoolOpenConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_open_pool_ledger ? config_name {:?} config {:?}", config_name, config);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .open(config_name, config)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, pool_handle) = prepare_result_1!(res, INVALID_POOL_HANDLE);
        trace!("indy_open_pool_ledger ? err {:?} pool_handle {:?}", err, pool_handle);

        cb(command_handle, err, pool_handle)
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandOpen, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_open_pool_ledger < {:?}", res);

    res
}

/// Refreshes a local copy of a pool ledger and updates pool nodes connections.
///
/// #Params
/// handle: pool handle returned by indy_open_pool_ledger
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_refresh_pool_ledger(
    command_handle: CommandHandle,
    handle: PoolHandle,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_refresh_pool_ledger > handle {:?}", handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_refresh_pool_ledger ? handle {:?}", handle);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .refresh(handle)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_refresh_pool_ledger ? err {:?}", err);

        cb(command_handle, err)
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandRefresh, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_refresh_pool_ledger < {:?}", res);

    res
}

/// Lists names of created pool ledgers
///
/// #Params
///
/// #Returns
/// Error code
///
/// #Errors
#[no_mangle]
pub extern fn indy_list_pools(
    command_handle: CommandHandle,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode,
                         pools: *const c_char)>) -> ErrorCode {
    trace!("indy_list_pools >");

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam2);

    trace!("indy_list_pools ?");

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .list();
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result_1!(res, String::new());
        trace!("indy_list_pools ? err {:?} res {:?}", err, res);

        let list = ctypes::string_to_cstring(res);
        cb(command_handle, err, list.as_ptr());
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandList, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_list_pools < {:?}", res);

    res
}

/// Closes opened pool ledger, opened nodes connections and frees allocated resources.
///
/// #Params
/// handle: pool handle returned by indy_open_pool_ledger.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_close_pool_ledger(
    command_handle: CommandHandle,
    handle: PoolHandle,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_close_pool_ledger > handle {:?}", handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_close_pool_ledger ? handle {:?}", handle);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .close(handle)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_close_pool_ledger ? err {:?}", err);

        cb(command_handle, err);
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandClose, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_close_pool_ledger < {:?}", res);

    res
}

/// Deletes created pool ledger configuration.
///
/// #Params
/// config_name: Name of the pool ledger configuration to delete.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_delete_pool_ledger_config(
    command_handle: CommandHandle,
    config_name: *const c_char,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_delete_pool_ledger_config > config_name {:?}", config_name);

    check_useful_c_str!(config_name, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_delete_pool_ledger_config ? config_name {:?}", config_name);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .delete(config_name)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_delete_pool_ledger_config ? err {:?}", err);

        cb(command_handle, err);
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandDelete, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_delete_pool_ledger_config < {:?}", res);

    res
}

/// Set PROTOCOL_VERSION to specific version.
///
/// There is a global property PROTOCOL_VERSION that used in every request to the pool and
/// specified version of Indy Node which Libindy works.
///
/// By default PROTOCOL_VERSION=1.
///
/// #Params
/// protocol_version: Protocol version will be used:
///     1 - for Indy Node 1.3
///     2 - for Indy Node 1.4 and greater
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_set_protocol_version(
    command_handle: CommandHandle,
    protocol_version: usize,
    cb: Option<extern fn(command_handle_: CommandHandle,
                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_set_protocol_version > protocol_version {:?}", protocol_version);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_set_protocol_version ? protocol_version {:?}", protocol_version);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pool_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .set_protocol_version(protocol_version);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_set_protocol_version ? err {:?}", err);

        cb(command_handle, err);
    };

    executor.spawn_ok_instrumented(CommandMetric::PoolCommandSetProtocolVersion, action, cb);

    let res = ErrorCode::Success;

    trace!("indy_set_protocol_version < {:?}", res);

    res
}
