use bitcoin_customs::{
    queries::GetGenTicketReqsArgs,
    state::{GenTicketRequest, RunesBalance},
    updates::update_runes_balance::{UpdateRunesBalanceArgs, UpdateRunesBalanceError},
};
use candid::{Decode, Encode};
use ic_agent::{export::Principal, identity::Secp256k1Identity, Agent};
use ic_btc_interface::Txid;

pub struct Customs {
    agent: Agent,
    canister_id: Principal,
}

impl Customs {
    pub async fn new(url: String, canister_id: Principal, identity: Secp256k1Identity) -> Self {
        let agent = Agent::builder()
            .with_url(&url)
            .with_identity(identity)
            .build()
            .expect("failed to build agent");
        if url.starts_with("http://") {
            agent
                .fetch_root_key()
                .await
                .expect("failed to fetch root key");
        }
        Self { agent, canister_id }
    }

    pub async fn get_pending_gen_ticket_requests(
        &self,
        start_txid: Option<Txid>,
        max_count: u64,
    ) -> Result<Vec<GenTicketRequest>, String> {
        let arg = Encode!(&GetGenTicketReqsArgs {
            start_txid,
            max_count,
        })
        .expect("failed to encode args");
        let response = self
            .agent
            .query(&self.canister_id, "get_pending_gen_ticket_requests")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| err.to_string())?;

        let result =
            Decode!(response.as_slice(), Vec<GenTicketRequest>).map_err(|err| err.to_string())?;
        Ok(result)
    }

    pub async fn update_runes_balance(
        &self,
        txid: Txid,
        balances: Vec<RunesBalance>,
    ) -> Result<Result<(), UpdateRunesBalanceError>, String> {
        let arg =
            Encode!(&UpdateRunesBalanceArgs { txid, balances }).expect("failed to encode args");

        let response = self
            .agent
            .update(&self.canister_id, "update_runes_balance")
            .with_arg(arg)
            .call_and_wait()
            .await
            .map_err(|err| err.to_string())?;

        let result = Decode!(response.as_slice(), Result<(), UpdateRunesBalanceError>)
            .map_err(|err| err.to_string())?;

        Ok(result)
    }
}
