use std::str::FromStr;

use crate::contract_types::{PrivilegedExecuteDirectiveCall, PrivilegedMintTokenCall};
use ethers_core::abi::{ethereum_types, AbiEncode};
use ethers_core::types::{Bytes, Eip1559TransactionRequest, NameOrAddress, U256};
use ethers_core::utils::keccak256;
use serde_derive::{Deserialize, Serialize};

use crate::eth_common::EvmAddress;
use crate::state::read_state;
use crate::types::{Directive, Ticket, ToggleAction};

pub type PortContractCommandIndex = u8;

pub fn gen_execute_directive_data(directive: &Directive, seq: U256) -> Vec<u8> {
    let index: PortContractCommandIndex = directive.clone().into();
    let data = match directive {
        Directive::AddChain(c) => (index, seq, (c.chain_id.clone())).encode(),
        Directive::AddToken(t) => {
            let token = t.clone();
            let t_info = token.token_id_info();
            let settlement_chain_id = t_info[0].to_string();
            let token_id = token.token_id;
            let contract_addr = ethereum_types::Address::from([0u8; 20]);
            let name = token.name;
            let symbol = token.symbol;
            let decimal = token.decimals;
            (
                index,
                seq,
                (
                    settlement_chain_id,
                    token_id,
                    contract_addr,
                    name,
                    symbol,
                    decimal,
                ),
            )
                .encode()
        }
        Directive::ToggleChainState(t) => (index, seq, (t.chain_id.clone())).encode(),
        Directive::UpdateFee(f) => {
            //TODO
            vec![]
        }
    };

    let call = PrivilegedExecuteDirectiveCall {
        directive_bytes: Bytes::from(data),
    };
    call.encode()
}

pub fn gen_mint_token_data(ticket: &Ticket) -> Vec<u8> {
    let receiver = ethereum_types::Address::from_slice(
        EvmAddress::from_str(ticket.receiver.as_str())
            .unwrap()
            .0
            .as_slice(),
    );
    let amount: u128 = ticket.amount.parse().unwrap();
    let call = PrivilegedMintTokenCall {
        token_id: ticket.token.clone(),
        receiver,
        amount: U256::from(amount),
        ticket_id: U256::from_str_radix(ticket.ticket_id.as_str(), 16).unwrap(),
        memo: String::from_utf8(ticket.memo.clone().unwrap_or_default()).unwrap_or_default(),
    };
    call.encode()
}

//TODO confirm the rule is correctly
impl Into<PortContractCommandIndex> for Directive {
    fn into(self) -> PortContractCommandIndex {
        match self {
            Directive::AddChain(_) => 0u8,
            Directive::AddToken(_) => 1u8,
            Directive::UpdateFee(_) => 2u8,
            Directive::ToggleChainState(t) => match t.action {
                ToggleAction::Activate => 4,
                ToggleAction::Deactivate => 3,
            },
        }
    }
}

pub fn gen_eip1559_tx(tx_data: Vec<u8>) -> Eip1559TransactionRequest {
    let chain_id = read_state(|s| s.evm_chain_id);
    let port_contract_addr = read_state(|s| s.omnity_port_contract.clone());
    let tx = Eip1559TransactionRequest {
        chain_id: Some(chain_id.into()),
        from: None,
        to: Some(NameOrAddress::Address(port_contract_addr.into())),
        gas: None,
        value: None,
        nonce: None,
        data: Some(Bytes::from(tx_data)),
        access_list: Default::default(),
        max_priority_fee_per_gas: None,
        max_fee_per_gas: None,
    };
    tx
}
