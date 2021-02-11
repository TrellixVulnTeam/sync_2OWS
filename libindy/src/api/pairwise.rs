use indy_api_types::{
    errors::prelude::*, validation::Validatable, CommandHandle, ErrorCode, WalletHandle,
};

use indy_utils::ctypes;
use libc::c_char;

use crate::{commands::Locator, domain::crypto::did::DidValue};
use crate::services::metrics::command_metrics::CommandMetric;

/// Check if pairwise is exists.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encrypted DID
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// exists: true - if pairwise is exists, false - otherwise
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_is_pairwise_exists(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    their_did: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, exists: bool)>,
) -> ErrorCode {
    trace!(
        "indy_is_pairwise_exists > wallet_handle {:?} their_did {:?}",
        wallet_handle,
        their_did
    );

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!(
        "indy_is_pairwise_exists ? wallet_handle {:?} their_did {:?}",
        wallet_handle,
        their_did
    );

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pairwise_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller.pairwise_exists(wallet_handle, their_did).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, exists) = prepare_result_1!(res, false);
        trace!(
            "indy_is_pairwise_exists ? err {:?} exists {:?}",
            err,
            exists
        );
        cb(command_handle, err, exists)
    };

    executor.spawn_ok_instrumented(CommandMetric::PairwiseCommandPairwiseExists, action, cb);

    let res = ErrorCode::Success;
    trace!("indy_is_pairwise_exists < {:?}", res);
    res
}

/// Creates pairwise.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encrypted DID
/// my_did: encrypted DID
/// metadata Optional: extra information for pairwise
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_create_pairwise(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    their_did: *const c_char,
    my_did: *const c_char,
    metadata: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    trace!(
        "indy_create_pairwise > wallet_handle {:?} \
            their_did {:?} my_did {:?} metadata {:?}",
        wallet_handle,
        their_did,
        my_did,
        metadata
    );

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_validatable_string!(my_did, ErrorCode::CommonInvalidParam4, DidValue);
    check_useful_opt_c_str!(metadata, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!(
        "indy_create_pairwise ? wallet_handle {:?} \
            their_did {:?} my_did {:?} metadata {:?}",
        wallet_handle,
        their_did,
        my_did,
        metadata
    );

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pairwise_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .create_pairwise(wallet_handle, their_did, my_did, metadata)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_create_pairwise ? err {:?}", err);
        cb(command_handle, err)
    };

    executor.spawn_ok_instrumented(CommandMetric::PairwiseCommandCreatePairwise, action, cb);

    let res = ErrorCode::Success;
    trace!("indy_create_pairwise < {:?}", res);
    res
}

/// Get list of saved pairwise.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// list_pairwise: list of saved pairwise
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_list_pairwise(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, list_pairwise: *const c_char),
    >,
) -> ErrorCode {
    trace!("indy_list_pairwise > wallet_handle {:?}", wallet_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_list_pairwise ? wallet_handle {:?}", wallet_handle);

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pairwise_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller.list_pairwise(wallet_handle).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result_1!(res, String::new());
        trace!("indy_list_pairwise ? err {:?} res {:?}", err, res);

        let res = ctypes::string_to_cstring(res);
        cb(command_handle, err, res.as_ptr())
    };

    executor.spawn_ok_instrumented(CommandMetric::PairwiseCommandListPairwise, action, cb);

    let res = ErrorCode::Success;
    trace!("indy_list_pairwise < {:?}", res);
    res
}

/// Gets pairwise information for specific their_did.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encoded Did
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// pairwise_info_json: did info associated with their did
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_get_pairwise(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    their_did: *const c_char,
    cb: Option<
        extern "C" fn(
            command_handle_: CommandHandle,
            err: ErrorCode,
            pairwise_info_json: *const c_char,
        ),
    >,
) -> ErrorCode {
    trace!(
        "indy_get_pairwise > wallet_handle {:?} their_did {:?}",
        wallet_handle,
        their_did
    );

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!(
        "indy_get_pairwise ? wallet_handle {:?} their_did {:?}",
        wallet_handle,
        their_did
    );

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pairwise_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller.get_pairwise(wallet_handle, their_did).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result_1!(res, String::new());
        trace!("indy_get_pairwise ? err {:?} res {:?}", err, res);

        let res = ctypes::string_to_cstring(res);
        cb(command_handle, err, res.as_ptr())
    };

    executor.spawn_ok_instrumented(CommandMetric::PairwiseCommandGetPairwise, action, cb);

    let res = ErrorCode::Success;
    trace!("indy_get_pairwise < {:?}", res);
    res
}

/// Save some data in the Wallet for pairwise associated with Did.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encoded Did
/// metadata: some extra information for pairwise
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_set_pairwise_metadata(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    their_did: *const c_char,
    metadata: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    trace!(
        "indy_set_pairwise_metadata > wallet_handle {:?} \
            their_did {:?} metadata {:?}",
        wallet_handle,
        their_did,
        metadata
    );

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_opt_c_str!(metadata, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!(
        "indy_set_pairwise_metadata ? wallet_handle {:?} \
            their_did {:?} metadata {:?}",
        wallet_handle,
        their_did,
        metadata
    );

    let (executor, controller) = {
        let locator = Locator::instance();
        let executor = locator.executor.clone();
        let controller = locator.pairwise_command_executor.clone();
        (executor, controller)
    };

    let action = async move {
        let res = controller
            .set_pairwise_metadata(wallet_handle, their_did, metadata)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        trace!("indy_set_pairwise_metadata ? err {:?}", err);
        cb(command_handle, err)
    };

    executor.spawn_ok_instrumented(CommandMetric::PairwiseCommandSetPairwiseMetadata, action, cb);

    let res = ErrorCode::Success;
    trace!("indy_set_pairwise_metadata < {:?}", res);
    res
}
