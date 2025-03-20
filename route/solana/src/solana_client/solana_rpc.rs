use crate::constants::RPC_CYCLE_COST;
use crate::eddsa::{eddsa_public_key, KeyType};
use crate::eddsa::{hash_with_sha256, sign_with_eddsa};
use crate::state::read_state;
use anyhow::anyhow;
use anyhow::Error;
use candid::{CandidType, Principal};
use core::fmt;
use ic_canister_log::log;
use ic_cdk::api;
use ic_cdk::api::call::call_with_payment;
use ic_cdk::api::management_canister::http_request::HttpHeader;
use ic_solana::logs::DEBUG;
use ic_solana::request::RpcRequest;
use ic_solana::rpc_client::{RpcApi, RpcConfig, RpcServices};
use ic_solana::types::tagged::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiAccount, UiTransaction,
};
use ic_solana::types::{
    BlockHash, Instruction, Message, RpcContextConfig, RpcSendTransactionConfig,
    RpcSignatureStatusConfig, RpcSimulateTransactionConfig, RpcSimulateTransactionResult,
    RpcTransactionConfig, Signature, Transaction, TransactionStatus, UiAccountEncoding,
    UiTransactionEncoding,
};
use ic_solana::types::{RpcAccountInfoConfig, RpcBlockhash};
use ic_solana::{rpc_client::RpcResult, types::Pubkey};
use ic_spl::compute_budget::compute_budget::{
    ComputeBudgetInstruction, Priority, DEFAULT_COMPUTE_UNITS,
};

use ic_spl::metaplex::create_fungible_ix::create_fungible_ix;
use ic_spl::metaplex::create_fungible_ix::CreateFungibleArgs;
use ic_spl::metaplex::create_metadata_ix::create_metadata_ix;
use ic_spl::metaplex::create_metadata_ix::CreateMetadataArgs;
use ic_spl::metaplex::types::FungibleFields;
use ic_spl::metaplex::update_metadata_ix::{update_asset_v1_ix, UpdateMetaArgs};

use ic_spl::token::{system_instruction, token_instruction};

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::cell::RefCell;
use std::str::FromStr;

thread_local! {
    static NEXT_ID: RefCell<u64> = RefCell::default();
}

pub fn next_request_id() -> u64 {
    NEXT_ID.with(|next_id| {
        let mut next_id = next_id.borrow_mut();
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        id
    })
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct TxError {
    pub block_hash: String,
    pub signature: String,
    pub error: String,
}
impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxError: block_hash={}, signature={}, error={}",
            self.block_hash, self.signature, self.error
        )
    }
}
impl std::error::Error for TxError {}
impl TryFrom<Error> for TxError {
    type Error = Error;

    fn try_from(e: Error) -> Result<Self, Self::Error> {
        if let Some(tx_error) = e.downcast_ref::<TxError>() {
            Ok(TxError {
                block_hash: tx_error.block_hash.to_owned(),
                signature: tx_error.signature.to_owned(),
                error: tx_error.error.to_owned(),
            })
        } else {
            Err(e)
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenInfo {
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub uri: String,
}
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct SolanaClient {
    pub sol_canister_id: Principal,
    pub payer: Pubkey,
    pub payer_derive_path: Vec<ByteBuf>,
    pub chainkey_name: String,
    pub priority: Option<Priority>,
    pub key_type: KeyType,
}

fn build_rpc_service<P: Serialize + Clone>(method: RpcRequest, params: P) -> RpcServices {
    let (proxy, providers) = read_state(|s| (s.proxy.to_owned(), s.providers.to_owned()));
    let rpc_apis: Vec<_> = providers
        .iter()
        .map(|p| {
            let network = format!(
                "{}{}",
                proxy,
                p.api_key_param
                    .clone()
                    .map_or("".into(), |param| format!("/?{}", param))
            );
            log!(
                DEBUG,
                "[solana_rpc::build_rpc_service] network: {}",
                network,
            );
            let payload = method.build_json(next_request_id(), params.clone());
            let idempotency_key = hash_with_sha256(&format!("{}{}", p.host, payload));
            let mut headers = vec![
                HttpHeader {
                    name: "x-forwarded-host".into(),
                    value: p.host.clone(),
                },
                HttpHeader {
                    name: "idempotency-key".into(),
                    value: idempotency_key,
                },
            ];
            if let Some(p_headers) = p.headers.as_ref() {
                headers.extend(p_headers.iter().cloned());
            }
            log!(
                DEBUG,
                "[solana_rpc::build_rpc_service] headers: {:?}",
                headers,
            );
            RpcApi {
                network,
                headers: Some(headers),
            }
        })
        .collect();
    RpcServices::Custom(rpc_apis[0..1].to_vec())
}

impl SolanaClient {
    pub async fn derive_account(
        key_type: KeyType,
        chainkey_name: String,
        derive_path: String,
    ) -> Pubkey {
        let path = vec![ByteBuf::from(derive_path.as_str())];
        Pubkey::try_from(eddsa_public_key(key_type, chainkey_name, path).await).unwrap()
    }

    pub async fn get_latest_blockhash(&self) -> anyhow::Result<BlockHash> {
        let config = None::<Option<RpcConfig>>;
        let params = None::<Option<RpcContextConfig>>;
        let source = build_rpc_service(RpcRequest::GetLatestBlockhash, (params,));
        log!(
            DEBUG,
            "[solana_rpc::get_latest_blockhash] rpc sources: {:?}",
            source,
        );
        let response = call_with_payment::<_, (RpcResult<RpcBlockhash>,)>(
            self.sol_canister_id,
            "sol_getLatestBlockhash",
            (source, config, params),
            RPC_CYCLE_COST,
        )
        .await
        .map_err(|e| anyhow!(format!("request solana provider error: {:?}, {}", e.0, e.1)))?
        .0
        .map_err(|e| anyhow!(format!("request latest block hash error: {:?}", e)))?;
        Ok(BlockHash::from_str(&response.blockhash)?)
    }

    pub async fn get_account_info(&self, account: String) -> anyhow::Result<Option<UiAccount>> {
        let config = None::<Option<RpcConfig>>;
        let params = Some(RpcAccountInfoConfig {
            // Encoded binary (base58) data should be less than 128 bytes, so use base64 encoding.
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: None,
            min_context_slot: None,
        });
        let source = build_rpc_service(
            RpcRequest::GetAccountInfo,
            (account.to_owned(), params.to_owned()),
        );

        let r = call_with_payment::<_, (RpcResult<Option<UiAccount>>,)>(
            self.sol_canister_id,
            "sol_getAccountInfo",
            (source, config, account, params),
            RPC_CYCLE_COST,
        )
        .await
        .map_err(|e| {
            anyhow!(format!(
                "[solana_client::get_account_info] call sol_getAccountInfo error: {:?}",
                e
            ))
        })?
        .0
        .map_err(|e| {
            anyhow!(format!(
                "[solana_client::get_account_info] sol_getAccountInfo rpc error:{:?}",
                e
            ))
        })?;
        log!(
            DEBUG,
            "[solana_client::get_account_info] sol_getAccountInfo resp: {:#?} ",
            r
        );

        Ok(r)
    }

    pub async fn get_balance(&self, account: String) -> anyhow::Result<u64> {
        let config = None::<Option<RpcConfig>>;
        let params = None::<Option<RpcContextConfig>>;
        let source = build_rpc_service(
            RpcRequest::GetBalance,
            (account.to_owned(), params.to_owned()),
        );

        let r = call_with_payment::<_, (RpcResult<u64>,)>(
            self.sol_canister_id,
            "sol_getBalance",
            (source, config, account, params),
            RPC_CYCLE_COST,
        )
        .await;
        let resp = r
            .map_err(|e| {
                anyhow!(format!(
                    "[solana_client::get_balance] call get_balance error: {:?}",
                    e
                ))
            })?
            .0
            .map_err(|e| {
                anyhow!(format!(
                    "[solana_client::get_balance] get_balance rpc error:{:?}",
                    e
                ))
            })?;
        log!(
            DEBUG,
            "[solana_client::get_balance] get_balance resp: {:#?} ",
            resp
        );

        Ok(resp)
    }

    pub async fn get_signature_status(
        &self,
        signatures: Vec<String>,
    ) -> Result<Vec<Option<TransactionStatus>>, String> {
        let config = None::<Option<RpcConfig>>;
        let params = Some(RpcSignatureStatusConfig {
            search_transaction_history: true,
        });
        let source = build_rpc_service(
            RpcRequest::GetSignatureStatuses,
            (signatures.to_owned(), params.to_owned()),
        );

        let result = call_with_payment::<_, (RpcResult<Vec<Option<TransactionStatus>>>,)>(
            self.sol_canister_id,
            "sol_getSignatureStatuses",
            (
                // Using normal RPC will not allow consensus to be reached
                // due to inconsistent slots.
                source, config, signatures, params,
            ),
            RPC_CYCLE_COST,
        )
        .await
        .map_err(|(_, err)| err)?
        .0
        .map_err(|err| err.to_string())?;

        Ok(result)
    }

    pub async fn get_compute_units(
        &self,
        instructions: &[Instruction],
        paths: Vec<Vec<ByteBuf>>,
        key_type: KeyType,
    ) -> anyhow::Result<Option<u64>> {
        let message = Message::new(instructions.iter().as_ref(), Some(&self.payer));
        let mut tx = Transaction::new_unsigned(message);

        log!(
            DEBUG,
            "[solana_client::get_compute_units] start exec, path len: {} ",
            paths.len()
        );

        for i in 0..paths.len() {
            let signature = self
                .sign(&key_type, paths[i].clone(), tx.message_data())
                .await?;
            tx.add_signature(i, signature);
        }

        log!(
            DEBUG,
            "[solana_client::get_compute_units] finished sign message"
        );

        let config = None::<Option<RpcConfig>>;

        // let params = Some(RpcSimulateTransactionConfig {
        //     sig_verify: false,
        //     replace_recent_blockhash: true,
        //     commitment: Some(CommitmentLevel::Confirmed),
        //     encoding: Some(UiTransactionEncoding::Base64),
        //     ..Default::default()
        // });

        let params = None::<Option<RpcSimulateTransactionConfig>>;
        let source = build_rpc_service(
            RpcRequest::SimulateTransaction,
            (tx.to_string(), params.to_owned()),
        );

        let response = call_with_payment::<_, (RpcResult<RpcSimulateTransactionResult>,)>(
            self.sol_canister_id,
            "sol_simulateTransaction",
            (source, config, tx.to_string(), params),
            RPC_CYCLE_COST,
        )
        .await;

        log!(DEBUG, "sol_getComputeUnits response: {:?}", response);

        let resp = response
            .map_err(|e| {
                anyhow!(format!(
                    "[solana_client::get_compute_units] call sol_getComputeUnits err: {:?}",
                    e
                ))
            })?
            .0
            .map_err(|e| {
                anyhow!(format!(
                    "[solana_client::get_compute_units] rpc error: {:?}",
                    e
                ))
            })?;

        log!(DEBUG, "get_compute_units response: {:?}", resp);
        let units = resp
            .units_consumed
            .map(|units| (units as f64 * 1.20) as u64);

        Ok(units)
    }

    pub async fn send_raw_transaction(
        &self,
        instructions: &[Instruction],
        paths: Vec<Vec<ByteBuf>>,
        key_type: KeyType,
        // forward: Option<String>,
    ) -> anyhow::Result<String> {
        let mut start = api::time();
        let blockhash = self.get_latest_blockhash().await?;
        let mut end = api::time();
        let mut elapsed = (end - start) / 1_000_000_000;

        log!(
            DEBUG,
            "[solana_client::send_raw_transaction] get_latest_blockhash : {:?} and time elapsed: {}",
            blockhash,elapsed
        );

        let message = Message::new_with_blockhash(
            instructions.iter().as_ref(),
            Some(&self.payer),
            &blockhash.to_owned(),
        );
        let mut tx = Transaction::new_unsigned(message);
        // let mut tx_hash = String::new();
        start = api::time();
        for i in 0..paths.len() {
            let signature = self
                .sign(&key_type, paths[i].clone(), tx.message_data())
                .await?;
            tx.add_signature(i, signature);
        }
        end = api::time();
        elapsed = (end - start) / 1_000_000_000;

        log!(
            DEBUG,
            "[solana_client::send_raw_transaction] the time elapsed for chainkey signing : {}",
            elapsed
        );
        let tx_hash = tx.signatures.first().unwrap().to_string();
        log!(
            DEBUG,
            "[solana_client::send_raw_transaction] tx first signature : {}",
            tx_hash
        );

        start = api::time();

        let config = None::<Option<RpcConfig>>;
        let params = None::<Option<RpcSendTransactionConfig>>;
        let source = build_rpc_service(
            RpcRequest::SendTransaction,
            (tx.to_string(), params.to_owned()),
        );
        let response = call_with_payment::<_, (RpcResult<String>,)>(
            self.sol_canister_id,
            "sol_sendTransaction",
            (source, config, tx.to_string(), params),
            RPC_CYCLE_COST,
        )
        .await;
        log!(DEBUG, "sol_sendRawTransaction response: {:?}", response);
        end = api::time();
        elapsed = (end - start) / 1_000_000_000;
        log!(
            DEBUG,
            "[solana_client::send_raw_transaction] the time elapsed for sol_sendRawTransaction : {}",
            elapsed
        );

        let resp = response
            .map_err(|e| {
                anyhow!(format!(
                    "[solana_client::send_raw_transaction] call send raw transaction err: {:?}",
                    e
                ))
            })?
            .0
            .map_err(|e| {
                let tx_error = TxError {
                    block_hash: blockhash.to_string(),
                    signature: tx_hash,
                    error: format!("[solana_client::send_raw_transaction] rpc error: {:?}", e),
                };
                anyhow!(tx_error)
            })?;

        Ok(resp)
    }

    pub async fn query_transaction(
        &self,
        signature: String,
        // forward: Option<String>,
    ) -> anyhow::Result<UiTransaction> {
        let providers = read_state(|s| (s.providers.to_owned()));
        let rpc_apis: Vec<_> = providers
            .iter()
            .map(|p| RpcApi {
                network: p.rpc_url(),
                headers: p.headers.to_owned(),
            })
            .collect();
        let source = RpcServices::Custom(rpc_apis[0..2].to_vec());

        let config = None::<Option<RpcConfig>>;

        let params = None::<Option<RpcTransactionConfig>>;

        let response = call_with_payment::<
            _,
            (RpcResult<Option<EncodedConfirmedTransactionWithStatusMeta>>,),
        >(
            self.sol_canister_id,
            "sol_getTransaction",
            (source, config, signature, params),
            RPC_CYCLE_COST,
        )
        .await
        .map_err(|e| anyhow!(format!("query transaction err: {:?}", e)))?
        .0
        .map_err(|e| anyhow!(format!("query transaction rpc error: {:?}", e)))?;

        match response {
            None => Err(anyhow!("result of query_transaction is None".to_string())),
            Some(tx) => match tx.transaction.transaction {
                EncodedTransaction::Json(tx) => Ok(tx),
                _ => Err(anyhow!("invalid type of query_transaction".to_string())),
            },
        }
    }

    pub async fn query_raw_transaction(
        &self,
        source: RpcServices,
        signature: String,
        // forward: Option<String>,
    ) -> anyhow::Result<Vec<u8>> {
    
        let config = None::<Option<RpcConfig>>;
        let params = Some(RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: None,
            max_supported_transaction_version: None,
        });

        let response = call_with_payment::<_, (RpcResult<Vec<u8>>,)>(
            self.sol_canister_id,
            "sol_getRawTransaction",
            (source, config, signature, params),
            RPC_CYCLE_COST,
        )
        .await
        .map_err(|e| anyhow!(format!("query transaction err: {:?}", e)))?
        .0
        .map_err(|e| anyhow!(format!("query transaction rpc error: {:?}", e)))?;

        Ok(response)
        // let tx = serde_json::from_str(tx.as_str()).unwrap();
        // Ok(tx)
    }

    pub async fn create_mint_with_metaplex(
        &self,
        token_mint: Pubkey,
        token_info: TokenInfo,
    ) -> anyhow::Result<String> {
        let metadata = FungibleFields {
            name: token_info.name,
            symbol: token_info.symbol,
            uri: token_info.uri,
        };
        let create_arg = CreateFungibleArgs {
            mint: token_mint,
            metadata,
            immutable: false,
            decimals: token_info.decimals,
            payer: self.payer.to_owned(),
        };
        let mut instructions = vec![create_fungible_ix(create_arg)];

        let derive_path = hash_with_sha256(token_info.token_id.to_owned().as_str());

        if let Some(priority) = &self.priority {
            self.set_compute_unit(
                &mut instructions,
                vec![
                    self.payer_derive_path.to_owned(),
                    vec![ByteBuf::from(derive_path.to_owned())],
                ],
                priority.to_owned(),
                self.key_type.to_owned(),
            )
            .await?;
        }

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![
                    self.payer_derive_path.to_owned(),
                    vec![ByteBuf::from(derive_path)],
                ],
                self.key_type.to_owned(),
            )
            .await?;
        Ok(tx_hash)
    }

    pub async fn create_metadata_account(
        &self,
        mint: String,
        metadata: FungibleFields,
        immutable: bool,
        // forward: Option<String>,
    ) -> anyhow::Result<String> {
        let meta_args = CreateMetadataArgs {
            mint: mint,
            metadata: metadata,
            immutable: immutable,
            payer: self.payer.to_owned(),
        };
        let create_ix = create_metadata_ix(meta_args).unwrap();
        let instructions = vec![create_ix];

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![self.payer_derive_path.clone()],
                KeyType::ChainKey,
            )
            .await?;
        Ok(tx_hash)
    }

    pub async fn update_with_metaplex(
        &self,
        token_mint: Pubkey,
        token_info: TokenInfo,
    ) -> anyhow::Result<String> {
        let update_meta_args = UpdateMetaArgs {
            payer: self.payer,
            mint_account: token_mint,
            name: token_info.name.to_owned(),
            symbol: token_info.symbol.to_owned(),
            uri: token_info.uri.to_owned(),
            seller_fee_basis_points: 0u16,
            creators: None,
        };
        let mut instructions = vec![update_asset_v1_ix(update_meta_args)];

        if let Some(priority) = &self.priority {
            self.set_compute_unit(
                &mut instructions,
                vec![self.payer_derive_path.to_owned()],
                priority.to_owned(),
                self.key_type.to_owned(),
            )
            .await?;
        }

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![self.payer_derive_path.to_owned()],
                self.key_type.to_owned(),
            )
            .await?;
        Ok(tx_hash)
    }

    pub async fn create_associated_token_account(
        &self,
        owner_addr: &Pubkey,
        token_mint: &Pubkey,
        token_program_id: &Pubkey,
    ) -> anyhow::Result<String> {
        let mut instructions = vec![
            ic_spl::token::associated_account::create_associated_token_account(
                &self.payer,
                &owner_addr,
                &token_mint,
                &token_program_id,
            ),
        ];

        if let Some(priority) = &self.priority {
            self.set_compute_unit(
                &mut instructions,
                vec![self.payer_derive_path.to_owned()],
                priority.to_owned(),
                self.key_type.to_owned(),
            )
            .await?;
        }

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![self.payer_derive_path.to_owned()],
                self.key_type.to_owned(),
            )
            .await?;
        Ok(tx_hash)
    }

    pub async fn mint_to(
        &self,
        associated_account: Pubkey,
        amount: u64,
        token_mint: Pubkey,
        token_program_id: Pubkey,
    ) -> anyhow::Result<String> {
        let mut instructions = vec![token_instruction::mint_to(
            &token_program_id,
            &token_mint,
            &associated_account,
            &self.payer,
            &[],
            amount,
        )];

        if let Some(priority) = &self.priority {
            self.set_compute_unit(
                &mut instructions,
                vec![self.payer_derive_path.to_owned()],
                priority.to_owned(),
                self.key_type.to_owned(),
            )
            .await?;
        }

        log!(DEBUG, "[solana_client::mint_to] set compoute unit success");

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![self.payer_derive_path.to_owned()],
                self.key_type.to_owned(),
            )
            .await?;

        log!(DEBUG, "[solana_client::mint_to] send tx success");

        Ok(tx_hash)
    }

    pub async fn transfer_to(&self, to_account: Pubkey, amount: u64) -> anyhow::Result<String> {
        self.transfer(
            self.payer,
            self.payer_derive_path.clone(),
            to_account,
            amount,
        )
        .await
    }

    pub async fn transfer(
        &self,
        from_account: Pubkey,
        from_path: Vec<ByteBuf>,
        to_account: Pubkey,
        amount: u64,
    ) -> anyhow::Result<String> {
        let lamports = self.get_balance(from_account.to_string()).await?;
        let fee = 10_000;
        let mut paths = vec![self.payer_derive_path.clone()];
        if from_account == self.payer {
            if lamports < amount + fee {
                return Err(anyhow!("not enough lamports"));
            }
        } else {
            if lamports < amount {
                return Err(anyhow!("not enough lamports"));
            }
            let fee_lamports = self.get_balance(self.payer.to_string()).await?;
            if fee_lamports < fee {
                return Err(anyhow!("not enough fee lamports"));
            }
            paths.push(from_path);
        }

        let instructions = vec![system_instruction::transfer(
            &from_account,
            &to_account,
            amount,
        )];

        let tx_hash = self
            .send_raw_transaction(instructions.as_slice(), paths, KeyType::ChainKey)
            .await?;
        Ok(tx_hash)
    }

    async fn sign(
        &self,
        key_type: &KeyType,
        key_path: Vec<ByteBuf>,
        tx: Vec<u8>,
    ) -> anyhow::Result<Signature> {
        let signature = sign_with_eddsa(key_type, self.chainkey_name.clone(), key_path, tx)
            .await
            .try_into()
            .map_err(|e| anyhow!("invalid signature: {:?}", e))?;
        Ok(signature)
    }

    async fn set_compute_unit(
        &self,
        instructions: &mut Vec<Instruction>,
        paths: Vec<Vec<ByteBuf>>,
        priority: Priority,
        key_type: KeyType,
    ) -> Result<(), anyhow::Error> {
        let micro_lamports = match priority {
            Priority::None => 20,        // 1       lamports
            Priority::Low => 20_000,     // 1_000   lamports  ~$1 for 10k updates
            Priority::Medium => 200_000, // 10_000  lamports  ~$10 for 10k updates
            Priority::High => 1_000_000, // 50_000  lamports  ~$0.01/update @ $150 SOL
            Priority::Max => 2_000_000,  // 100_000 lamports  ~$0.02/update @ $150 SOL
        };
        let mut extra_instructions = vec![];
        let compute_units = self
            .get_compute_units(&*instructions, paths, key_type)
            .await?
            .unwrap_or(DEFAULT_COMPUTE_UNITS);
        extra_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            compute_units as u32,
        ));
        extra_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            micro_lamports,
        ));
        instructions.splice(0..0, extra_instructions);

        Ok(())
    }

    pub async fn close_account(
        &self,
        token_program_id: Pubkey,
        close_account: Pubkey,
        dest_account: Pubkey,
        // forward: Option<String>,
    ) -> anyhow::Result<String> {
        let instructions = vec![token_instruction::close_account(
            &token_program_id,
            &close_account,
            &dest_account,
            &self.payer,
            &[],
        )];
        log!(
            DEBUG,
            "[solana_client::close_account] instructions: {:?} ",
            instructions
        );

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![self.payer_derive_path.clone()],
                KeyType::ChainKey,
            )
            .await?;
        Ok(tx_hash)
    }

    pub async fn freeze_account(
        &self,
        token_program_id: Pubkey,
        freeze_account: Pubkey,
        mint_account: Pubkey,
        // forward: Option<String>,
    ) -> anyhow::Result<String> {
        let instructions = vec![token_instruction::freeze_account(
            &token_program_id,
            &freeze_account,
            &mint_account,
            &self.payer,
            &[],
        )];
        log!(
            DEBUG,
            "[solana_client::freeze_account] instructions: {:?} ",
            instructions
        );

        let tx_hash = self
            .send_raw_transaction(
                instructions.as_slice(),
                vec![self.payer_derive_path.clone()],
                KeyType::ChainKey,
            )
            .await?;
        Ok(tx_hash)
    }
}
