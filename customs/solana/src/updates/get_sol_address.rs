use crate::state::read_state;
use candid::{CandidType, Deserialize};
use ic_solana::{eddsa::KeyType, types::Pubkey};
use serde::Serialize;
use serde_bytes::ByteBuf;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GetSolAddressArgs {
    pub target_chain_id: String,
    pub receiver: String,
}

impl GetSolAddressArgs {
    pub fn to_derivation_path(&self) -> Vec<ByteBuf> {
        vec![
            ByteBuf::from(self.target_chain_id.as_bytes()),
            ByteBuf::from(self.receiver.as_bytes()),
        ]
    }
}

pub async fn get_sol_address(args: GetSolAddressArgs) -> Pubkey {
    let key_name = read_state(|s| s.schnorr_key_name.clone());
    let pk =
        ic_solana::eddsa::eddsa_public_key(KeyType::ChainKey, key_name, args.to_derivation_path())
            .await;
    let p: [u8; 32] = pk.try_into().unwrap();
    Pubkey::from(p)
}
