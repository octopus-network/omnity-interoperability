use super::get_sol_address::{get_sol_address, GetSolAddressArgs};
use crate::{
    address::main_address_path,
    hub,
    solana_rpc::{self, init_solana_client, query_transaction},
    state::{mutate_state, read_state, CollectionTx},
    transaction::{ParsedIns, ParsedValue, Transaction, Transfer},
    types::omnity_types::{ChainId, ChainState, Ticket, TicketType, TokenId, TxAction},
};
use candid::{CandidType, Deserialize};
use ic_canister_log::log;
use ic_solana::{ic_log::ERROR, token::constants::system_program_id};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use serde_json::from_value;
use std::borrow::Cow;

const SOL_TOKEN: &str = "SOL";

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketArgs {
    pub target_chain_id: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u64,
    pub signature: String,
}

impl Storable for GenerateTicketArgs {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let cm =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode ReleaseTokenReq");
        cm
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    SendTicketErr(String),
    AmountIsZero,
    RpcError(String),
    MismatchWithGenTicketReq,
    UnsupportedChainId(ChainId),
    UnsupportedToken(TokenId),
    AlreadyProcessed,
}

pub async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    if !read_state(|s| {
        s.counterparties
            .get(&args.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            args.target_chain_id.clone(),
        ));
    }

    if !read_state(|s| s.tokens.contains_key(&args.token_id)) {
        return Err(GenerateTicketError::UnsupportedToken(args.token_id));
    };

    if args.amount == 0 {
        return Err(GenerateTicketError::AmountIsZero);
    }

    if read_state(|s| s.finalized_gen_tickets.contains_key(&args.signature)) {
        return Err(GenerateTicketError::AlreadyProcessed);
    }

    let get_address_args = GetSolAddressArgs {
        target_chain_id: args.target_chain_id.clone(),
        receiver: args.receiver.clone(),
    };
    let address = get_sol_address(get_address_args.clone()).await;

    let tx = match query_transaction(args.signature.clone()).await {
        Ok(transaction) => transaction,
        Err(err) => return Err(GenerateTicketError::RpcError(err)),
    };

    let (transfer_amount, sender) = get_transfer_amount(tx, address.to_string())?;
    if transfer_amount != args.amount {
        return Err(GenerateTicketError::MismatchWithGenTicketReq);
    }

    let (chain_id, hub_principal) = read_state(|s| (s.chain_id.clone(), s.hub_principal));
    hub::send_ticket(
        hub_principal,
        Ticket {
            ticket_id: args.signature.clone(),
            ticket_type: TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id,
            dst_chain: args.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: SOL_TOKEN.into(),
            amount: args.amount.to_string(),
            sender: Some(sender),
            receiver: args.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;

    let mut collection_tx = CollectionTx {
        source_signature: args.signature.clone(),
        from: address,
        from_path: get_address_args.to_derivation_path(),
        amount: args.amount,
        signature: None,
        last_sent_at: 0,
        try_cnt: 0,
    };
    if let Err(err) = send_collection_tx(&mut collection_tx).await {
        log!(ERROR, "failed to send collection tx: {}", err);
    }

    mutate_state(|s| {
        s.finalized_gen_tickets.insert(args.signature.clone(), args);
        s.submitted_collection_txs.insert(
            collection_tx.source_signature.clone(),
            collection_tx.clone(),
        );
    });
    Ok(())
}

pub async fn send_collection_tx(args: &mut CollectionTx) -> Result<(), String> {
    let sol_client = init_solana_client().await;
    let main_address = solana_rpc::ecdsa_public_key(main_address_path()).await;
    args.last_sent_at = ic_cdk::api::time();
    args.try_cnt += 1;

    match sol_client
        .transfer(args.from, args.from_path.clone(), main_address, args.amount)
        .await
    {
        Ok(signature) => {
            args.signature = Some(signature);
            Ok(())
        }
        Err(err) => Err(err.to_string()),
    }
}

fn get_transfer_amount(
    tx: Transaction,
    dest: String,
) -> Result<(u64, String), GenerateTicketError> {
    let mut amount: u64 = 0;
    let mut sender = String::default();
    for inst in tx.message.instructions {
        if let Ok(parsed_value) = from_value::<ParsedValue>(inst.parsed.to_owned().unwrap()) {
            if let Ok(pi) = from_value::<ParsedIns>(parsed_value.parsed.to_owned()) {
                if pi.instr_type == "transfer" {
                    let transfer = from_value::<Transfer>(pi.info.to_owned())
                        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;

                    if inst.program_id == system_program_id().to_string()
                        && transfer.destination == dest
                    {
                        sender = transfer.source;
                        amount += transfer.lamports;
                    }
                }
            }
        }
    }
    Ok((amount, sender))
}
