
use crate::ic_sui::sui_types::object::Owner;
use crate::ic_sui::move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::fmt::{Display, Formatter, Result};
use crate::ic_sui::sui_types::sui_serde::SuiTypeTag;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BalanceChange {
    /// Owner of the balance change
    pub owner: Owner,
    #[serde_as(as = "SuiTypeTag")]
    pub coin_type: TypeTag,
    /// The amount indicate the balance value changes,
    /// negative amount means spending coin value and positive means receiving coin value.
    #[serde_as(as = "DisplayFromStr")]
    pub amount: i128,
}

impl Display for BalanceChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            " ┌──\n │ Owner: {} \n │ CoinType: {} \n │ Amount: {}\n └──",
            self.owner, self.coin_type, self.amount
        )
    }
}
