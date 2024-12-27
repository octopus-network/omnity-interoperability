pub use bitcoin;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

pub use error::{InscriptionParseError, OrdError};
pub use inscription::brc20::Brc20;
pub use inscription::iid::InscriptionId;
pub use inscription::Inscription;
pub use inscription::nft::Nft;
pub use result::OrdResult;
pub use utils::{constants, push_bytes};
pub use utils::fees::{self, MultisigConfig};
pub use wallet::{CreateCommitTransaction, OrdParser,
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

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EtchingArgs {
    pub rune_name: String,
    pub divisibility: Option<u8>,
    pub amount: u128,
    pub cap: u128,
    pub bridge_logo_url: String,
    pub premine: Option<u128>,
    pub logo: Option<LogoParams>,
}


#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct InternalEtchingArgs {
    pub rune_name: String,
    pub divisibility: Option<u8>,
    pub amount: u128,
    pub cap: u128,
    pub bridge_logo_url: String,
    pub premine: Option<u128>,
    pub premine_receiver_principal: String,
    pub logo: Option<LogoParams>,
    pub token_id: String,
    pub target_chain_id: String,
}

impl Into<InternalEtchingArgs> for (EtchingArgs, Principal) {
    fn into(self) -> InternalEtchingArgs {
        let (args, receiver) = self;
        let token_id = format!("Bitcoin-runes-{}", args.rune_name.clone());
        InternalEtchingArgs {
            rune_name: args.rune_name,
            divisibility: args.divisibility,
            amount: args.amount,
            cap: args.cap,
            bridge_logo_url: args.bridge_logo_url,
            premine: args.premine,
            premine_receiver_principal: receiver.to_text(),
            logo: args.logo,
            token_id,
            target_chain_id: "eICP".to_string(),
        }
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LogoParams {
    pub content_type: String,
    pub content_base64: String,
}