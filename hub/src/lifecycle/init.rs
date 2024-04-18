use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::state::{set_state, HubState};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {}

pub fn init(args: InitArgs) {
    let state = HubState::from(args);
    set_state(state);
}
