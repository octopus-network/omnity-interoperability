use crate::*;
use business::{
    mint_token::MintTokenRequest,
    redeem_token::{parse_redeem_event, RedeemEvent},
};
use cosmrs::{
    tendermint::{
        self,
        abci::{Event, EventAttributeIndexExt},
    },
    AccountId,
};
use cosmwasm::rpc::tx::TxResultByHashResponse;
use cosmwasm_schema::cw_serde;
use memory::read_state;
use std::collections::HashMap;

use crate::{cosmwasm::rpc::wrapper::Wrapper, CosmWasmClient};

use super::TxHash;

pub type ChainId = String;
pub type TokenId = String;

pub const REDEEM_EVENT_KIND: &str = "wasm-RedeemRequested";
pub const DIRECTIVE_EXECUTED_EVENT_KIND: &str = "wasm-DirectiveExecuted";
pub const TOKEN_MINTED_EVENT_KIND: &str = "wasm-TokenMinted";

#[derive(Debug, Clone)]
pub struct PortContractExecutor {
    pub client: CosmWasmClient,
    pub contract_id: AccountId,
    pub tendermint_public_key: tendermint::public_key::PublicKey,
}

impl PortContractExecutor {
    pub fn new(
        client: CosmWasmClient,
        contract_id: AccountId,
        tendermint_public_key: tendermint::public_key::PublicKey,
    ) -> Self {
        Self {
            client,
            contract_id,
            tendermint_public_key,
        }
    }

    pub fn from_state() -> Result<PortContractExecutor> {
        let client = CosmWasmClient::cosmos_wasm_port_client();
        let contract_id = get_contract_id();
        // let public_key_response = query_cw_public_key().await?;
        let public_key_vec = read_state(|s| {
            s.cw_public_key_vec.clone().ok_or(RouteError::CustomError(
                "cw_public_key_vec not found".to_string(),
            ))
            // .expect("cw_public_key_vec not found")
        })?;

        let tendermint_public_key =
            tendermint::public_key::PublicKey::from_raw_secp256k1(public_key_vec.as_slice())
                .ok_or(RouteError::CustomError(
                    "failed to init tendermint public key".to_string(),
                ))?;

        Ok(Self::new(client, contract_id, tendermint_public_key))
    }

    pub async fn execute_directive(&self, seq: u64, directive: Directive) -> Result<TxHash> {
        let msg = ExecuteMsg::ExecDirective { seq, directive };

        let tx_hash = self
            .client
            .execute_msg(
                self.contract_id.clone(),
                msg,
                self.tendermint_public_key.clone(),
            )
            .await?;

        log::info!("execute directive tx_hash: {:?}", tx_hash);

        let mut times: i32 = 3;
        while times > 0 {
            match self.confirm_execute_directive(seq, tx_hash.clone()).await {
                Ok(_) => {
                    return Ok(tx_hash);
                }
                Err(_) => {}
            }
            times -= 1;
        }

        Err(RouteError::ConfirmExecuteDirectiveErr(seq, tx_hash))
    }

    pub async fn query_redeem_token_event(&self, tx_hash: TxHash) -> Result<RedeemEvent> {
        let tx_response = self.client.query_tx_by_hash(tx_hash).await?;
        let wrapper: Wrapper<TxResultByHashResponse> =
            serde_json::from_slice(&tx_response.body).unwrap();

        let result: TxResultByHashResponse = wrapper.into_result()?;
        log::info!("tx_result: {:?}", result);
        let event = result
            .find_first_event_by_kind(REDEEM_EVENT_KIND.to_string())
            .ok_or(RouteError::EventNotFound("RedeemRequested".to_string()))?;
        log::info!("event: {:?}", event);
        let redeem_event = parse_redeem_event(event)?;
        Ok(redeem_event)
    }

    pub async fn confirm_execute_directive(&self, seq: u64, tx_hash: TxHash) -> Result<()> {
        let tx_response = self.client.query_tx_by_hash(tx_hash).await?;
        let wrapper: Wrapper<TxResultByHashResponse> =
            serde_json::from_slice(&tx_response.body).unwrap();

        let result: TxResultByHashResponse = wrapper.into_result()?;
        log::info!("tx_result: {:?}", result);

        let expect_event = Event::new(
            DIRECTIVE_EXECUTED_EVENT_KIND,
            [("sequence", seq.to_string()).no_index()],
        );
        result.assert_event_exist(&expect_event)?;
        Ok(())
    }

    pub async fn confirm_mint_token(
        &self,
        mint_token_request: MintTokenRequest,
        tx_hash: TxHash,
    ) -> Result<()> {
        let tx_response = self.client.query_tx_by_hash(tx_hash).await?;
        let wrapper: Wrapper<TxResultByHashResponse> =
            serde_json::from_slice(&tx_response.body).unwrap();

        let result: TxResultByHashResponse = wrapper.into_result()?;
        log::info!("tx_result: {:?}", result);

        let expect_event = Event::new(
            TOKEN_MINTED_EVENT_KIND,
            [
                ("ticket_id", mint_token_request.ticket_id).no_index(),
                ("token_id", mint_token_request.token_id).no_index(),
                ("receiver", mint_token_request.receiver).no_index(),
                ("amount", mint_token_request.amount.to_string()).no_index(),
            ],
        );
        result.assert_event_exist(&expect_event)?;
        Ok(())
    }

    pub async fn mint_token(&self, mint_token_request: MintTokenRequest) -> Result<TxHash> {
        let msg = ExecuteMsg::PrivilegeMintToken {
            ticket_id: mint_token_request.ticket_id.clone(),
            token_id: mint_token_request.token_id.clone(),
            receiver: mint_token_request.receiver.clone(),
            amount: mint_token_request.amount.clone().to_string(),
        };

        let tx_hash = self
            .client
            .execute_msg(
                self.contract_id.clone(),
                msg,
                self.tendermint_public_key.clone(),
            )
            .await?;

        log::info!("mint token tx_hash: {:?}", tx_hash);

        let mut times = 3;
        while times > 0 {
            match self
                .confirm_mint_token(mint_token_request.clone(), tx_hash.clone())
                .await
            {
                Ok(_) => {
                    return Ok(tx_hash);
                }
                Err(_) => {}
            }
            times -= 1;
        }
        Err(RouteError::ConfirmMintTokenErr(
            format!("{:?}", mint_token_request).to_string(),
            tx_hash,
        ))
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    ExecDirective {
        seq: u64,
        directive: Directive,
    },
    PrivilegeMintToken {
        ticket_id: String,
        token_id: String,
        receiver: String,
        amount: String,
    },
}

#[cw_serde]
pub struct Chain {
    pub chain_id: ChainId,
    pub canister_id: String,
    pub chain_type: ChainType,
    // the chain default state is true
    pub chain_state: ChainState,
    // settlement chain: export contract address
    // execution chain: port contract address
    pub contract_address: Option<String>,

    // optional counterparty chains
    pub counterparties: Option<Vec<ChainId>>,
    // fee token
    pub fee_token: Option<TokenId>,
}

impl From<omnity_types::Chain> for Chain {
    fn from(value: omnity_types::Chain) -> Self {
        Self {
            chain_id: value.chain_id,
            canister_id: value.canister_id,
            chain_type: value.chain_type.into(),
            chain_state: value.chain_state.into(),
            contract_address: value.contract_address,
            counterparties: value.counterparties,
            fee_token: value.fee_token,
        }
    }
}

#[cw_serde]
pub enum ChainType {
    SettlementChain,
    ExecutionChain,
}

impl From<omnity_types::ChainType> for ChainType {
    fn from(value: omnity_types::ChainType) -> Self {
        match value {
            omnity_types::ChainType::SettlementChain => Self::SettlementChain,
            omnity_types::ChainType::ExecutionChain => Self::ExecutionChain,
        }
    }
}

#[cw_serde]
pub enum ChainState {
    Active,
    Deactive,
}

impl From<omnity_types::ChainState> for ChainState {
    fn from(value: omnity_types::ChainState) -> Self {
        match value {
            omnity_types::ChainState::Active => Self::Active,
            omnity_types::ChainState::Deactive => Self::Deactive,
        }
    }
}

#[cw_serde]
pub enum Directive {
    AddChain(Chain),
    AddToken(Token),
    UpdateChain(Chain),
    UpdateToken(Token),
    ToggleChainState(ToggleState),
    UpdateFee(Factor),
}

impl From<omnity_types::Directive> for Directive {
    fn from(value: omnity_types::Directive) -> Self {
        match value {
            omnity_types::Directive::AddChain(chain) => Self::AddChain(chain.into()),
            omnity_types::Directive::AddToken(token) => Self::AddToken(token.into()),
            omnity_types::Directive::UpdateChain(chain) => Self::UpdateChain(chain.into()),
            omnity_types::Directive::UpdateToken(token) => Self::UpdateToken(token.into()),
            omnity_types::Directive::ToggleChainState(toggle) => {
                Self::ToggleChainState(toggle.into())
            }
            omnity_types::Directive::UpdateFee(factor) => Self::UpdateFee(factor.into()),
        }
    }
}

#[cw_serde]
pub struct ToggleState {
    pub chain_id: ChainId,
    pub action: ToggleAction,
}

impl From<omnity_types::ToggleState> for ToggleState {
    fn from(value: omnity_types::ToggleState) -> Self {
        Self {
            chain_id: value.chain_id,
            action: value.action.into(),
        }
    }
}

#[cw_serde]
pub enum ToggleAction {
    Activate,
    Deactivate,
}

impl From<omnity_types::ToggleAction> for ToggleAction {
    fn from(value: omnity_types::ToggleAction) -> Self {
        match value {
            omnity_types::ToggleAction::Activate => Self::Activate,
            omnity_types::ToggleAction::Deactivate => Self::Deactivate,
        }
    }
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
    pub name: String,
    pub symbol: String,

    pub decimals: u8,
    pub icon: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl From<omnity_types::Token> for Token {
    fn from(value: omnity_types::Token) -> Self {
        Self {
            token_id: value.token_id,
            name: value.name,
            symbol: value.symbol,
            decimals: value.decimals,
            icon: value.icon,
            metadata: value.metadata,
        }
    }
}

#[cw_serde]
pub enum Factor {
    UpdateTargetChainFactor(TargetChainFactor),
    UpdateFeeTokenFactor(FeeTokenFactor),
}

impl From<omnity_types::Factor> for Factor {
    fn from(value: omnity_types::Factor) -> Self {
        match value {
            omnity_types::Factor::UpdateTargetChainFactor(factor) => {
                Self::UpdateTargetChainFactor(factor.into())
            }
            omnity_types::Factor::UpdateFeeTokenFactor(factor) => {
                Self::UpdateFeeTokenFactor(factor.into())
            }
        }
    }
}

#[cw_serde]
pub struct TargetChainFactor {
    pub target_chain_id: ChainId,
    pub target_chain_factor: u128,
}

impl From<omnity_types::TargetChainFactor> for TargetChainFactor {
    fn from(value: omnity_types::TargetChainFactor) -> Self {
        Self {
            target_chain_id: value.target_chain_id,
            target_chain_factor: value.target_chain_factor,
        }
    }
}

#[cw_serde]
pub struct FeeTokenFactor {
    pub fee_token: TokenId,
    pub fee_token_factor: u128,
}

impl From<omnity_types::FeeTokenFactor> for FeeTokenFactor {
    fn from(value: omnity_types::FeeTokenFactor) -> Self {
        Self {
            fee_token: value.fee_token,
            fee_token_factor: value.fee_token_factor,
        }
    }
}
