use bitcoin_customs::{
    state::{GenTicketRequest, RunesBalance, RunesId},
    updates::update_runes_balance::{UpdateRunesBalanceError, UpdateRunesBlanceArgs},
};
use candid::{Decode, Encode};
use ic_agent::{export::Principal, identity::AnonymousIdentity, Agent};
use ic_btc_interface::Txid;

pub struct Customs {
    agent: Agent,
    canister_id: Principal,
}

impl Customs {
    pub fn new(url: String, canister_id: Principal) -> Self {
        let agent = Agent::builder()
            .with_url(url)
            .with_identity(AnonymousIdentity)
            .build()
            .expect("failed to build agent");
        Self { agent, canister_id }
    }

    pub async fn get_pending_gen_ticket_requets(&self) -> Result<Vec<GenTicketRequest>, String> {
        let response = self
            .agent
            .query(&self.canister_id, "get_pending_gen_ticket_requets")
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
        vout: u32,
        runes_id: RunesId,
        value: u128,
    ) -> Result<Result<(), UpdateRunesBalanceError>, String> {
        let arg = Encode!(&UpdateRunesBlanceArgs {
            txid,
            vout,
            balance: RunesBalance { runes_id, value }
        })
        .expect("failed to encode args");

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
