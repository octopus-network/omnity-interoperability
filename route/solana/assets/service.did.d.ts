import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AccountInfo {
  'status' : TxStatus,
  'signature' : [] | [string],
  'retry_4_building' : bigint,
  'account' : string,
  'retry_4_status' : bigint,
}
export interface CallError { 'method' : string, 'reason' : Reason }
export interface Chain {
  'fee_token' : [] | [string],
  'canister_id' : string,
  'chain_id' : string,
  'counterparties' : [] | [Array<string>],
  'chain_state' : ChainState,
  'chain_type' : ChainType,
  'contract_address' : [] | [string],
}
export type ChainState = { 'Active' : null } |
  { 'Deactive' : null };
export type ChainType = { 'SettlementChain' : null } |
  { 'ExecutionChain' : null };
export type GenerateTicketError = {
    'InsufficientRedeemFee' : { 'provided' : bigint, 'required' : bigint }
  } |
  { 'SendTicketErr' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'InsufficientAllowance' : { 'allowance' : bigint } } |
  { 'TransferFailure' : string } |
  { 'UnsupportedAction' : string } |
  { 'RedeemFeeNotSet' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export interface GenerateTicketOk { 'ticket_id' : string }
export interface GenerateTicketReq {
  'signature' : string,
  'action' : TxAction,
  'token_id' : string,
  'memo' : [] | [string],
  'sender' : string,
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface InitArgs {
  'admin' : Principal,
  'hub_principal' : Principal,
  'fee_account' : [] | [string],
  'sol_canister' : Principal,
  'chain_id' : string,
  'schnorr_key_name' : [] | [string],
  'chain_state' : ChainState,
}
export interface MintTokenRequest {
  'status' : TxStatus,
  'signature' : [] | [string],
  'associated_account' : string,
  'retry_4_building' : bigint,
  'ticket_id' : string,
  'retry_4_status' : bigint,
  'amount' : bigint,
  'token_mint' : string,
}
export type Reason = { 'QueueIsFull' : null } |
  { 'CanisterError' : string } |
  { 'OutOfCycles' : null } |
  { 'Rejected' : string } |
  { 'TxError' : TxError };
export type Result = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type Result_1 = { 'Ok' : MintTokenRequest } |
  { 'Err' : CallError };
export type Result_2 = { 'Ok' : TxStatus } |
  { 'Err' : CallError };
export type Result_3 = { 'Ok' : [] | [string] } |
  { 'Err' : CallError };
export type RouteArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export interface TxError {
  'signature' : string,
  'block_hash' : string,
  'error' : string,
}
export type TxStatus = { 'New' : null } |
  { 'Finalized' : null } |
  { 'TxFailed' : { 'e' : TxError } } |
  { 'Pending' : null };
export interface UpgradeArgs {
  'admin' : [] | [Principal],
  'hub_principal' : [] | [Principal],
  'fee_account' : [] | [string],
  'sol_canister' : [] | [Principal],
  'chain_id' : [] | [string],
  'schnorr_key_name' : [] | [string],
  'chain_state' : [] | [ChainState],
}
export interface _SERVICE {
  'gen_tickets_req' : ActorMethod<[string], [] | [GenerateTicketReq]>,
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_fee_account' : ActorMethod<[], string>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'mint_token_req' : ActorMethod<[string], Result_1>,
  'mint_token_status' : ActorMethod<[string], Result_2>,
  'mint_token_tx_hash' : ActorMethod<[string], Result_3>,
  'query_mint_account' : ActorMethod<[string], [] | [AccountInfo]>,
  'query_mint_address' : ActorMethod<[string], [] | [string]>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
