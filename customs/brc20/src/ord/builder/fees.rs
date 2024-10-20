use bitcoin::Amount;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Fees {
    pub commit_fee: Amount,
    pub reveal_fee: Amount,
    pub spend_fee: Amount,
}

impl Fees {
    pub fn sum(&self) -> u64 {
        self.commit_fee.to_sat() + self.reveal_fee.to_sat() + self.spend_fee.to_sat()
    }
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
