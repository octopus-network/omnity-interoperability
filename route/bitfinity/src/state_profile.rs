use crate::state::read_state;
use ethereum_common::traits::{ChainInfo, SignatureBase, StateProvider};
use ethereum_common::tx_types::EvmTxType;
use omnity_types::{ChainId, Token, TokenId};

pub struct BitfinityStateProvider;

impl StateProvider for BitfinityStateProvider {
    fn token_added(token: &Token) -> bool {
        read_state(|s| s.tokens.contains_key(&token.token_id))
    }

    fn get_redeem_fee(chain_id: ChainId) -> Option<u64> {
        read_state(|s| {
            s.target_chain_factor
                .get(&chain_id)
                .map_or(None, |target_chain_factor| {
                    s.fee_token_factor
                        .map(|fee_token_factor| (target_chain_factor * fee_token_factor) as u64)
                })
        })
    }

    fn chain_info() -> ChainInfo {
        read_state(|s| ChainInfo {
            ommnity_chain_id: s.omnity_chain_id.clone(),
            fee_token: s.fee_token_id.clone(),
            port_contract_address: s.omnity_port_contract.clone(),
            evm_id: s.evm_chain_id,
            tx_type: EvmTxType::Eip1559,
        })
    }

    fn get_token(token_id: &TokenId) -> Option<Token> {
        read_state(|s| s.tokens.get(token_id).cloned())
    }

    fn get_signature_base() -> SignatureBase {
        read_state(|s| SignatureBase {
            key_derivation_path: s.key_derivation_path.clone(),
            key_id: s.key_id.clone(),
            public_key: s.pubkey.clone(),
        })
    }
}
