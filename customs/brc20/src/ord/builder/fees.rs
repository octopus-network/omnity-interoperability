use bitcoin::{Amount, Network};
use serde::{Deserialize, Serialize};

use crate::constants::{COMMIT_TX_VBYTES, DEFAULT_FEE, REVEAL_TX_VBYTES, TRANSFER_TX_VBYTES};
use crate::custom_to_bitcoin::estimate_fee_per_vbyte;
use crate::ord::parser::POSTAGE;

#[allow(dead_code)]
pub struct Fees {
    pub commit_fee: Amount,
    pub reveal_fee: Amount,
    pub utxo_fee: Amount,
}

/// Represents multisig configuration (m of n) for a transaction, if applicable.
/// Encapsulates the number of required signatures and the total number of signatories.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultisigConfig {
    /// Number of required signatures (m)
    pub required: usize,
    /// Total number of signatories (n)
    pub total: usize,
}

pub async fn calc_fees(network: Network) -> Fees {

    match network {
        Network::Bitcoin => {
            let r = estimate_fee_per_vbyte().await;
            match r {
                None => {
                    DEFAULT_FEE
                }
                Some(v_price) => {
                    Fees {
                        commit_fee: Amount::from_sat(COMMIT_TX_VBYTES*v_price/1000),
                        reveal_fee: Amount::from_sat(REVEAL_TX_VBYTES*v_price/1000),
                        utxo_fee: Amount::from_sat(TRANSFER_TX_VBYTES*v_price/1000 + POSTAGE),
                    }
                }
            }
        }

        Network::Testnet | Network::Regtest | Network::Signet => Fees {
            commit_fee: Amount::from_sat(2_500),
            reveal_fee: Amount::from_sat(4_700),
            utxo_fee: Amount::from_sat(3_000),
        },
        _ => panic!("unknown network"),
    }
}
