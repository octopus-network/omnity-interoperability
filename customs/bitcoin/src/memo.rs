use minicbor::Encoder;
use minicbor::{Decode, Encode};

/// Encodes minter memo as a binary blob.
pub fn encode<T: minicbor::Encode<()>>(t: &T) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.encode(t).expect("minicbor encoding failed");
    encoder.into_writer()
}

#[derive(Debug, Decode, Encode, Eq, PartialEq)]
#[cbor(index_only)]
pub enum Status {
    #[n(0)]
    /// The minter accepted a retrieve_btc request.
    Accepted,
    /// The minter rejected a retrieve_btc due to a failed KYT check.
    #[n(1)]
    Rejected,
    #[n(2)]
    CallFailed,
}
