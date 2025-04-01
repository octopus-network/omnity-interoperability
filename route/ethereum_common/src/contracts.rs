use ethers_core::abi::{ethereum_types, AbiEncode};
use ethers_core::types::Eip1559TransactionRequest;
use ethers_core::types::{Bytes, NameOrAddress, TransactionRequest, U256};
use std::str::FromStr;

use omnity_types::{Directive, Factor, Ticket};

use crate::address::EvmAddress;
use crate::contract_types::{PrivilegedExecuteDirectiveCall, PrivilegedMintTokenCall};
use crate::convert::{convert_factor_to_port_factor_type_index, directive_to_port_command_index};
use crate::traits::StateProvider;
use crate::tx_types::{EvmTxRequest, EvmTxType};

pub type PortContractCommandIndex = u8;
pub type PortContractFactorTypeIndex = u8;

pub fn gen_execute_directive_data<P: StateProvider>(directive: &Directive, seq: U256) -> Vec<u8> {
    let data = match directive {
        Directive::AddChain(_) | Directive::UpdateChain(_) | Directive::UpdateToken(_) => {
            return vec![];
        }
        Directive::AddToken(token) => {
            if P::token_added(token) {
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
            if t.chain_id == P::chain_info().ommnity_chain_id {
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
                    if factor.fee_token != P::chain_info().fee_token {
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

pub fn gen_evm_tx<P: StateProvider>(
    tx_data: Vec<u8>,
    gas_price: Option<U256>,
    nonce: u64,
    gas: u32,
) -> EvmTxRequest {
    match P::chain_info().tx_type {
        EvmTxType::Legacy => {
            EvmTxRequest::Legacy(gen_evm_legacy_tx::<P>(tx_data, gas_price, nonce, gas))
        }
        EvmTxType::Eip1559 => {
            EvmTxRequest::Eip1559(gen_evm_eip1559_tx::<P>(tx_data, gas_price, nonce, gas))
        }
    }
}

pub fn gen_evm_legacy_tx<P: StateProvider>(
    tx_data: Vec<u8>,
    gas_price: Option<U256>,
    nonce: u64,
    gas: u32,
) -> TransactionRequest {
    let chain_info = P::chain_info();
    TransactionRequest {
        chain_id: Some(chain_info.evm_id.into()),
        from: None,
        to: Some(NameOrAddress::Address(
            chain_info.port_contract_address.into(),
        )),
        gas: Some(U256::from(gas)),
        gas_price,
        value: None,
        nonce: Some(U256::from(nonce)),
        data: Some(Bytes::from(tx_data)),
    }
}

pub fn gen_evm_eip1559_tx<P: StateProvider>(
    tx_data: Vec<u8>,
    gas_price: Option<U256>,
    nonce: u64,
    gas: u32,
) -> Eip1559TransactionRequest {
    let chain_info = P::chain_info();
    Eip1559TransactionRequest {
        chain_id: Some(chain_info.evm_id.into()),
        from: None,
        to: Some(NameOrAddress::Address(
            chain_info.port_contract_address.into(),
        )),
        gas: Some(U256::from(gas)),
        value: None,
        nonce: Some(U256::from(nonce)),
        data: Some(Bytes::from(tx_data)),
        access_list: Default::default(),
        max_priority_fee_per_gas: gas_price,
        max_fee_per_gas: gas_price,
    }
}
