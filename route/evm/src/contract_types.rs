use ethers_core::abi::{ethereum_types, AbiDecode, AbiEncode, RawLog};
use ethers_core::types::U256;
use ethers_core::utils::keccak256;
use serde_derive::{Deserialize, Serialize};

pub trait AbiSignature {
    fn abi_signature() -> String;
    fn signature_hash() -> [u8; 32] {
        keccak256(Self::abi_signature().as_bytes())
    }
    fn signature_hex() -> String {
        Self::signature_hash().encode_hex()
    }
}
pub trait DecodeLog {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self>
    where
        Self: Sized;
}
#[derive(Debug, Eq, PartialEq)]
pub struct PrivilegedExecuteDirectiveCall {
    pub directive_bytes: ::ethers_core::types::Bytes,
}

impl AbiEncode for PrivilegedExecuteDirectiveCall {
    fn encode(self) -> Vec<u8> {
        let signature = keccak256(PrivilegedExecuteDirectiveCall::abi_signature());
        let mut v = vec![];
        v.append(&mut signature[0..4].to_vec());
        let mut data = self.directive_bytes.encode();
        v.append(&mut data);
        v
    }
}

impl AbiSignature for PrivilegedExecuteDirectiveCall {
    fn abi_signature() -> String {
        "privilegedExecuteDirective(bytes)".into()
    }
}

pub struct PrivilegedMintTokenCall {
    pub token_id: ::std::string::String,
    pub receiver: ::ethers_core::types::Address,
    pub amount: ::ethers_core::types::U256,
    pub ticket_id: String,
    pub memo: ::std::string::String,
}
impl AbiEncode for PrivilegedMintTokenCall {
    fn encode(self) -> Vec<u8> {
        let signature = keccak256(PrivilegedMintTokenCall::abi_signature());
        let mut v = vec![];
        v.append(&mut signature[0..4].to_vec());
        let mut data = (
            self.token_id,
            self.receiver,
            self.amount,
            self.ticket_id,
            self.memo,
        )
            .encode();
        v.append(&mut data);
        v
    }
}

impl AbiSignature for PrivilegedMintTokenCall {
    fn abi_signature() -> String {
        "privilegedMintToken(string,address,uint256,string,string)".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMinted {
    pub token_id: String,
    pub receiver: ethereum_types::Address,
    pub amount: U256,
    pub ticket_id: String,
    pub memo: String,
}

impl DecodeLog for TokenMinted {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self> {
        let (token_id, receiver, amount, ticket_id, memo) = AbiDecode::decode(&log.data)?;
        Ok(Self {
            token_id,
            receiver,
            amount,
            ticket_id,
            memo,
        })
    }
}

impl AbiSignature for TokenMinted {
    fn abi_signature() -> String {
        "TokenMinted(string,address,uint256,string,string)".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransportRequested {
    pub dst_chain_id: String,
    pub token_id: String,
    pub sender: ethereum_types::Address,
    pub receiver: String,
    pub amount: U256,
    pub memo: String,
}

impl DecodeLog for TokenTransportRequested {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self> {
        let (dst_chain_id, token_id, sender, receiver, amount, memo) =
            AbiDecode::decode(&log.data)?;
        Ok(Self {
            dst_chain_id,
            token_id,
            sender,
            receiver,
            amount,
            memo,
        })
    }
}
impl AbiSignature for TokenTransportRequested {
    fn abi_signature() -> String {
        "TokenTransportRequested(string,string,address,string,uint256,string)".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBurned {
    pub token_id: String,
    pub sender: ethereum_types::Address,
    pub receiver: String,
    pub amount: ethereum_types::U256,
}

impl AbiSignature for TokenBurned {
    fn abi_signature() -> String {
        "TokenBurned(string,address,string,uint256)".to_string()
    }
}

impl DecodeLog for TokenBurned {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self> {
        let (token_id, sender, receiver, amount) = AbiDecode::decode(&log.data)?;
        Ok(Self {
            token_id,
            sender,
            receiver,
            amount,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveExecuted {
    pub seq: U256,
}

impl AbiSignature for DirectiveExecuted {
    fn abi_signature() -> String {
        "DirectiveExecuted(uint256)".into()
    }
}

impl DecodeLog for DirectiveExecuted {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self> {
        let u = U256::decode(&log.data)?;
        Ok(Self { seq: u })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAdded {
    pub token_id: String,
    pub token_address: ethereum_types::Address,
}

impl AbiSignature for TokenAdded {
    fn abi_signature() -> String {
        "TokenAdded(string,address)".into()
    }
}

impl DecodeLog for TokenAdded {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let (token_id, token_address) = AbiDecode::decode(&log.data)?;
        Ok(Self {
            token_id,
            token_address,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunesMintRequested {
    pub token_id: String,
    pub sender: ethereum_types::Address,
    pub receiver: ethereum_types::Address,
}

impl AbiSignature for RunesMintRequested {
    fn abi_signature() -> String {
        "RunesMintRequested(string,address,address)".to_string()
    }
}

impl DecodeLog for RunesMintRequested {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let (token_id, sender, receiver) = AbiDecode::decode(&log.data)?;
        Ok(Self {
            token_id,
            sender,
            receiver,
        })
    }
}

#[cfg(test)]
mod test {
    use ethers_contract::abigen;
    use ethers_core::abi::{ethereum_types, AbiEncode};
    use ethers_core::types::{Bytes, U256};

    use crate::contract_types::{AbiSignature, TokenBurned};

    abigen!(
        OmnityPortContract,
        r#"[
        function privilegedMintToken(string tokenId,address receiver,uint256 amount,string memory ticketId, string memory memo) external
        function privilegedExecuteDirective(bytes memory directiveBytes) external
        event TokenMinted(string tokenId,address receiver,uint256 amount,uint256 ticketId,string memo)
        event TokenTransportRequested(string dstChainId,string tokenId,string receiver,uint256 amount,string memo)
        event TokenBurned(string tokenId,string receiver,uint256 amount)
        event DirectiveExecuted(uint256 seq)
        function tsx(string id)
    ]"#,
        derives(serde::Deserialize, serde::Serialize)
    );

    #[test]
    pub fn tex() {
        let call1 = PrivilegedMintTokenCall {
            token_id: "122".to_string(),
            receiver: ethereum_types::Address::from([1u8; 20]),
            amount: U256::from(10),
            ticket_id: "U256::from(1000)".to_string(),
            memo: "".to_string(),
        };

        let call2 = crate::contract_types::PrivilegedMintTokenCall {
            token_id: "122".to_string(),
            receiver: ethereum_types::Address::from([1u8; 20]),
            amount: U256::from(10),
            ticket_id: "U256::from(1000)".to_string(),
            memo: "".to_string(),
        };

        assert_eq!(call1.encode(), call2.encode());

        let call1 = PrivilegedExecuteDirectiveCall {
            directive_bytes: Bytes::from("hahah".as_bytes().to_vec()),
        };
        let call2 = crate::contract_types::PrivilegedExecuteDirectiveCall {
            directive_bytes: Bytes::from("hahah".as_bytes().to_vec()),
        };
        assert_eq!(call1.encode(), call2.encode());
        println!("{}", hex::encode(TokenBurned::signature_hash()));
    }
}
