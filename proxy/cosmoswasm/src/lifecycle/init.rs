use crate::*;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub ckbtc_index_principal: Principal,
    pub icp_custom_principal: Principal,
}

pub fn init(args: InitArgs) {
    set_ckbtc_index_principal(args.ckbtc_index_principal);
    set_icp_custom_principal(args.icp_custom_principal);
}
