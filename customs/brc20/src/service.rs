use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

use crate::bitcoin_to_custom::finalize_generate_ticket_request;
use crate::custom_to_bitcoin::test_send_ticket;
use crate::generate_ticket::{GenerateTicketArgs, GenerateTicketError};
use omnity_types::TxAction::Redeem;
use omnity_types::{Network, Ticket, TicketType, TxAction};

use crate::state::{
    init_ecdsa_public_key, mutate_state, read_state, replace_state, Brc20State, StateProfile,
};

#[init]
fn init(args: InitArgs) {
    replace_state(Brc20State::init(args).expect("params error"));
}

#[pre_upgrade]
fn pre_upgrade() {
    read_state(|s| s.pre_upgrade());
}

#[post_upgrade]
fn post_upgrade() {
    Brc20State::post_upgrade();
}

#[update]
pub async fn generate_ticket(req: GenerateTicketArgs) {
    crate::generate_ticket::generate_ticket(req).await.unwrap();
}
#[update]
pub async fn generate_deposit_addr() -> (Option<String>, Option<String>) {
    init_ecdsa_public_key().await;
    read_state(|s| (s.deposit_addr.clone(), s.deposit_pubkey.clone()))
}

#[update]
pub async fn finalize_gen() {
    finalize_generate_ticket_request().await;
}

#[query]
pub fn brc20_state() -> StateProfile {
    read_state(|s| StateProfile::from(s))
}

#[update]
pub async fn test_create_tx() -> String {
    let ticket = Ticket {
        ticket_id: "sfisdiasddssfsdf".to_string(),
        ticket_type: TicketType::Normal,
        ticket_time: 0,
        src_chain: "Bitlayer".to_string(),
        dst_chain: "brc20".to_string(),
        action: TxAction::Redeem,
        token: "nbcs".to_string(),
        amount: "1000000".to_string(),
        sender: None,
        receiver: "tb1qyelgkxpfhfjrg6hg8hlr9t4dzn7n88eacqjh0t".to_string(),
        memo: None,
    };
    let r = test_send_ticket(ticket).await.unwrap();
    serde_json::to_string(&r).unwrap()
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub admins: Vec<Principal>,
    pub hub_principal: Principal,
    pub network: Network,
    pub chain_id: String,
    pub indexer_principal: Principal,
}

ic_cdk::export_candid!();
