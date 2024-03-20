use crate::*;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub hub_address: Principal,

}
