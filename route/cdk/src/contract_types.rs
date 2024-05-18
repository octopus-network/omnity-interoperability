use ethers_core::abi::{ethereum_types, AbiDecode, AbiEncode, RawLog};
use ethers_core::types::U256;
use ethers_core::utils::keccak256;
use serde_derive::{Deserialize, Serialize};

pub trait AbiSignature {
    fn abi_signature() -> String;
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
    pub ticket_id: ::ethers_core::types::U256,
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
        "privilegedMintToken(string,address,uint256,uint256,string)".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMinted {
    pub token_id: String,
    pub receiver: ethereum_types::Address,
    pub amount: U256,
    pub ticket_id: U256,
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
        "TokenMinted(string,address,uint256,uint256,string)".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransportRequested {
    pub dst_chain_id: String,
    pub token_id: String,
    pub receiver: String,
    pub amount: U256,
    pub channel_id: String,
    pub memo: String,
}

impl DecodeLog for TokenTransportRequested {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self> {
        let (dst_chain_id, token_id, receiver, amount, channel_id, memo) =
            AbiDecode::decode(&log.data)?;
        Ok(Self {
            dst_chain_id,
            token_id,
            receiver,
            amount,
            channel_id,
            memo,
        })
    }
}
impl AbiSignature for TokenTransportRequested {
    fn abi_signature() -> String {
        "TokenTransportRequested(string,string,string,uint256,string,string)".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBurned {
    pub token_id: String,
    pub receiver: String,
    pub amount: ethereum_types::U256,
    pub channel_id: String,
}

impl AbiSignature for TokenBurned {
    fn abi_signature() -> String {
        "TokenBurned(string,string,uint256,string)".to_string()
    }
}

impl DecodeLog for TokenBurned {
    fn decode_log(log: &RawLog) -> anyhow::Result<Self> {
        let (token_id, receiver, amount, channel_id) = AbiDecode::decode(log.data.to_vec())?;
        Ok(Self {
            token_id,
            receiver,
            amount,
            channel_id,
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

#[cfg(test)]
mod test {
    use ethers_contract::abigen;
    use ethers_core::abi::{ethereum_types, AbiEncode};
    use ethers_core::types::{Bytes, U256};
    abigen!(
        OmnityPortContract,
        r#"[
        function privilegedMintToken(string tokenId,address receiver,uint256 amount,uint256 ticketId, string memory memo) external
        function privilegedExecuteDirective(bytes memory directiveBytes) external
        event TokenMinted(string tokenId,address receiver,uint256 amount,uint256 ticketId,string memo)
        event TokenTransportRequested(string dstChainId,string tokenId,string receiver,uint256 amount,string channelId,string memo)
        event TokenBurned(string tokenId,string receiver,uint256 amount,string channelId)
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
            ticket_id: U256::from(1000),
            memo: "".to_string(),
        };

        let call2 = crate::contract_types::PrivilegedMintTokenCall {
            token_id: "122".to_string(),
            receiver: ethereum_types::Address::from([0u8; 20]),
            amount: U256::from(10),
            ticket_id: U256::from(1000),
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
    }
}
