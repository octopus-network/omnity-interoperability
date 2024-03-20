pub mod auth;
pub mod args;

use crate::*;
use args::InitArgs;
use auth::auth_port;

#[ic_cdk::init]
pub fn init(args: InitArgs) {
}

// #[update(name = "redeem", guard = "auth_port")]
// pub async fn redeem()->Result {

// }

