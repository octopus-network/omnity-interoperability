use thiserror::Error;

pub type OrdResult<T> = std::result::Result<T, OrdError>;

/// Ordinal transaction handling error types
#[derive(Error, Debug)]
pub enum OrdError {
    #[error("Hex codec error: {0}")]
    HexCodec(#[from] hex::FromHexError),
    #[error("Ord codec error: {0}")]
    Codec(#[from] serde_json::Error),
    #[error("Bitcoin sighash error: {0}")]
    BitcoinSigHash(#[from] bitcoin::sighash::Error),
    #[error("Bitcoin script error: {0}")]
    PushBytes(#[from] bitcoin::script::PushBytesError),
    #[error("Bad transaction input: {0}")]
    InputNotFound(usize),
    #[error("Insufficient balance")]
    InsufficientBalance { required: u64, available: u64 },
    #[error("Invalid signature: {0}")]
    Signature(#[from] bitcoin::secp256k1::Error),
    #[error("Failed to convert slice to public key: {0}")]
    PubkeyConversion(#[from] bitcoin::key::Error),
    #[error("Invalid signature")]
    UnexpectedSignature,
    #[error("Taproot builder error: {0}")]
    TaprootBuilder(#[from] bitcoin::taproot::TaprootBuilderError),
    #[error("Taproot compute error")]
    TaprootCompute,
    #[error("Scripterror: {0}")]
    Script(#[from] bitcoin::blockdata::script::Error),
    #[error("Invalid UTF-8 in: {0}")]
    Utf8Encoding(#[from] std::str::Utf8Error),
    #[error("Inscription parser error: {0}")]
    InscriptionParser(#[from] InscriptionParseError),
    #[error("management error: rs:{0}")]
    ManagementError(String),
}

/// Inscription parsing errors.
#[derive(Error, Debug)]
pub enum InscriptionParseError {
    #[error("invalid transaction id: {0}")]
    Txid(#[from] bitcoin::hashes::hex::HexToArrayError),
    #[error("invalid character: {0}")]
    Character(char),
    #[error("invalid length: {0}")]
    InscriptionIdLength(usize),
    #[error("invalid separator: {0}")]
    CharacterSeparator(char),
    #[error("invalid index: {0}")]
    Index(#[from] std::num::ParseIntError),
    #[error("content of envelope: {0}")]
    ParsedEnvelope(String),
    #[error("cannot convert non-Ordinal inscription to Nft")]
    NotOrdinal,
    #[error("cannot convert non-Brc20 inscription to Brc20")]
    NotBrc20,
}
