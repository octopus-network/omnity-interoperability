use cosmwasm_schema::cw_serde;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{
    schnorr::{SchnorrKeyIds, SchnorrPublicKeyArgs},
    state,
};

#[cw_serde]
pub enum ExecuteMsg {
    ExecDirective {
        seq: u64,
        directive: Directive,
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

#[cw_serde]
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

// pub async fn execute_msg(msg: ExecuteMsg) -> Result<(), String> {
//     let schnorr_canister_principal: candid::Principal =
//         state::read_state(|state| state.schnorr_canister_principal);

//     let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
//         .iter()
//         .map(|v| ByteBuf::from(v.clone()))
//         .collect();

//     let public_arg = SchnorrPublicKeyArgs {
//         canister_id: Some(ic_cdk::api::id()),
//         derivation_path: derivation_path.clone(),
//         key_id: SchnorrKeyIds::TestKey1.to_key_id(),
//     };

//     Ok(())
// }
