use bitcoin::{Amount, Network};
use serde::{Deserialize, Serialize};

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


pub fn calc_fees(network: Network) -> Fees {

    match network {
        Network::Bitcoin => Fees {
            commit_fee: Amount::from_sat(1000),
            reveal_fee: Amount::from_sat(1000),
            utxo_fee: Amount::from_sat(10_000),
        },
        Network::Testnet | Network::Regtest | Network::Signet => Fees {
            commit_fee: Amount::from_sat(2_500),
            reveal_fee: Amount::from_sat(4_700),
            utxo_fee: Amount::from_sat(3_000),
        },
        _ => panic!("unknown network"),
    }
}
