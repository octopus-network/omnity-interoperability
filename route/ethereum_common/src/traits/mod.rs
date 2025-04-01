use crate::address::EvmAddress;
use crate::tx_types::EvmTxType;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use omnity_types::{ChainId, Memo, Token, TokenId};

pub trait StateProvider {
    fn token_added(token: &Token) -> bool;
    fn get_redeem_fee(chain_id: ChainId) -> Option<u64>;
    fn chain_info() -> ChainInfo;
    fn get_token(token_id: &TokenId) -> Option<Token>;
    fn get_signature_base() -> SignatureBase;
}

pub fn get_memo<P: StateProvider>(memo: Option<String>, dst_chain: ChainId) -> Option<String> {
    let fee = P::get_redeem_fee(dst_chain);
    let memo_json = Memo {
        memo,
        bridge_fee: fee.unwrap_or_default() as u128,
    }
    .convert_to_memo_json()
    .unwrap_or_default();
    Some(memo_json)
}

pub struct SignatureBase {
    pub key_derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
    pub public_key: Vec<u8>,
}
pub struct ChainInfo {
    pub ommnity_chain_id: ChainId,
    pub fee_token: TokenId,
    pub port_contract_address: EvmAddress,
    pub evm_id: u64,
    pub tx_type: EvmTxType,
}
