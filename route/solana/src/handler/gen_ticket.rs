use crate::types::Ticket;
use crate::types::{ChainState, Error, TicketType, TxAction};
use candid::{CandidType, Principal};

use crate::handler::solana_rpc::solana_client;
use ic_solana::token::SolanaClient;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::{
    call_error::{CallError, Reason},
    state::{mutate_state, read_state},
};

use ic_canister_log::log;
use ic_solana::ic_log::{DEBUG, ERROR};
use serde_json::from_value;
use serde_json::Value;

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum GenerateTicketError {
    TemporarilyUnavailable(String),
    UnsupportedToken(String),
    UnsupportedChainId(String),
    /// The redeem account does not hold the requested token amount.
    InsufficientFunds {
        balance: u64,
    },
    /// The caller didn't approve enough funds for spending.
    InsufficientAllowance {
        allowance: u64,
    },
    SendTicketErr(String),
    InsufficientRedeemFee {
        required: u64,
        provided: u64,
    },
    RedeemFeeNotSet,
    TransferFailure(String),
    UnsupportedAction(String),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GenerateTicketReq {
    pub signature: String,
    pub target_chain_id: String,
    pub sender: String,
    pub receiver: String,
    pub token_id: String,
    pub amount: u64,
    pub action: TxAction,
    pub memo: Option<String>,
}

impl Storable for GenerateTicketReq {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let tm =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode GenerateTicketReq");
        tm
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GenerateTicketOk {
    pub ticket_id: String,
}

pub async fn generate_ticket(
    req: GenerateTicketReq,
) -> Result<GenerateTicketOk, GenerateTicketError> {
    log!(DEBUG, "[generate_ticket] generate_ticket req: {:#?}", req);

    mutate_state(|s| {
        s.gen_ticket_reqs
            .insert(req.signature.to_owned(), req.to_owned())
    });

    if read_state(|s| s.chain_state == ChainState::Deactive) {
        return Err(GenerateTicketError::TemporarilyUnavailable(
            "chain state is deactive!".into(),
        ));
    }

    if !read_state(|s| {
        s.counterparties
            .get(&req.target_chain_id)
            .is_some_and(|c| c.chain_state == ChainState::Active)
    }) {
        return Err(GenerateTicketError::UnsupportedChainId(
            req.target_chain_id.clone(),
        ));
    }

    if !read_state(|s| s.tokens.contains_key(&req.token_id.to_string())) {
        return Err(GenerateTicketError::UnsupportedToken(req.token_id.clone()));
    }

    if !matches!(req.action, TxAction::Redeem) {
        return Err(GenerateTicketError::UnsupportedAction(
            "[generate_ticket] Transfer action is not supported".into(),
        ));
    }

    let (hub_principal, chain_id) = read_state(|s| (s.hub_principal, s.chain_id.to_owned()));

    if !verify_tx(req.to_owned()).await? {
        return Err(GenerateTicketError::TemporarilyUnavailable(format!(
            "[generate_ticket] Unable to verify the tx ({}) ",
            req.signature,
        )));
    }

    let ticket = Ticket {
        ticket_id: req.signature.to_string(),
        ticket_type: TicketType::Normal,
        ticket_time: ic_cdk::api::time(),
        src_chain: chain_id,
        dst_chain: req.target_chain_id.to_owned(),
        action: req.action.to_owned(),
        token: req.token_id.to_owned(),
        amount: req.amount.to_string(),
        sender: Some(req.sender.to_owned()),
        receiver: req.receiver.to_string(),
        memo: req.memo.to_owned().map(|m| m.to_bytes().to_vec()),
    };

    match send_ticket(hub_principal, ticket.to_owned()).await {
        Err(err) => {
            mutate_state(|s| {
                s.tickets_failed_to_hub
                    .insert(ticket.ticket_id.to_string(), ticket.to_owned());
            });
            log!(
                ERROR,
                "[generate_ticket] failed to send ticket: {}",
                req.signature.to_string()
            );
            Err(GenerateTicketError::SendTicketErr(format!("{}", err)))
        }
        Ok(()) => {
            log!(
                DEBUG,
                "[generate_ticket] successful to send ticket: {:?}",
                ticket
            );
            // remove the req
            mutate_state(|s| s.gen_ticket_reqs.remove(&req.signature.to_owned()));
            Ok(GenerateTicketOk {
                ticket_id: req.signature.to_string(),
            })
        }
    }
}

pub async fn verify_tx(req: GenerateTicketReq) -> Result<bool, GenerateTicketError> {
    // let mut receiver = String::from("");
    // let mut tx = String::from("");
    let client = solana_client().await;
    let multi_rpc_config = read_state(|s| s.multi_rpc_config.to_owned());
    multi_rpc_config
        .check_config_valid()
        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;
    let tx_response = query_tx_from_multi_rpc(
        &client,
        req.signature.to_owned(),
        multi_rpc_config.rpc_list.to_owned(),
    )
    .await;
    log!(
        DEBUG,
        "[generate_ticket] query_tx_from_multi_rpc tx_response: {:?}",
        tx_response
    );

    let instructions = multi_rpc_config
        .valid_and_get_result(&tx_response)
        .map_err(|e| GenerateTicketError::TemporarilyUnavailable(e.to_string()))?;
    log!(
        DEBUG,
        "[generate_ticket] valid_and_get_result result: {:?}",
        instructions
    );

    let mut transfer_ok = false;
    let mut burn_ok = false;
    let mut memo_ok = false;

    // parse instruction
    for instruction in &instructions {
        if let Ok(parsed_value) = from_value::<ParsedValue>(instruction.parsed.to_owned().unwrap())
        {
            if let Ok(pi) = from_value::<ParsedIns>(parsed_value.parsed.to_owned()) {
                log!(DEBUG, "[generate_ticket] Parsed instruction: {:#?}", pi);

                match pi.instr_type.as_str() {
                    "transfer" => {
                        let transfer = from_value::<Transfer>(pi.info.to_owned()).map_err(|e| {
                            GenerateTicketError::TemporarilyUnavailable(e.to_string())
                        })?;
                        log!(DEBUG, "[generate_ticket] Parsed transfer: {:#?}", transfer);
                        let fee = read_state(|s| s.get_fee(req.target_chain_id.clone())).ok_or(
                            GenerateTicketError::TemporarilyUnavailable(format!(
                                "[generate_ticket] No found fee for {}",
                                req.target_chain_id
                            )),
                        )?;
                        let fee_account = read_state(|s| s.fee_account.to_string());
                        let lamports = transfer.lamports as u128;
                        if !(transfer.source.eq(&req.sender)
                            && transfer.destination.eq(&fee_account)
                            && lamports == fee)
                        {
                            return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                                "[generate_ticket] Unable to verify the collect fee info",
                            )));
                        }
                        transfer_ok = true;
                    }
                    "burnChecked" => {
                        let burn_checked =
                            from_value::<BurnChecked>(pi.info.to_owned()).map_err(|e| {
                                GenerateTicketError::TemporarilyUnavailable(e.to_string())
                            })?;
                        log!(
                            DEBUG,
                            "[generate_ticket] Parsed burn_checked: {:#?}",
                            burn_checked
                        );
                        let burned_amount = burn_checked
                            .token_amount
                            .ui_amount_string
                            .parse::<u64>()
                            .map_err(|e| {
                                GenerateTicketError::TemporarilyUnavailable(e.to_string())
                            })?;
                        let mint_address =
                            read_state(|s| s.token_mint_accounts.get(&req.token_id).to_owned())
                                .ok_or(GenerateTicketError::TemporarilyUnavailable(format!(
                                    "[generate_ticket] No found token mint address for {}",
                                    req.token_id
                                )))?;
                        if !(burn_checked.authority.eq(&req.sender)
                            && burn_checked.mint.eq(&mint_address.account)
                            && burned_amount == req.amount)
                        {
                            return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                                "[generate_ticket] Unable to verify the token burned info",
                            )));
                        }
                        burn_ok = true;
                    }
                    "burn" => {
                        let burn = from_value::<Burn>(pi.info.to_owned()).map_err(|e| {
                            GenerateTicketError::TemporarilyUnavailable(e.to_string())
                        })?;
                        log!(DEBUG, "[generate_ticket] Parsed burn: {:#?}", burn);
                        let burned_amount = burn.amount.parse::<u64>().map_err(|e| {
                            GenerateTicketError::TemporarilyUnavailable(e.to_string())
                        })?;
                        let mint_address =
                            read_state(|s| s.token_mint_accounts.get(&req.token_id).to_owned())
                                .ok_or(GenerateTicketError::TemporarilyUnavailable(format!(
                                    "[generate_ticket] No found token mint address for {}",
                                    req.token_id
                                )))?;
                        if !(burn.authority.eq(&req.sender)
                            && burn.mint.eq(&mint_address.account)
                            && burned_amount == req.amount)
                        {
                            return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                                "[generate_ticket] Unable to verify the token burned info",
                            )));
                        }
                        burn_ok = true;
                    }
                    _ => {
                        log!(
                            DEBUG,
                            "[generate_ticket] Skipped non-relevant instruction: {:#?}",
                            pi.instr_type
                        );
                    }
                }
            } else if let Ok(memo) = from_value::<String>(parsed_value.parsed.to_owned()) {
                log!(DEBUG, "[generate_ticket] Parsed memo: {:?}", memo);
                //verify memo.eq(req.receiver.)
                if memo.eq(&req.receiver) {
                    // receiver = memo;
                    memo_ok = true;
                } else {
                    return Err(GenerateTicketError::TemporarilyUnavailable(format!(
                        "[generate_ticket] receiver({}) from memo not match req.receiver({})",
                        memo, req.receiver,
                    )));
                }
            } else {
                log!(
                    DEBUG,
                    "[generate_ticket] Unknown Parsed instruction: {:#?}",
                    parsed_value.parsed
                );
            }
        } else {
            log!(
                DEBUG,
                "[generate_ticket] Unknown Parsed Value: {:#?}",
                instruction.parsed
            );
        }
    }

    Ok(transfer_ok && burn_ok && memo_ok)
    // Ok(burn_ok && memo_ok)
}

/// send ticket to hub
pub async fn send_ticket(hub_principal: Principal, ticket: Ticket) -> Result<(), CallError> {
    let resp: (Result<(), Error>,) =
        ic_cdk::api::call::call(hub_principal, "send_ticket", (ticket,))
            .await
            .map_err(|(code, message)| CallError {
                method: "send_ticket".to_string(),
                reason: Reason::from_reject(code, message),
            })?;
    let data = resp.0.map_err(|err| CallError {
        method: "send_ticket".to_string(),
        reason: Reason::CanisterError(err.to_string()),
    })?;
    Ok(data)
}

pub async fn query_tx_from_multi_rpc(
    client: &SolanaClient,
    signature: String,
    rpc_url_vec: Vec<String>,
) -> Vec<anyhow::Result<String>> {
    let mut fut = Vec::with_capacity(rpc_url_vec.len());
    for rpc_url in rpc_url_vec {
        fut.push(async {
            client
                .query_transaction(signature.to_owned(), Some(rpc_url))
                .await
        });
    }
    futures::future::join_all(fut).await
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetail {
    pub block_time: Option<u64>,
    pub meta: Meta,
    pub slot: u64,
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub compute_units_consumed: u64,
    pub err: Option<Value>,
    pub fee: u64,
    pub inner_instructions: Vec<Value>,
    pub log_messages: Vec<Value>,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<Value>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<Value>,
    pub rewards: Vec<Value>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Status {
    #[serde(rename = "Ok")]
    pub ok: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<AccountKey>,
    pub instructions: Vec<Instruction>,
    pub recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AccountKey {
    pub pubkey: String,
    pub signer: bool,
    pub source: String,
    pub writable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    #[serde(flatten)]
    pub parsed: Option<Value>,
    // #[serde(flatten)]
    pub program: Option<String>,
    pub program_id: String,
    pub stack_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
pub struct ParsedValue {
    pub parsed: Value,
    // pub parsed: InstructionEnum,
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
pub struct ParsedInstruction {
    pub parsed: Value,
    // Memo(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum InstructionEnum {
    Transfer(ParsedIns),
    Burn(ParsedBurnChecked),
    Memo(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParsedIns {
    pub info: Value,
    #[serde(rename = "type")]
    pub instr_type: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Transfer {
    pub destination: String,
    pub lamports: u64,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBurnChecked {
    pub info: BurnChecked,
    #[serde(rename = "type")]
    pub instr_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BurnChecked {
    pub account: String,
    pub authority: String,
    pub mint: String,
    pub token_amount: TokenAmount,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBurn {
    pub info: Burn,
    #[serde(rename = "type")]
    pub instr_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Burn {
    pub account: String,
    pub amount: String,
    pub authority: String,
    pub mint: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f64,
    pub ui_amount_string: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use candid::Principal;
    use ic_solana::rpc_client::JsonRpcResponse;
    use serde_json::from_value;

    #[test]
    fn test_parse_tx_with_transfer_burn_memo() {
        let json_data = r#"
            {
  "jsonrpc": "2.0",
  "result": {
    "blockTime": 1727316269,
    "meta": {
      "computeUnitsConsumed": 25649,
      "err": null,
      "fee": 25000,
      "innerInstructions": [],
      "logMessages": [
        "Program ComputeBudget111111111111111111111111111111 invoke [1]",
        "Program ComputeBudget111111111111111111111111111111 success",
        "Program ComputeBudget111111111111111111111111111111 invoke [1]",
        "Program ComputeBudget111111111111111111111111111111 success",
        "Program 11111111111111111111111111111111 invoke [1]",
        "Program 11111111111111111111111111111111 success",
        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb invoke [1]",
        "Program log: Instruction: Burn",
        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb consumed 1519 of 199550 compute units",
        "Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb success",
        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr invoke [1]",
        "Program log: Memo (len 62): \"bc1p830q5uwpaxpmzaam2t93jgcq55nrs0x2n6xhl70arkzu3gy9w00qwa7pug\"",
        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr consumed 23680 of 198031 compute units",
        "Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr success"
      ],
      "postBalances": [
        163636620,
        401278600,
        2074080,
        3897600,
        1,
        1,
        521498880,
        1141440
      ],
      "postTokenBalances": [
        {
          "accountIndex": 2,
          "mint": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx",
          "owner": "E3dQM443fE4qfF7seeSjkXSkfghbpzCkY2pJqVPnEm26",
          "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
          "uiTokenAmount": {
            "amount": "40000",
            "decimals": 2,
            "uiAmount": 400.0,
            "uiAmountString": "400"
          }
        }
      ],
      "preBalances": [
        178041620,
        386898600,
        2074080,
        3897600,
        1,
        1,
        521498880,
        1141440
      ],
      "preTokenBalances": [
        {
          "accountIndex": 2,
          "mint": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx",
          "owner": "E3dQM443fE4qfF7seeSjkXSkfghbpzCkY2pJqVPnEm26",
          "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
          "uiTokenAmount": {
            "amount": "50000",
            "decimals": 2,
            "uiAmount": 500.0,
            "uiAmountString": "500"
          }
        }
      ],
      "rewards": [],
      "status": {
        "Ok": null
      }
    },
    "slot": 292030024,
    "transaction": {
      "message": {
        "accountKeys": [
          {
            "pubkey": "E3dQM443fE4qfF7seeSjkXSkfghbpzCkY2pJqVPnEm26",
            "signer": true,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
            "signer": false,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "3hntCFiY3a3QFUjcYXnbxc1pp4cMFGEsTELNzhK3zvC6",
            "signer": false,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx",
            "signer": false,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "11111111111111111111111111111111",
            "signer": false,
            "source": "transaction",
            "writable": false
          },
          {
            "pubkey": "ComputeBudget111111111111111111111111111111",
            "signer": false,
            "source": "transaction",
            "writable": false
          },
          {
            "pubkey": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            "signer": false,
            "source": "transaction",
            "writable": false
          },
          {
            "pubkey": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            "signer": false,
            "source": "transaction",
            "writable": false
          }
        ],
        "instructions": [
          {
            "accounts": [],
            "data": "3gJqkocMWaMm",
            "programId": "ComputeBudget111111111111111111111111111111",
            "stackHeight": null
          },
          {
            "accounts": [],
            "data": "Fj2Eoy",
            "programId": "ComputeBudget111111111111111111111111111111",
            "stackHeight": null
          },
          {
            "parsed": {
              "info": {
                "destination": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                "lamports": 14380000,
                "source": "E3dQM443fE4qfF7seeSjkXSkfghbpzCkY2pJqVPnEm26"
              },
              "type": "transfer"
            },
            "program": "system",
            "programId": "11111111111111111111111111111111",
            "stackHeight": null
          },
          {
            "parsed": {
              "info": {
                "account": "3hntCFiY3a3QFUjcYXnbxc1pp4cMFGEsTELNzhK3zvC6",
                "amount": "10000",
                "authority": "E3dQM443fE4qfF7seeSjkXSkfghbpzCkY2pJqVPnEm26",
                "mint": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx"
              },
              "type": "burn"
            },
            "program": "spl-token",
            "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            "stackHeight": null
          },
          {
            "parsed": "bc1p830q5uwpaxpmzaam2t93jgcq55nrs0x2n6xhl70arkzu3gy9w00qwa7pug",
            "program": "spl-memo",
            "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            "stackHeight": null
          }
        ],
        "recentBlockhash": "2sZDws5kCHRKoNADzcKwFPNRURFuRBeAszwrzFnXRjbd"
      },
      "signatures": [
        "5ib76kdHiu39h8Tsi7aAJNmwdpz8jMvz7QVuhsuXqCjTAwDop6hJ4TbrwLT7Nfeit6gFN3NYxM2Z2MezMApSfu3d"
      ]
    }
  },
  "id": 1
}
        "#;

        let transaction_response =
            serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(json_data)
                .expect("Failed to parse JSON");

        println!("transaction_response: {:#?}", transaction_response);
        for instruction in &transaction_response
            .result
            .unwrap()
            .transaction
            .message
            .instructions
        {
            if instruction.parsed.is_none() {
                println!("Skipped unknown instruction");
                continue;
            }
            if let Ok(parsed_value) = from_value::<ParsedValue>(instruction.parsed.clone().unwrap())
            {
                println!("Parsed value: {:#?}", parsed_value);

                if let Ok(pi) = from_value::<ParsedIns>(parsed_value.parsed.clone()) {
                    match pi.instr_type.as_str() {
                        "transfer" => {
                            let transfer = from_value::<Transfer>(pi.info.clone());
                            println!("Parsed transfer: {:#?}", transfer);
                        }
                        "burnChecked" => {
                            let burn = from_value::<BurnChecked>(pi.info.clone());
                            println!("Parsed burn: {:#?}", burn);
                        }
                        "burn" => {
                            let burn = from_value::<Burn>(pi.info.clone());
                            println!("Parsed burn: {:#?}", burn);
                        }
                        _ => {
                            println!("Skipped non-relevant instruction: {:#?}", pi.instr_type);
                        }
                    }
                } else if let Ok(memo) = from_value::<String>(parsed_value.parsed.clone()) {
                    println!("Parsed memo: {:?}", memo);
                } else {
                    println!("Unknown Parsed instruction: {:#?}", parsed_value.parsed);
                }
            } else {
                println!("Unknown Parsed Value: {:#?}", instruction.parsed);
            }
        }
    }

    #[test]
    fn test_parse_tx_without_log_message() {
        let json_data = r#"
            {
  "jsonrpc": "2.0",
  "result": {
    "blockTime": 1727663653,
    "meta": {
      "computeUnitsConsumed": 18580,
      "err": null,
      "fee": 25000,
      "innerInstructions": null,
      "logMessages": null,
      "postBalances": [
        458649200,
        2074080,
        3897600,
        1,
        1,
        521498880,
        1141440
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx",
          "owner": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
          "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
          "uiTokenAmount": {
            "amount": "320000",
            "decimals": 2,
            "uiAmount": 3200.0,
            "uiAmountString": "3200"
          }
        }
      ],
      "preBalances": [
        458674200,
        2074080,
        3897600,
        1,
        1,
        521498880,
        1141440
      ],
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx",
          "owner": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
          "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
          "uiTokenAmount": {
            "amount": "1120000",
            "decimals": 2,
            "uiAmount": 11200.0,
            "uiAmountString": "11200"
          }
        }
      ],
      "rewards": [],
      "status": {
        "Ok": null
      }
    },
    "slot": 292803140,
    "transaction": {
      "message": {
        "accountKeys": [
          {
            "pubkey": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
            "signer": true,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "4qwZrtmZpoMqtUKCSeCPmPLdgZ6EJJ7j3P9vSf88CkLS",
            "signer": false,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx",
            "signer": false,
            "source": "transaction",
            "writable": true
          },
          {
            "pubkey": "11111111111111111111111111111111",
            "signer": false,
            "source": "transaction",
            "writable": false
          },
          {
            "pubkey": "ComputeBudget111111111111111111111111111111",
            "signer": false,
            "source": "transaction",
            "writable": false
          },
          {
            "pubkey": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            "signer": false,
            "source": "transaction",
            "writable": false
          },
          {
            "pubkey": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            "signer": false,
            "source": "transaction",
            "writable": false
          }
        ],
        "instructions": [
          {
            "accounts": [],
            "data": "3gJqkocMWaMm",
            "programId": "ComputeBudget111111111111111111111111111111",
            "stackHeight": null
          },
          {
            "accounts": [],
            "data": "Fj2Eoy",
            "programId": "ComputeBudget111111111111111111111111111111",
            "stackHeight": null
          },
          {
            "parsed": {
              "info": {
                "destination": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                "lamports": 14380000,
                "source": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
              },
              "type": "transfer"
            },
            "program": "system",
            "programId": "11111111111111111111111111111111",
            "stackHeight": null
          },
          {
            "parsed": {
              "info": {
                "account": "4qwZrtmZpoMqtUKCSeCPmPLdgZ6EJJ7j3P9vSf88CkLS",
                "amount": "800000",
                "authority": "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia",
                "mint": "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx"
              },
              "type": "burn"
            },
            "program": "spl-token",
            "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            "stackHeight": null
          },
          {
            "parsed": "bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6",
            "program": "spl-memo",
            "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            "stackHeight": null
          }
        ],
        "recentBlockhash": "3QAxYh7P3Z1iu5Uiz8pwUM9pKdqiMw9mfwLRBKbQ5hEQ"
      },
      "signatures": [
      ]
    }
  },
  "id": 1
}
        "#;

        let transaction_response =
            serde_json::from_str::<JsonRpcResponse<TransactionDetail>>(json_data)
                .expect("Failed to parse JSON");

        println!("transaction_response: {:#?}", transaction_response);
        for instruction in &transaction_response
            .result
            .unwrap()
            .transaction
            .message
            .instructions
        {
            if instruction.parsed.is_none() {
                println!("Skipped unknown instruction");
                continue;
            }
            if let Ok(parsed_value) = from_value::<ParsedValue>(instruction.parsed.clone().unwrap())
            {
                println!("Parsed value: {:#?}", parsed_value);

                if let Ok(pi) = from_value::<ParsedIns>(parsed_value.parsed.clone()) {
                    match pi.instr_type.as_str() {
                        "transfer" => {
                            let transfer = from_value::<Transfer>(pi.info.clone());
                            println!("Parsed transfer: {:#?}", transfer);
                        }
                        "burnChecked" => {
                            let burn = from_value::<BurnChecked>(pi.info.clone());
                            println!("Parsed burn: {:#?}", burn);
                        }
                        "burn" => {
                            let burn = from_value::<Burn>(pi.info.clone());
                            println!("Parsed burn: {:#?}", burn);
                        }
                        _ => {
                            println!("Skipped non-relevant instruction: {:#?}", pi.instr_type);
                        }
                    }
                } else if let Ok(memo) = from_value::<String>(parsed_value.parsed.clone()) {
                    println!("Parsed memo: {:?}", memo);
                } else {
                    println!("Unknown Parsed instruction: {:#?}", parsed_value.parsed);
                }
            } else {
                println!("Unknown Parsed Value: {:#?}", instruction.parsed);
            }
        }
    }

    #[test]
    fn test_management_canister() {
        let principal = Principal::management_canister();
        println!("The management principal value is: {}", principal)
    }
}
