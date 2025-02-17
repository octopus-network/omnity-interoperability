use crate::state::read_state;
use serde_bytes::ByteBuf;

pub fn main_address_path() -> Vec<ByteBuf> {
    let chain_id = read_state(|s| s.chain_id.clone());
    vec![ByteBuf::from(chain_id.as_bytes())]
}

pub fn fee_address_path() -> Vec<ByteBuf> {
    let chain_id = read_state(|s| s.chain_id.clone());
    vec![
        ByteBuf::from(chain_id.as_bytes()),
        ByteBuf::from("FEE".as_bytes()),
    ]
}
