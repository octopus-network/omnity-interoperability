use crate::*;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub ckbtc_index_principal: Principal,
    pub icp_customs_principal: Principal,
    pub trigger: Principal,
}

pub fn init(args: InitArgs) {
    set_ckbtc_index_principal(args.ckbtc_index_principal);
    set_icp_customs_principal(args.icp_customs_principal);
    state::set_trigger_principal(args.trigger);
}
