use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExecuteMsg {
    ExecDirective {
        seq: u64,
        directive: Directive,
        signature: Vec<u8>,
    },
    // PrivilegeMintToken {
    //     ticket_id: String,
    //     token_id: String,
    //     receiver: Addr,
    //     amount: String,
    // },
    RedeemToken {
        token_id: String,
        receiver: String,
        amount: String,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Directive {
    AddToken {
        settlement_chain: String,
        token_id: String,
        name: String,
    },
    // UpdateFee {
    //     factor: Factor,
    // },
}