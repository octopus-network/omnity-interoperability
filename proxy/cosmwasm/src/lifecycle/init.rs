use state::Settings;

use crate::*;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub ckbtc_ledger_principal: Principal,
    pub ckbtc_minter_principal: Principal,
    pub icp_customs_principal: Principal,
}

pub fn init(args: InitArgs) {
    state::set_settings(Settings {
        ckbtc_ledger_principal: args.ckbtc_ledger_principal,
        ckbtc_minter_principal: args.ckbtc_minter_principal,
        icp_customs_principal: args.icp_customs_principal,
    });
}
