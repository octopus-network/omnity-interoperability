import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'query_stats' : QueryStats,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
  'reserved_cycles' : bigint,
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
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
export type ChangeFeeCollector = { 'SetTo' : Account } |
  { 'Unset' : null };
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'reserved_cycles_limit' : bigint,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
export type Event = {
    'finalized_gen_ticket' : {
      'request' : GenerateTicketReq,
      'ticket_id' : string,
    }
  } |
  { 'updated_fee' : { 'fee' : Factor } } |
  {
    'finalized_mint_token' : { 'block_index' : bigint, 'ticket_id' : string }
  } |
  { 'added_token' : { 'token' : Token, 'ledger_id' : Principal } } |
  { 'init' : InitArgs } |
  { 'upgrade' : UpgradeArgs } |
  { 'added_chain' : Chain } |
  { 'toggle_chain_state' : ToggleState };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeatureFlags { 'icrc2' : boolean }
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
export interface GenerateTicketOk { 'ticket_id' : string }
export interface GenerateTicketReq {
  'token_id' : string,
  'burn' : [] | [boolean],
  'from_subaccount' : [] | [Uint8Array | number[]],
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface GetEventsArg { 'start' : bigint, 'length' : bigint }
export interface InitArgs {
  'hub_principal' : Principal,
  'chain_id' : string,
  'chain_state' : ChainState,
}
export interface Log { 'log' : string, 'offset' : bigint }
export interface Logs { 'logs' : Array<Log>, 'all_logs_count' : bigint }
export type MetadataValue = { 'Int' : bigint } |
  { 'Nat' : bigint } |
  { 'Blob' : Uint8Array | number[] } |
  { 'Text' : string };
export type MintTokenStatus = { 'Finalized' : { 'block_index' : bigint } } |
  { 'Unknown' : null };
export interface QueryStats {
  'response_payload_bytes_total' : bigint,
  'num_instructions_total' : bigint,
  'num_calls_total' : bigint,
  'request_payload_bytes_total' : bigint,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : CanisterStatusResponse } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type RouteArg = { 'Upgrade' : [] | [UpgradeArgs] } |
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
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'name' : string,
  'symbol' : string,
}
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export interface UpgradeArgs {
  'hub_principal' : [] | [Principal],
  'chain_id' : [] | [string],
  'chain_state' : [] | [ChainState],
}
export interface UpgradeArgs_1 {
  'token_symbol' : [] | [string],
  'transfer_fee' : [] | [bigint],
  'metadata' : [] | [Array<[string, MetadataValue]>],
  'maximum_number_of_accounts' : [] | [bigint],
  'accounts_overflow_trim_quantity' : [] | [bigint],
  'change_fee_collector' : [] | [ChangeFeeCollector],
  'max_memo_length' : [] | [number],
  'token_name' : [] | [string],
  'feature_flags' : [] | [FeatureFlags],
}
export interface _SERVICE {
  'add_controller' : ActorMethod<[Principal, Principal], Result>,
  'controlled_canister_status' : ActorMethod<[Principal], Result_1>,
  'delete_controlled_canister' : ActorMethod<[Principal], Result>,
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result_2>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_events' : ActorMethod<[GetEventsArg], Array<Event>>,
  'get_fee_account' : ActorMethod<[[] | [Principal]], Uint8Array | number[]>,
  'get_log_records' : ActorMethod<[bigint, bigint], Logs>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_token_ledger' : ActorMethod<[string], [] | [Principal]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'mint_token_status' : ActorMethod<[string], MintTokenStatus>,
  'remove_controller' : ActorMethod<[Principal, Principal], Result>,
  'start_controlled_canister' : ActorMethod<[Principal], Result>,
  'stop_controlled_canister' : ActorMethod<[Principal], Result>,
  'update_icrc_ledger' : ActorMethod<[Principal, UpgradeArgs_1], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
