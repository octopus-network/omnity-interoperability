use std::str::FromStr;
use anyhow::anyhow;
pub use bitcoin;
use candid::{CandidType, Deserialize, Principal};
use ordinals::SpacedRune;
use serde::Serialize;

pub use error::{InscriptionParseError, OrdError};
pub use inscription::iid::InscriptionId;
pub use inscription::Inscription;
pub use inscription::nft::Nft;
pub use result::OrdResult;
pub use utils::{constants, push_bytes};
pub use utils::fees::{self, MultisigConfig};
pub use wallet::{CreateCommitTransaction,
                 OrdTransactionBuilder, RevealTransactionArgs, SignCommitTransactionArgs, Utxo, Wallet,
};

pub mod error;
pub mod inscription;
pub mod result;
pub mod utils;
pub mod wallet;
pub mod transactions;
mod fee_calculator;
pub mod sync;
pub mod icp_swap;

#[derive(CandidType, Clone,Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EtchingArgs {
    pub rune_name: String,
    pub divisibility: Option<u8>,
    pub premine: Option<u128>,
    pub logo: Option<LogoParams>,
    pub symbol: Option<String>,
    pub terms: Option<OrdinalsTerms>,
    turbo: bool
}

#[derive(Default, CandidType, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct OrdinalsTerms {
    pub amount: Option<u128>,
    pub cap: Option<u128>,
    pub height: (Option<u64>, Option<u64>),
    pub offset: (Option<u64>, Option<u64>),
}

impl OrdinalsTerms {
    pub fn check(&self) -> anyhow::Result<()> {
        if self.amount.is_none() ||self.cap.is_none() ||self.amount.clone().unwrap() ==0 || self.cap.clone().unwrap() == 0 {
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
        let space_rune = SpacedRune::from_str(self.rune_name.as_str()).map_err(|e|anyhow!(e.to_string()))?;
        let name = space_rune.rune.to_string();
        if name.len() < 10 {
            return Err(anyhow!("rune name's length must be more than 10"));
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