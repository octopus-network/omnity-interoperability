use anyhow::anyhow;
use base64::Engine;
pub use bitcoin;
use candid::{CandidType, Deserialize, Principal};
use ordinals::{Etching, SpacedRune};
use serde::Serialize;
use std::str::FromStr;

use crate::runes_etching::fee_calculator::MAX_LOGO_CONTENT_SIZE;
pub use error::{InscriptionParseError, OrdError};
pub use inscription::iid::InscriptionId;
pub use inscription::nft::Nft;
pub use inscription::Inscription;
pub use result::OrdResult;
pub use utils::fees::{self, MultisigConfig};
pub use utils::{constants, push_bytes};
pub use wallet::{
    CreateCommitTransaction, OrdTransactionBuilder, RevealTransactionArgs,
    SignCommitTransactionArgs, Utxo, Wallet,
};

pub mod error;
pub mod fee_calculator;
pub mod icp_swap;
pub mod inscription;
pub mod result;
pub mod sync;
pub mod transactions;
pub mod utils;
pub mod wallet;

#[derive(CandidType, Clone, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EtchingArgs {
    pub rune_name: String,
    pub divisibility: Option<u8>,
    pub premine: Option<u128>,
    pub logo: Option<LogoParams>,
    pub symbol: Option<String>,
    pub terms: Option<OrdinalsTerms>,
    turbo: bool,
}

#[derive(Default, CandidType, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct OrdinalsTerms {
    pub amount: u128,
    pub cap: u128,
    pub height: (Option<u64>, Option<u64>),
    pub offset: (Option<u64>, Option<u64>),
}

impl OrdinalsTerms {
    pub fn check(&self) -> anyhow::Result<()> {
        if self.amount == 0 || self.cap == 0 {
            return Err(anyhow!("cap or amt is none".to_string()));
        }
        Ok(())
    }
}

impl EtchingArgs {
    pub fn check(&self) -> anyhow::Result<()> {
        if let Some(t) = self.terms {
            t.check()?;
        }
        if let Some(d) = self.divisibility {
            if d > Etching::MAX_DIVISIBILITY {
                return Err(anyhow!("the max divisibility is 38"));
            }
        }
        if let Some(l) = self.logo.clone() {
            let logo_content = base64::engine::general_purpose::STANDARD
                .decode(l.content_base64)
                .map_err(|e| anyhow!(e.to_string()))?;
            if logo_content.len() > MAX_LOGO_CONTENT_SIZE {
                return Err(anyhow!("the max size of logo content is 128k".to_string()));
            }
        }
        let space_rune =
            SpacedRune::from_str(self.rune_name.as_str()).map_err(|e| anyhow!(e.to_string()))?;
        let name = space_rune.rune.to_string();
        if name.len() < 10 || name.len() > 26 {
            return Err(anyhow!("rune name's length must be >= 10 and <=26"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct InternalEtchingArgs {
    pub rune_name: String,
    pub divisibility: Option<u8>,
    pub premine: Option<u128>,
    pub premine_receiver_principal: String,
    pub logo: Option<LogoParams>,
    pub token_id: String,
    pub target_chain_id: String,
    pub symbol: Option<String>,
    pub terms: Option<OrdinalsTerms>,
    pub turbo: bool,
}

impl Into<EtchingArgs> for InternalEtchingArgs {
    fn into(self) -> EtchingArgs {
        EtchingArgs {
            rune_name: self.rune_name,
            divisibility: self.divisibility,
            premine: self.premine,
            logo: self.logo,
            symbol: self.symbol,
            terms: self.terms,
            turbo: self.turbo,
        }
    }
}
impl Into<InternalEtchingArgs> for (EtchingArgs, Principal) {
    fn into(self) -> InternalEtchingArgs {
        let (args, receiver) = self;
        let token_id = format!("Bitcoin-runes-{}", args.rune_name.clone());
        InternalEtchingArgs {
            rune_name: args.rune_name,
            divisibility: args.divisibility,
            premine: args.premine,
            premine_receiver_principal: receiver.to_text(),
            logo: args.logo,
            token_id,
            target_chain_id: "eICP".to_string(),
            symbol: args.symbol,
            terms: args.terms,
            turbo: args.turbo,
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LogoParams {
    pub content_type: String,
    pub content_base64: String,
}
