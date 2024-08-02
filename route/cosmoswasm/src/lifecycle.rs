pub mod init;
pub mod upgrade;

use crate::*;

#[derive(CandidType, Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChainState {
    #[default]
    Active,
    Deactive,
}