use std::str::FromStr;

use ethers_core::abi::{ethereum_types, AbiEncode};
use ethers_core::types::Eip1559TransactionRequest;
use ethers_core::types::{Bytes, NameOrAddress, U256};
use ic_canister_log::log;

use crate::contract_types::{PrivilegedExecuteDirectiveCall, PrivilegedMintTokenCall};
use crate::eth_common::{EvmAddress};
use crate::state::read_state;
use omnity_types::{Directive, Factor, Ticket};
use crate::convert::{convert_factor_to_port_factor_type_index, directive_to_port_command_index};
use crate::logs::P0;

pub type PortContractCommandIndex = u8;
pub type PortContractFactorTypeIndex = u8;

pub fn gen_execute_directive_data(directive: &Directive, seq: U256) -> Vec<u8> {
    let data = match directive {
        Directive::AddChain(_) | Directive::UpdateChain(_) | Directive::UpdateToken(_) => {
            return vec![];
        }
        Directive::AddToken(token) => {
            if read_state(|s| s.tokens.get(&token.token_id).is_some()) {
                log!(P0, "duplicate issue token id: {}", token.token_id);
                return vec![];
            }
            Bytes::from(
                (
                    token.token_id_info()[0].to_string(),
                    token.token_id.clone(),
                    ethereum_types::Address::from([0u8; 20]),
                    token.name.clone(),
                    token.symbol.clone(),
                    token.decimals,
                )
                    .encode(),
            )
        }
        Directive::ToggleChainState(t) => {
            if t.chain_id == read_state(|s| s.omnity_chain_id.clone()) {
                Bytes::from(t.chain_id.clone().encode())
            } else {
                return vec![];
            }
        }
        Directive::UpdateFee(f) => {
            let factor_index = convert_factor_to_port_factor_type_index(f);
            let data = match f {
                Factor::UpdateTargetChainFactor(factor) => (
                    factor_index,
                    factor.target_chain_id.clone(),
                    factor.target_chain_factor,
                )
                    .encode(),
                Factor::UpdateFeeTokenFactor(factor) => {
                    if factor.fee_token != read_state(|s| s.fee_token_id.clone()) {
                        return vec![];
                    }
                    (
                        factor_index,
                        factor.fee_token.clone(),
                        factor.fee_token_factor,
                    )
                        .encode()
                }
            };
            Bytes::from(data)
        }
    };
    let index: Option<PortContractCommandIndex> = directive_to_port_command_index(directive);
    let data = (index.unwrap(), seq, data).encode();
    PrivilegedExecuteDirectiveCall {
        directive_bytes: Bytes::from(data),
    }
    .encode()
}

pub fn gen_mint_token_data(ticket: &Ticket) -> Vec<u8> {
    let receiver = ethereum_types::Address::from_slice(
        EvmAddress::from_str(ticket.receiver.as_str())
            .unwrap()
            .0
            .as_slice(),
    );
    let amount: u128 = ticket.amount.parse().unwrap();
    PrivilegedMintTokenCall {
        token_id: ticket.token.clone(),
        receiver,
        amount: U256::from(amount),
        ticket_id: ticket.ticket_id.clone(),
        memo: String::from_utf8(ticket.memo.clone().unwrap_or_default()).unwrap_or_default(),
    }
    .encode()
}

pub fn gen_evm_eip1559_tx(
    tx_data: Vec<u8>,
    gas_price: Option<U256>,
    nonce: U256,
    gas: u32,
) -> Eip1559TransactionRequest {
    let chain_id = read_state(|s| s.evm_chain_id);
    let port_contract_addr = read_state(|s| s.omnity_port_contract.clone());
    Eip1559TransactionRequest {
        chain_id: Some(chain_id.into()),
        from: None,
        to: Some(NameOrAddress::Address(port_contract_addr.into())),
        gas: Some(U256::from(gas)),
        value: None,
        nonce: Some(nonce),
        data: Some(Bytes::from(tx_data)),
        access_list: Default::default(),
        max_priority_fee_per_gas: gas_price,
        max_fee_per_gas: gas_price,
    }
}
