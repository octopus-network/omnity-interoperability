pub mod constants;
pub mod envelope;
pub mod push_bytes;

use crate::ord::builder::RedeemScriptPubkey;
use crate::ord::inscription::brc20::Brc20;
use crate::ord::inscription::iid::InscriptionId;
use crate::ord::inscription::nft::Nft;
use crate::ord::inscription::Inscription;
use crate::ord::result::{InscriptionParseError, OrdError, OrdResult};
use bitcoin::script::{Builder as ScriptBuilder, PushBytesBuf};
use bitcoin::Transaction;
pub use constants::*;
use serde::{Deserialize, Serialize};

use self::envelope::ParsedEnvelope;

/// Encapsulates inscription parsing logic for both Ordinals and BRC20s.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum OrdParser {
    /// Denotes a parsed [Nft] inscription.
    Ordinal(Nft),
    /// Denotes a parsed [Brc20] inscription.
    Brc20(Brc20),
}

impl OrdParser {
    /// Parses all inscriptions from a given transaction and categorizes them as either `Self::Brc20` or `Self::Ordinal`.
    ///
    /// This function extracts all inscription data from the transaction, attempts to parse each inscription,
    /// and returns a vector of categorized inscriptions with their corresponding IDs.
    ///
    /// # Errors
    ///
    /// Will return an error if any inscription data cannot be parsed correctly,
    /// or if no valid inscriptions are found in the transaction.
    pub fn parse_all(tx: &Transaction) -> OrdResult<Vec<(InscriptionId, Self)>> {
        let txid = tx.txid();

        ParsedEnvelope::from_transaction(tx)
            .into_iter()
            .map(|envelope| {
                let inscription_id = InscriptionId {
                    txid,
                    index: envelope.input,
                };

                let raw_body = envelope.payload.body.as_ref().ok_or_else(|| {
                    OrdError::InscriptionParser(InscriptionParseError::ParsedEnvelope(
                        "Empty payload body in envelope".to_string(),
                    ))
                })?;

                if let Some(brc20) = Self::parse_brc20(raw_body) {
                    Ok((inscription_id, Self::Brc20(brc20)))
                } else {
                    Ok((inscription_id, Self::Ordinal(envelope.payload)))
                }
            })
            .collect::<Result<Vec<(InscriptionId, Self)>, OrdError>>()
    }

    /// Parses a single inscription from a transaction at a specified index, returning the
    /// parsed inscription along with its ID.
    ///
    /// This method specifically targets one inscription identified by its index within the transaction's inputs.
    /// It extracts the inscription data, attempts to parse it, and categorizes it as either `Self::Brc20` or `Self::Ordinal`.
    ///
    /// # Errors
    ///
    /// Returns an error if the inscription data at the specified index cannot be parsed,
    /// if there is no data at the specified index, or if the data at the index does not contain a valid payload.
    pub fn parse_one(tx: &Transaction, index: usize) -> OrdResult<(InscriptionId, Self)> {
        let envelope = ParsedEnvelope::from_transaction_input(tx, index).ok_or_else(|| {
            OrdError::InscriptionParser(InscriptionParseError::ParsedEnvelope(
                "No data found in envelope at specified index".to_string(),
            ))
        })?;

        let raw_body = envelope.payload.body.as_ref().ok_or_else(|| {
            OrdError::InscriptionParser(InscriptionParseError::ParsedEnvelope(
                "Empty payload body in envelope".to_string(),
            ))
        })?;

        let inscription_id = InscriptionId {
            txid: tx.txid(),
            index: envelope.input,
        };

        if let Some(brc20) = Self::parse_brc20(raw_body) {
            Ok((inscription_id, Self::Brc20(brc20)))
        } else {
            Ok((inscription_id, Self::Ordinal(envelope.payload)))
        }
    }

    /// Attempts to parse the raw data as a BRC20 inscription.
    /// Returns `Some(Brc20)` if successful, otherwise `None`.
    fn parse_brc20(raw_body: &[u8]) -> Option<Brc20> {
        serde_json::from_slice::<Brc20>(raw_body).ok()
    }
}

impl From<Brc20> for OrdParser {
    fn from(inscription: Brc20) -> Self {
        Self::Brc20(inscription)
    }
}

impl From<Nft> for OrdParser {
    fn from(inscription: Nft) -> Self {
        Self::Ordinal(inscription)
    }
}

impl TryFrom<OrdParser> for Nft {
    type Error = OrdError;

    fn try_from(parser: OrdParser) -> Result<Self, Self::Error> {
        match parser {
            OrdParser::Ordinal(nft) => Ok(nft),
            _ => Err(OrdError::InscriptionParser(
                InscriptionParseError::NotOrdinal,
            )),
        }
    }
}

impl TryFrom<&OrdParser> for Nft {
    type Error = OrdError;

    fn try_from(parser: &OrdParser) -> Result<Self, Self::Error> {
        match parser {
            OrdParser::Ordinal(nft) => Ok(nft.clone()),
            _ => Err(OrdError::InscriptionParser(
                InscriptionParseError::NotOrdinal,
            )),
        }
    }
}

impl TryFrom<OrdParser> for Brc20 {
    type Error = OrdError;

    fn try_from(parser: OrdParser) -> Result<Self, Self::Error> {
        match parser {
            OrdParser::Brc20(brc20) => Ok(brc20),
            _ => Err(OrdError::InscriptionParser(InscriptionParseError::NotBrc20)),
        }
    }
}

impl TryFrom<&OrdParser> for Brc20 {
    type Error = OrdError;

    fn try_from(parser: &OrdParser) -> Result<Self, Self::Error> {
        match parser {
            OrdParser::Brc20(brc20) => Ok(brc20.clone()),
            _ => Err(OrdError::InscriptionParser(InscriptionParseError::NotBrc20)),
        }
    }
}

impl Inscription for OrdParser {
    fn content_type(&self) -> String {
        match self {
            Self::Brc20(inscription) => inscription.content_type(),
            Self::Ordinal(inscription) => Inscription::content_type(inscription),
        }
    }

    fn data(&self) -> OrdResult<PushBytesBuf> {
        match self {
            Self::Brc20(inscription) => inscription.data(),
            Self::Ordinal(inscription) => inscription.data(),
        }
    }

    fn generate_redeem_script(
        &self,
        builder: ScriptBuilder,
        pubkey: RedeemScriptPubkey,
    ) -> OrdResult<ScriptBuilder> {
        match self {
            Self::Brc20(inscription) => inscription.generate_redeem_script(builder, pubkey),
            Self::Ordinal(inscription) => inscription.generate_redeem_script(builder, pubkey),
        }
    }
}
