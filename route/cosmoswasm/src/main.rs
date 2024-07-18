
use std::collections::HashMap;

use cosmoswasm_route::{lifecycle::{self, init::InitArgs}, state, update::add_new_token::add_new_token};
use ic_cdk::{api::call::{CallResult, RejectionCode}, init, update};
use omnity_types::Token;


#[init]
fn init(args: InitArgs) {
    lifecycle::init::init(args);
}

#[update]
pub async fn msg_send()->Result<(),String> {
    

    Ok(())
}

#[update]
pub async fn test_add_token() {
    add_new_token(Token { 
        token_id: "token_id".to_string(), 
        name: "name".to_string(), 
        symbol: "symbol".to_string(), 
        decimals: 2u8, 
        icon: None, 
        metadata: HashMap::new() 
    }).await.unwrap();
}

fn main() {}

ic_cdk::export_candid!();