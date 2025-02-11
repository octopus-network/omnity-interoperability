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
export interface GenerateTicketArgs {
  'token_id' : string,
  'sender' : string,
  'target_chain_id' : string,
  'tx_hash' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export type IcpChainKeyToken = { 'CKBTC' : null };
export interface InitArgs {
  'hub_principal' : Principal,
  'admins' : Array<Principal>,
}
export type MintTokenStatus = { 'Finalized' : { 'tx_hash' : string } } |
  { 'Unknown' : null };
export interface PendingDirectiveStatus {
  'seq' : bigint,
  'ton_tx_hash' : [] | [string],
  'error' : [] | [string],
}
export interface PendingTicketStatus {
  'seq' : bigint,
  'pending_time' : bigint,
  'ticket_id' : string,
  'ton_tx_hash' : [] | [string],
  'error' : [] | [string],
}
export type Result = { 'Ok' : Ticket } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : number } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : [] | [string] } |
  { 'Err' : string };
export interface StateProfile {
  'next_consume_ticket_seq' : bigint,
  'next_consume_directive_seq' : bigint,
  'hub_principal' : Principal,
  'last_success_seqno' : number,
  'token_contracts' : Array<[string, string]>,
  'next_directive_seq' : bigint,
  'pubkey' : Uint8Array | number[],
  'omnity_chain_id' : string,
  'tokens' : Array<[string, Token]>,
  'admins' : Array<Principal>,
  'target_chain_factor' : Array<[string, bigint]>,
  'counterparties' : Array<[string, Chain]>,
  'next_ticket_seq' : bigint,
  'fee_token_factor' : [] | [bigint],
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
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'ton_contract' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export interface TransformArgs {
  'context' : Uint8Array | number[],
  'response' : HttpResponse,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'RedeemIcpChainKeyAssets' : IcpChainKeyToken } |
  { 'Transfer' : null };
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_fee' : ActorMethod<[string], [[] | [bigint], string]>,
  'get_ticket' : ActorMethod<[string], [] | [[bigint, Ticket]]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'mint_token_status' : ActorMethod<[string], MintTokenStatus>,
  'pubkey_and_ton_addr' : ActorMethod<[], [string, string]>,
  'query_account_seqno' : ActorMethod<[string], Result_1>,
  'query_directives' : ActorMethod<
    [bigint, bigint],
    Array<[bigint, Directive]>
  >,
  'query_pending_directive' : ActorMethod<
    [bigint, bigint],
    Array<[bigint, PendingDirectiveStatus]>
  >,
  'query_pending_ticket' : ActorMethod<
    [bigint, bigint],
    Array<[bigint, PendingTicketStatus]>
  >,
  'query_tickets' : ActorMethod<[bigint, bigint], Array<[bigint, Ticket]>>,
  'resend_ticket' : ActorMethod<[bigint], Result_2>,
  'route_state' : ActorMethod<[], StateProfile>,
  'set_token_master' : ActorMethod<[string, string], undefined>,
  'transform' : ActorMethod<[TransformArgs], HttpResponse>,
  'update_admins' : ActorMethod<[Array<Principal>], undefined>,
  'update_consume_directive_seq' : ActorMethod<[bigint], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
