use candid::CandidType;
use serde::Deserialize;
use thiserror::Error;

#[derive(CandidType, Deserialize, Debug, Error)]
pub enum Error {
    #[error("client has been created")]
    ClientHasBeenCreated,
    #[error("connection has been created")]
    ConnectionHasBeenCreated,
    #[error("channel has been created: `{0}`")]
    ChannelHasBeenCreated(String),
    #[error("client state not found: `{0}`")]
    ClientStateNotFound(String),
    #[error("consensus state not found: `{0}`")]
    ConsensusStateNotFound(String),
    #[error("unknown any message")]
    UnknownAnyMessage,
    #[error("the message is malformed and cannot be decoded error")]
    MalformedMessageBytes,
    #[error("unauthorized")]
    Unauthorized,
    #[error("custom error: (`{0}`)")]
    CustomError(String),
}
