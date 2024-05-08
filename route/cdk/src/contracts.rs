use std::str::FromStr;
use cketh_common::eth_rpc_client::RpcConfig;

use ethers_contract::abigen;
use ethers_core::abi::{AbiEncode, ethereum_types};
use ethers_core::types::{Bytes, Eip1559TransactionRequest, NameOrAddress, U256};
use ethers_core::utils::keccak256;
use evm_rpc::candid_types::SendRawTransactionStatus;
use evm_rpc::RpcServices;
use hex::ToHex;
use ic_cdk::api::management_canister::ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument};
use secp256k1::{Message, PublicKey};
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
use crate::Error;

use crate::evm_address::EvmAddress;
use crate::state::read_state;
use crate::types::{Directive, Ticket, ToggleState};
pub type PortContractCommandIndex = u8;

abigen!(
    OmnityPortContract,
    r#"[
        function privilegedMintToken(bytes32 tokenId,address receiver,uint256 amount,uint256 ticketId, string memory memo) external
        function privilegedExecuteDirective(bytes memory directiveBytes) external
    ]"#,
    derives(serde::Deserialize, serde::Serialize)

);

pub fn gen_execute_directive_data(directive: &Directive) -> Vec<u8> {
    match directive {
        Directive::AddChain(c) => {



        }
        Directive::AddToken(_) => {


        }
        Directive::ToggleChainState(_) => {}
        Directive::UpdateFee(_) => {}
    }




    let v: Vec<u8> = vec![];
    let call = PrivilegedExecuteDirectiveCall {
        directive_bytes: Bytes::from(v),
    };
    call.encode()
}

pub fn gen_mint_token_data(ticket: &Ticket) -> Vec<u8> {
    let token_id = ticket.token.clone();
    let receiver = ethereum_types::Address::from_slice(EvmAddress::from_str(ticket.receiver.as_str()).unwrap().0.as_slice());
    let amount: u128 = ticket.amount.parse().unwrap();
    let call = PrivilegedMintTokenCall {
        token_id: [1u8;32],
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
            Directive::ToggleChainState(t) => {

                3
            }
            Directive::UpdateFee(_) => 2u8
        }
    }
}

pub fn gen_eip1559_tx(tx_data: Vec<u8>) -> Eip1559TransactionRequest {
    let chain_id = read_state(|s|s.evm_chain_id);
    let port_contract_addr = read_state(|s|s.omnity_port_contract.clone());
    let tx = Eip1559TransactionRequest {
        chain_id: Some(chain_id.into()),
        from: None,
        to: Some(
            NameOrAddress::Address(port_contract_addr.into())
        ),
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
pub async fn sign_transaction( tx: Eip1559TransactionRequest ) -> anyhow::Result<Vec<u8>> {
    use ethers_core::types::Signature;

    const EIP1559_TX_ID: u8 = 2;

    let caller = ic_cdk::caller();
    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);
    let txhash = keccak256(&unsigned_tx_bytes);
    let arg = SignWithEcdsaArgument {
        message_hash: txhash.clone().to_vec(),
        derivation_path: crate::state::key_derivation_path(),
        key_id: crate::state::key_id(),
    };
    // The signatures are encoded as the concatenation of the 32-byte big endian encodings of the two values r and s.
    let (r,) = sign_with_ecdsa(arg)
        .await
        .map_err(|(_, e)| super::Error::ChainKeyError(e))?;
    let chain_id = crate::state::target_chain_id();
    let signature = EthereumSignature::try_from_ecdsa(
        &r.signature,
        &txhash,
        chain_id,
        crate::state::try_public_key()?.as_ref(),
    )?;

    let signature = Signature {
        v: signature.v,
        r: U256::from_big_endian(&signature.r),
        s: U256::from_big_endian(&signature.s),
    };
    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);
    Ok(signed_tx_bytes)
}


pub async fn broadcast(tx: Vec<u8>) -> Result<String, super::Error> {
    let raw = hex::encode(tx);
    let (r,): (SendRawTransactionStatus,) = ic_cdk::call(
        crate::state::rpc_addr(),
        "eth_sendRawTransaction",
        (
            RpcServices::Custom {
                chain_id: crate::state::target_chain_id(),
                services: crate::state::rpc_providers(),
            },
            None::<RpcConfig>,
            raw,
        ),
    )
        .await
        .map_err(|(_, e)| super::Error::EvmRpcError(e))?;
    match r {
        SendRawTransactionStatus::Ok(hash) => hash.map(|h| h.to_string()).ok_or(
            super::Error::EvmRpcError("A transaction hash is expected".to_string()),
        ),
        _ => Err(super::Error::EvmRpcError(format!("{:?}", r))),
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EthereumSignature {
    pub r: Vec<u8>,
    pub s: Vec<u8>,
    pub v: u64,
}

impl EthereumSignature {
    pub(crate) fn try_from_ecdsa(
        signature: &[u8],
        prehash: &[u8],
        chain_id: u64,
        pubkey: &[u8],
    ) -> Result<Self, Error> {
        let mut r = signature[..32].to_vec();
        let mut s = signature[32..].to_vec();
        while r[0] == 0 {
            r.remove(0);
        }
        while s[0] == 0 {
            s.remove(0);
        }
        let v = Self::try_derive_recid(signature, prehash, chain_id, pubkey)?;
        Ok(Self { r, s, v })
    }

    fn try_derive_recid(
        signature: &[u8],
        prehash: &[u8],
        chain_id: u64,
        pubkey: &[u8],
    ) -> Result<u64, Error> {
        let pubkey = PublicKey::from_slice(pubkey)
            .map_err(|_| Error::ChainKeyError("invalid public key".to_string()))?;
        let digest = Message::from_digest_slice(prehash)
            .map_err(|_| Error::ChainKeyError("invalid signature".to_string()))?;
        for r in 0..4 {
            let rec_id = RecoveryId::from_i32(r).expect("less than 4;qed");
            let sig = RecoverableSignature::from_compact(signature, rec_id)
                .map_err(|_| Error::ChainKeyError("invalid signature length".to_string()))?;
            if let Ok(pk) = sig.recover(&digest) {
                if pk == pubkey {
                    return Ok(r as u64 + chain_id * 2 + 35);
                }
            }
        }
        Err(Error::ChainKeyError("invalid signature".to_string()))
    }
}
