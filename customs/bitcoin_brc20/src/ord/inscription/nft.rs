//! NFT
//! Closely follows https://github.com/ordinals/ord/blob/master/src/inscriptions/inscription.rs

use std::mem;
use std::str::FromStr;

use bitcoin::constants::MAX_SCRIPT_ELEMENT_SIZE;
use bitcoin::opcodes;
use bitcoin::opcodes::all::OP_CHECKSIG;
use bitcoin::script::{Builder as ScriptBuilder, PushBytes, PushBytesBuf};
use serde::{Deserialize, Serialize};

use crate::ord::builder::RedeemScriptPubkey;
use crate::ord::inscription::Inscription;
use crate::ord::parser::constants;
use crate::ord::parser::push_bytes::bytes_to_push_bytes;
use crate::ord::result::{OrdError, OrdResult};

/// Represents an arbitrary Ordinal inscription. We're "unofficially" referring to this as an NFT
/// (e.g., like an ERC721 token). Ordinal inscriptions allow for the embedding of data directly
/// into individual satoshis on the Bitcoin blockchain, enabling a unique form of digital
/// artifact creation and ownership tracking.
///
/// NFTs may include fields before an optional body. Each field consists of two data pushes,
/// a tag and a value.
///
/// [Reference](https://docs.ordinals.com/inscriptions.html#fields)
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Nft {
    /// The main body of the NFT. This is the core data or content of the NFT,
    /// which might represent an image, text, or other types of digital assets.
    pub body: Option<Vec<u8>>,
    /// Has a tag of 1, representing the MIME type of the body. This describes
    /// the format of the body content, such as "image/png" or "text/plain".
    pub content_type: Option<Vec<u8>>,
    /// Has a tag of 2, representing the position of the inscribed satoshi in the outputs.
    /// It is used to locate the specific satoshi that carries this inscription.
    pub pointer: Option<Vec<u8>>,
    /// Has a tag of 3, representing the parent NFTs, i.e., the owner of an NFT
    /// can create child NFTs, establishing a hierarchy or a collection of related NFTs.
    pub parents: Vec<Vec<u8>>,
    /// Has a tag of 5, representing CBOR (Concise Binary Object Representation) metadata,
    /// stored as data pushes. This is used for storing structured metadata in a compact format.
    pub metadata: Option<Vec<u8>>,
    /// Has a tag of 7, representing the metaprotocol identifier. This field specifies
    /// which metaprotocol, if any, this NFT follows, allowing for interoperability and
    /// standardized behavior across different systems.
    pub metaprotocol: Option<Vec<u8>>,
    /// Indicates if any field is incomplete. This is used
    /// to signal that the data is partially constructed or awaiting further information.
    pub incomplete_field: bool,
    /// Indicates if there are any duplicate fields. Duplicate fields
    /// could arise from errors in data entry or processing and may need resolution for the
    /// this structure to be considered valid.
    pub duplicate_field: bool,
    /// Has a tag of 9, representing the encoding of the body. This specifies how the body content
    /// is encoded, such as "base64" or "utf-8", providing guidance on how to interpret or display the content.
    pub content_encoding: Option<Vec<u8>>,
    /// Indicates if there are any unrecognized even fields. Even tags are reserved for future use
    /// and should not be used by current implementations.
    pub unrecognized_even_field: bool,
    /// Has a tag of 11, representing a nominated NFT. Used to delegate certain rights or
    /// attributes from one NFT to another, effectively linking them in a specified relationship.
    pub delegate: Option<Vec<u8>>,
}