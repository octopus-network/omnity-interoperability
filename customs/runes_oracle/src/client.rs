use bitcoin_customs::{
    queries::GetGenTicketReqsArgs,
    state::{GenTicketRequest, RunesBalance},
    updates::update_runes_balance::{UpdateRunesBalanceArgs, UpdateRunesBalanceError},
};
use candid::{Decode, Encode};
use ic_agent::{export::Principal, identity::Secp256k1Identity, Agent};
use ic_btc_interface::Txid;
use omnity_hub::self_help::{AddRunesTokenReq, FinalizeAddRunesArgs, SelfServiceError};

pub struct Client {
    agent: Agent,
}

impl Client {
    pub async fn new(url: String, identity: Secp256k1Identity) -> Self {
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
        Self { agent }
    }

    pub async fn get_pending_gen_ticket_requests(
        &self,
        canister_id: &Principal,
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
            .query(canister_id, "get_pending_gen_ticket_requests")
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
        canister_id: &Principal,
        txid: Txid,
        balances: Vec<RunesBalance>,
    ) -> Result<Result<(), UpdateRunesBalanceError>, String> {
        let arg =
            Encode!(&UpdateRunesBalanceArgs { txid, balances }).expect("failed to encode args");

        let response = self
            .agent
            .update(canister_id, "update_runes_balance")
            .with_arg(arg)
            .call_and_wait()
            .await
            .map_err(|err| err.to_string())?;

        let result = Decode!(response.as_slice(), Result<(), UpdateRunesBalanceError>)
            .map_err(|err| err.to_string())?;

        Ok(result)
    }

    pub async fn get_add_runes_token_requests(
        &self,
        canister_id: &Principal,
    ) -> Result<Vec<AddRunesTokenReq>, String> {
        let response = self
            .agent
            .query(canister_id, "get_add_runes_token_requests")
            .with_arg(Encode!().unwrap())
            .call()
            .await
            .map_err(|err| err.to_string())?;

        let result =
            Decode!(response.as_slice(), Vec<AddRunesTokenReq>).map_err(|err| err.to_string())?;
        Ok(result)
    }

    pub async fn finalize_add_runes_token_req(
        &self,
        canister_id: &Principal,
        args: FinalizeAddRunesArgs,
    ) -> Result<Result<(), SelfServiceError>, String> {
        let arg = Encode!(&args).expect("failed to encode args");

        let response = self
            .agent
            .update(canister_id, "finalize_add_runes_token_req")
            .with_arg(arg)
            .call_and_wait()
            .await
            .map_err(|err| err.to_string())?;

        let result = Decode!(response.as_slice(), Result<(), SelfServiceError>)
            .map_err(|err| err.to_string())?;

        Ok(result)
    }
}
