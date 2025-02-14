
#[allow(unused)]
use serde::{Deserialize, Serialize};

pub use balance_changes::*;
pub use object_changes::*;

pub use sui_coin::*;
pub use sui_event::*;

pub use sui_move::*;

mod balance_changes;
mod object_changes;
// mod sui_checkpoint;
mod sui_coin;
// mod sui_displays;
mod sui_event;
mod sui_move;
// mod sui_protocol;
pub mod sui_object;
pub mod sui_transaction;

// pub type DynamicFieldPage = Page<DynamicFieldInfo, ObjectId>;
/// `next_cursor` points to the last item in the page;
/// Reading with `next_cursor` will start from the next item after `next_cursor` if
/// `next_cursor` is `Some`, otherwise it will start from the first item.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Page<T, C> {
    pub data: Vec<T>,
    pub next_cursor: Option<C>,
    pub has_next_page: bool,
}

impl<T, C> Page<T, C> {
    pub fn empty() -> Self {
        Self {
            data: vec![],
            next_cursor: None,
            has_next_page: false,
        }
    }
}

pub const CLIENT_REQUEST_METHOD_HEADER: &str = "client-request-method";
pub const CLIENT_SDK_TYPE_HEADER: &str = "client-sdk-type";
/// The version number of the SDK itself. This can be different from the API version.
pub const CLIENT_SDK_VERSION_HEADER: &str = "client-sdk-version";
/// The RPC API version that the client is targeting. Different SDK versions may target the same
/// API version.
pub const CLIENT_TARGET_API_VERSION_HEADER: &str = "client-target-api-version";

pub const TRANSIENT_ERROR_CODE: i32 = -32050;
pub const TRANSACTION_EXECUTION_CLIENT_ERROR_CODE: i32 = -32002;
