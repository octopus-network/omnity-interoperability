//! NFT
//! Closely follows https://github.com/ordinals/ord/blob/master/src/inscriptions/inscription.rs

use std::io::Cursor;
use std::mem;
use std::str::FromStr;

use bitcoin::constants::MAX_SCRIPT_ELEMENT_SIZE;
use bitcoin::opcodes;
use bitcoin::opcodes::all::OP_CHECKSIG;
use bitcoin::script::{Builder as ScriptBuilder, PushBytes, PushBytesBuf, ScriptBuf};
use serde::{Deserialize, Serialize};
use crate::ord::builder::RedeemScriptPubkey;
use crate::ord::inscription::Inscription;
use crate::ord::parser::constants;
use crate::ord::parser::push_bytes::bytes_to_push_bytes;
use crate::ord::result::{InscriptionParseError, OrdError, OrdResult};

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
    /// Has a tag of 13, denoting whether or not this inscription caries any rune.
    pub rune: Option<Vec<u8>>,
}

impl Nft {
    /// Creates a new `Nft` with optional data.
    pub fn new(content_type: Option<Vec<u8>>, body: Option<Vec<u8>>) -> Self {
        Self {
            content_type,
            body,
            ..Default::default()
        }
    }

    pub fn append_reveal_script_to_builder(
        &self,
        mut builder: ScriptBuilder,
    ) -> OrdResult<ScriptBuilder> {
        builder = builder
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(constants::PROTOCOL_ID);

        Self::append(
            constants::CONTENT_TYPE_TAG,
            &mut builder,
            &self.content_type,
        );
        Self::append(
            constants::CONTENT_ENCODING_TAG,
            &mut builder,
            &self.content_encoding,
        );
        Self::append(
            constants::METAPROTOCOL_TAG,
            &mut builder,
            &self.metaprotocol,
        );
        Self::append_array(constants::PARENT_TAG, &mut builder, &self.parents);
        Self::append(constants::DELEGATE_TAG, &mut builder, &self.delegate);
        Self::append(constants::POINTER_TAG, &mut builder, &self.pointer);
        Self::append(constants::METADATA_TAG, &mut builder, &self.metadata);
        Self::append(constants::RUNE_TAG, &mut builder, &self.rune);

        if let Some(body) = &self.body {
            builder = builder.push_slice(constants::BODY_TAG);
            for chunk in body.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
                builder = builder.push_slice::<&PushBytes>(chunk.try_into().unwrap());
            }
        }

        Ok(builder.push_opcode(opcodes::all::OP_ENDIF))
    }

    fn append(tag: [u8; 1], builder: &mut ScriptBuilder, value: &Option<Vec<u8>>) {
        if let Some(value) = value {
            let mut tmp = ScriptBuilder::new();
            mem::swap(&mut tmp, builder);

            if is_chunked(tag) {
                for chunk in value.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
                    tmp = tmp
                        .push_slice::<&PushBytes>(tag.as_slice().try_into().unwrap())
                        .push_slice::<&PushBytes>(chunk.try_into().unwrap());
                }
            } else {
                tmp = tmp
                    .push_slice::<&PushBytes>(tag.as_slice().try_into().unwrap())
                    .push_slice::<&PushBytes>(value.as_slice().try_into().unwrap());
            }

            mem::swap(&mut tmp, builder);
        }
    }

    fn append_array(tag: [u8; 1], builder: &mut ScriptBuilder, values: &Vec<Vec<u8>>) {
        let mut tmp = ScriptBuilder::new();
        mem::swap(&mut tmp, builder);

        for value in values {
            tmp = tmp
                .push_slice::<&PushBytes>(tag.as_slice().try_into().unwrap())
                .push_slice::<&PushBytes>(value.as_slice().try_into().unwrap());
        }

        mem::swap(&mut tmp, builder);
    }

    /// Validates the NFT's content type.
    fn validate_content_type(&self) -> OrdResult<Self> {
        if let Some(content_type) = &self.content_type {
            let content_type_str =
                std::str::from_utf8(content_type).map_err(OrdError::Utf8Encoding)?;

            if !content_type_str.contains('/') {
                return Err(OrdError::InscriptionParser(
                    InscriptionParseError::ContentType,
                ));
            }
        }

        Ok(self.clone())
    }

    /// Creates a new `Nft` from JSON-encoded string.
    pub fn from_json_str(data: &str) -> OrdResult<Self> {
        Self::from_str(data)?.validate_content_type()
    }

    /// Returns `Self` as a JSON-encoded data to be pushed to the redeem script.
    pub fn as_push_bytes(&self) -> OrdResult<PushBytesBuf> {
        bytes_to_push_bytes(self.encode()?.as_bytes())
    }

    pub fn body(&self) -> Option<&str> {
        std::str::from_utf8(self.body.as_ref()?).ok()
    }

    pub fn content_type(&self) -> Option<&str> {
        std::str::from_utf8(self.content_type.as_ref()?).ok()
    }

    pub fn metadata(&self) -> Option<ciborium::Value> {
        ciborium::from_reader(Cursor::new(self.metadata.as_ref()?)).ok()
    }

    pub fn reveal_script_as_scriptbuf(&self, builder: ScriptBuilder) -> OrdResult<ScriptBuf> {
        Ok(self.append_reveal_script_to_builder(builder)?.into_script())
    }
}

impl FromStr for Nft {
    type Err = OrdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(OrdError::from)
    }
}

impl Inscription for Nft {
    fn generate_redeem_script(
        &self,
        builder: ScriptBuilder,
        pubkey: RedeemScriptPubkey,
    ) -> OrdResult<ScriptBuilder> {
        let encoded_pubkey = pubkey.encode()?;

        let builder = builder
            .push_slice(encoded_pubkey.as_push_bytes())
            .push_opcode(OP_CHECKSIG);

        self.append_reveal_script_to_builder(builder)
    }

    fn content_type(&self) -> String {
        match self.content_type() {
            Some(t) => t.to_string(),
            None => "".to_string(),
        }
    }

    fn data(&self) -> OrdResult<PushBytesBuf> {
        bytes_to_push_bytes(self.encode()?.as_bytes())
    }
}

fn is_chunked(tag: [u8; 1]) -> bool {
    matches!(tag, constants::METADATA_TAG)
}

#[allow(unused)]
pub(crate) fn create_nft(content_type: &str, body: impl AsRef<[u8]>) -> Nft {
    Nft::new(Some(content_type.into()), Some(body.as_ref().into()))
}
