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
export type EcdsaCurve = { 'secp256k1' : null };
export interface EcdsaKeyId { 'name' : string, 'curve' : EcdsaCurve }
export type EvmTxType = { 'Eip1559' : null } |
  { 'Legacy' : null };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeeTokenFactor {
  'fee_token' : string,
  'fee_token_factor' : bigint,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'status_code' : number,
}
export interface HttpResponse_1 {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface InitArgs {
  'evm_chain_id' : bigint,
  'hub_principal' : Principal,
  'rpcs' : Array<RpcApi>,
  'evm_tx_type' : EvmTxType,
  'network' : Network,
  'fee_token_id' : string,
  'block_interval_secs' : bigint,
  'chain_id' : string,
  'admins' : Array<Principal>,
  'evm_rpc_canister_addr' : Principal,
  'port_addr' : [] | [string],
}
export interface MetricsStatus {
  'chainkey_addr_balance' : bigint,
  'latest_scan_interval_secs' : bigint,
}
export type MintTokenStatus = { 'Finalized' : { 'tx_hash' : string } } |
  { 'Unknown' : null };
export type Network = { 'mainnet' : null } |
  { 'local' : null } |
  { 'testnet' : null };
export interface PendingDirectiveStatus {
  'seq' : bigint,
  'evm_tx_hash' : [] | [string],
  'error' : [] | [string],
}
export interface PendingTicketStatus {
  'seq' : bigint,
  'evm_tx_hash' : [] | [string],
  'ticket_id' : string,
  'error' : [] | [string],
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export interface RpcApi { 'url' : string, 'headers' : [] | [Array<HttpHeader>] }
export interface StateProfile {
  'next_consume_ticket_seq' : bigint,
  'evm_chain_id' : bigint,
  'finality_blocks' : [] | [bigint],
  'omnity_port_contract' : Uint8Array | number[],
  'evm_gasfee_percent' : bigint,
  'next_consume_directive_seq' : bigint,
  'hub_principal' : Principal,
  'key_id' : EcdsaKeyId,
  'token_contracts' : Array<[string, string]>,
  'next_directive_seq' : bigint,
  'evm_tx_type' : EvmTxType,
  'pubkey' : Uint8Array | number[],
  'key_derivation_path' : Array<Uint8Array | number[]>,
  'omnity_chain_id' : string,
  'tokens' : Array<[string, Token]>,
  'admins' : Array<Principal>,
  'target_chain_factor' : Array<[string, bigint]>,
  'evm_rpc_addr' : Principal,
  'counterparties' : Array<[string, Chain]>,
  'next_ticket_seq' : bigint,
  'rpc_providers' : Array<RpcApi>,
  'chain_state' : ChainState,
  'minimum_response_count' : bigint,
  'total_required_count' : bigint,
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
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'evm_contract' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export interface TransformArgs {
  'context' : Uint8Array | number[],
  'response' : HttpResponse_1,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[string], Result>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_ticket' : ActorMethod<[string], [] | [[bigint, Ticket]]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'insert_pending_hash' : ActorMethod<[string], undefined>,
  'metrics' : ActorMethod<[], MetricsStatus>,
  'mint_token_status' : ActorMethod<[string], MintTokenStatus>,
  'pubkey_and_evm_addr' : ActorMethod<[], [string, string]>,
  'query_directives' : ActorMethod<
    [bigint, bigint],
    Array<[bigint, Directive]>
  >,
  'query_handled_event' : ActorMethod<[string], [] | [string]>,
  'query_hub_tickets' : ActorMethod<[bigint], Array<[bigint, Ticket]>>,
  'query_pending_directive' : ActorMethod<
    [bigint, bigint],
    Array<[bigint, PendingDirectiveStatus]>
  >,
  'query_pending_ticket' : ActorMethod<
    [bigint, bigint],
    Array<[string, PendingTicketStatus]>
  >,
  'query_tickets' : ActorMethod<[bigint, bigint], Array<[bigint, Ticket]>>,
  'resend_directive' : ActorMethod<[bigint], undefined>,
  'resend_ticket' : ActorMethod<[bigint], undefined>,
  'rewrite_tx_hash' : ActorMethod<[string, string], undefined>,
  'route_state' : ActorMethod<[], StateProfile>,
  'set_finality_blocks' : ActorMethod<[bigint], undefined>,
  'set_port_address' : ActorMethod<[string], undefined>,
  'sync_mint_status' : ActorMethod<[string], undefined>,
  'transform' : ActorMethod<[TransformArgs], HttpResponse_1>,
  'update_admins' : ActorMethod<[Array<Principal>], undefined>,
  'update_consume_directive_seq' : ActorMethod<[bigint], undefined>,
  'update_fee_token' : ActorMethod<[string], undefined>,
  'update_rpc_check_rate' : ActorMethod<[bigint, bigint], undefined>,
  'update_rpcs' : ActorMethod<[Array<RpcApi>], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
