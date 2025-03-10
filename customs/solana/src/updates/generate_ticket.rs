use crate::{
    hub,
    port_native::{
        instruction::{InstSerialize, Transport},
        port_address, vault_address, ParsedValue,
    },
    solana_rpc::query_transaction,
    state::{mutate_state, read_state},
    transaction::Transaction,
    types::omnity_types::{ChainId, ChainState, Ticket, TicketType, TokenId, TxAction},
};
use borsh::BorshDeserialize;
use candid::{CandidType, Deserialize};
use ic_canister_log::log;
use ic_solana::ic_log::DEBUG;
use ic_solana::token::constants::system_program_id;
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use serde_json::from_value;
use std::borrow::Cow;

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
    RpcError(String),
    MismatchWithGenTicketReq,
    UnsupportedChainId(ChainId),
    UnsupportedToken(TokenId),
    AlreadyProcessed,
    DecodeTxError(String),
}

pub async fn generate_ticket(args: GenerateTicketArgs) -> Result<(), GenerateTicketError> {
    log!(DEBUG, "[solana-custom] generate_ticket: {:?}", args);
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

    if read_state(|s| s.finalized_gen_tickets.contains_key(&args.signature)) {
        return Err(GenerateTicketError::AlreadyProcessed);
    }

    let tx = match query_transaction(args.signature.clone()).await {
        Ok(transaction) => transaction,
        Err(err) => return Err(GenerateTicketError::RpcError(err)),
    };
    log!(DEBUG, "[solana-custom] query transaction: {:?}", args);

    let transport = parse_transport(tx, &args.target_chain_id, &args.receiver)?;
    if transport.raw.amount != args.amount {
        return Err(GenerateTicketError::MismatchWithGenTicketReq);
    }

    let (chain_id, hub_principal) = read_state(|s| (s.chain_id.clone(), s.hub_principal));
    hub::send_ticket(
        hub_principal,
        Ticket {
            ticket_id: args.signature.clone(),
            ticket_type: TicketType::Normal,
            ticket_time: ic_cdk::api::time(),
            src_chain: chain_id.clone(),
            dst_chain: args.target_chain_id.clone(),
            action: TxAction::Transfer,
            token: args.token_id.clone(),
            amount: args.amount.to_string(),
            sender: Some(transport.sender),
            receiver: args.receiver.clone(),
            memo: None,
        },
    )
    .await
    .map_err(|err| GenerateTicketError::SendTicketErr(format!("{}", err)))?;
    log!(DEBUG, "[solana-custom] send ticket: {:?}", args);

    mutate_state(|s| {
        s.finalized_gen_tickets.insert(args.signature.clone(), args);
    });
    Ok(())
}

struct TransportWithSender {
    raw: Transport,
    sender: String,
}

fn parse_transport(
    tx: Transaction,
    target_chain: &String,
    receiver: &String,
) -> Result<TransportWithSender, GenerateTicketError> {
    let (port, _) = port_address();
    let (vault, _) = vault_address();
    for inst in tx.message.instructions {
        if inst.program_id != read_state(|s| s.port_program_id.to_string()) {
            continue;
        }
        if let Ok(value) = from_value::<ParsedValue>(inst.parsed.to_owned().unwrap()) {
            let accounts = &value.accounts;
            if accounts.len() != 4
                || accounts[0] != port.to_string()
                || accounts[1] != vault.to_string()
                || accounts[3] != system_program_id().to_string()
            {
                continue;
            }
            if value.data[..8] == Transport::discriminator() {
                let transport = Transport::try_from_slice(&value.data[8..])
                    .map_err(|err| GenerateTicketError::DecodeTxError(err.to_string()))?;

                if transport.target_chain.eq(target_chain) && transport.recipient.eq(receiver) {
                    return Ok(TransportWithSender {
                        sender: accounts[2].clone(),
                        raw: transport,
                    });
                }
            }
        }
    }
    Err(GenerateTicketError::MismatchWithGenTicketReq)
}
