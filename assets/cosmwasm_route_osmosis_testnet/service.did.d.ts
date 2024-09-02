import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

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
export type Directive = { 'UpdateChain' : Chain } |
  { 'UpdateFee' : Factor } |
  { 'AddToken' : Token } |
  { 'AddChain' : Chain } |
  { 'ToggleChainState' : ToggleState } |
  { 'UpdateToken' : Token };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeeTokenFactor {
  'fee_token' : string,
  'fee_token_factor' : bigint,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface InitArgs {
  'hub_principal' : Principal,
  'cw_rpc_url' : string,
  'cw_rest_url' : string,
  'chain_id' : string,
  'cosmwasm_port_contract_address' : string,
}
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : HttpResponse } |
  { 'Err' : string };
export interface RouteState {
  'hub_principal' : Principal,
  'cw_rpc_url' : string,
  'cw_chain_key_derivation_path' : Array<Uint8Array | number[]>,
  'is_timer_running' : Array<string>,
  'next_directive_seq' : bigint,
  'cw_rest_url' : string,
  'cw_public_key_vec' : [] | [Uint8Array | number[]],
  'chain_id' : string,
  'cw_port_contract_address' : string,
  'processing_tickets' : Array<[bigint, Ticket]>,
  'next_ticket_seq' : bigint,
  'chain_state' : ChainState,
  'processing_directive' : Array<[bigint, Directive]>,
}
export interface TargetChainFactor {
  'target_chain_id' : string,
  'target_chain_factor' : bigint,
}
export interface Ticket {
  'token' : string,
  'action' : TxAction,
  'dst_chain' : string,
  'memo' : [] | [Uint8Array | number[]],
  'ticket_id' : string,
  'sender' : [] | [string],
  'ticket_time' : bigint,
  'ticket_type' : TicketType,
  'src_chain' : string,
  'amount' : string,
  'receiver' : string,
}
export type TicketType = { 'Resubmit' : null } |
  { 'Normal' : null };
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
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export interface UpdateCwSettingsArgs {
  'cw_rpc_url' : [] | [string],
  'cw_rest_url' : [] | [string],
  'cw_port_contract_address' : [] | [string],
}
export interface _SERVICE {
  'cache_public_key_and_start_timer' : ActorMethod<[], undefined>,
  'osmosis_account_id' : ActorMethod<[], Result>,
  'redeem' : ActorMethod<[string], Result>,
  'route_status' : ActorMethod<[], RouteState>,
  'test_execute_directive' : ActorMethod<[string, Directive], Result>,
  'test_http_outcall' : ActorMethod<[string], Result_1>,
  'update_cw_settings' : ActorMethod<[UpdateCwSettingsArgs], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
