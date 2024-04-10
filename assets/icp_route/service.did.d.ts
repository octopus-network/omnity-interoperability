import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Chain {
  'fee_token' : [] | [string],
  'chain_id' : string,
  'chain_state' : ChainState,
  'chain_type' : ChainType,
  'contract_address' : [] | [string],
}
export type ChainState = { 'Active' : null } |
  { 'Deactive' : null };
export type ChainType = { 'SettlementChain' : null } |
  { 'ExecutionChain' : null };
export type Event = {
    'finalized_gen_ticket' : {
      'block_index' : bigint,
      'request' : GenerateTicketReq,
    }
  } |
  { 'updated_fee' : { 'fee' : Factor } } |
  { 'finalized_mint_token' : MintTokenRequest } |
  { 'added_token' : { 'token' : Token, 'ledger_id' : Principal } } |
  { 'added_chain' : Chain } |
  { 'toggle_chain_state' : ToggleState };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeeTokenFactor {
  'fee_token' : string,
  'fee_token_factor' : bigint,
}
export type GenerateTicketError = {
    'InsufficientRedeemFee' : { 'provided' : bigint, 'required' : bigint }
  } |
  { 'SendTicketErr' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'InsufficientAllowance' : { 'allowance' : bigint } } |
  { 'TransferFailure' : string } |
  { 'RedeemFeeNotSet' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export interface GenerateTicketOk { 'block_index' : bigint }
export interface GenerateTicketReq {
  'token_id' : string,
  'from_subaccount' : [] | [Uint8Array | number[]],
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface GetEventsArg { 'start' : bigint, 'length' : bigint }
export interface InitArgs { 'hub_principal' : Principal, 'chain_id' : string }
export interface Log { 'log' : string, 'offset' : bigint }
export interface Logs { 'logs' : Array<Log>, 'all_logs_count' : bigint }
export interface MintTokenRequest {
  'token_id' : string,
  'ticket_id' : string,
  'finalized_block_index' : [] | [bigint],
  'amount' : bigint,
  'receiver' : Principal,
}
export type MintTokenStatus = { 'Finalized' : GenerateTicketOk } |
  { 'Unknown' : null };
export type Result = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type RouteArg = { 'Upgrade' : {} } |
  { 'Init' : InitArgs };
export interface TargetChainFactor {
  'target_chain_id' : string,
  'target_chain_factor' : bigint,
}
export type ToggleAction = { 'Deactivate' : null } |
  { 'Activate' : null };
export interface ToggleState { 'action' : ToggleAction, 'chain_id' : string }
export interface Token {
  'decimals' : number,
  'token_id' : string,
  'metadata' : [] | [Array<[string, string]>],
  'icon' : [] | [string],
  'issue_chain' : string,
  'symbol' : string,
}
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_events' : ActorMethod<[GetEventsArg], Array<Event>>,
  'get_fee_account' : ActorMethod<[[] | [Principal]], Uint8Array | number[]>,
  'get_log_records' : ActorMethod<[bigint, bigint], Logs>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_token_ledger' : ActorMethod<[string], [] | [Principal]>,
  'get_token_list' : ActorMethod<[], Array<Token>>,
  'mint_token_status' : ActorMethod<[string], MintTokenStatus>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
