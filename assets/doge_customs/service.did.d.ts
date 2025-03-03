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
export type CustomsError = { 'SendTicketErr' : string } |
  { 'RpcError' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'HttpOutCallError' : [string, string, string] } |
  { 'AlreadyProcessed' : null } |
  { 'HttpStatusError' : [bigint, string, string] } |
  { 'OrdTxError' : string } |
  { 'NotBridgeTx' : null } |
  { 'AmountIsZero' : null } |
  { 'InvalidRuneId' : string } |
  { 'InvalidArgs' : string } |
  { 'AlreadySubmitted' : null } |
  { 'InvalidTxId' : null } |
  { 'NotPayFees' : null } |
  { 'CallError' : [Principal, string, string] } |
  { 'TxNotFoundInMemPool' : null } |
  { 'Unknown' : null } |
  { 'InvalidTxReceiver' : null } |
  { 'UnsupportedChainId' : string } |
  { 'ECDSAPublicKeyNotFound' : null } |
  { 'HttpOutExceedRetryLimit' : null } |
  { 'DepositUtxoNotFound' : [string, Destination] } |
  { 'UnsupportedToken' : string } |
  { 'CustomError' : string };
export interface Destination {
  'token' : [] | [string],
  'target_chain_id' : string,
  'receiver' : string,
}
export interface EcdsaPublicKeyResponse {
  'public_key' : Uint8Array | number[],
  'chain_code' : Uint8Array | number[],
}
export interface GenerateTicketArgs {
  'token_id' : string,
  'target_chain_id' : string,
  'receiver' : string,
}
export interface GenerateTicketWithTxidArgs {
  'token_id' : string,
  'txid' : string,
  'target_chain_id' : string,
  'receiver' : string,
}
export interface InitArgs {
  'fee_token' : string,
  'hub_principal' : Principal,
  'chain_id' : string,
  'default_doge_rpc_config' : RpcConfig,
  'admins' : Array<Principal>,
}
export interface LockTicketRequest {
  'received_at' : bigint,
  'transaction_hex' : string,
  'token_id' : string,
  'txid' : Uint8Array | number[],
  'target_chain_id' : string,
  'amount' : string,
  'receiver' : string,
}
export interface MultiRpcConfig {
  'rpc_list' : Array<RpcConfig>,
  'minimum_response_count' : number,
}
export type ReleaseTokenStatus = { 'Signing' : null } |
  { 'Confirmed' : string } |
  { 'Sending' : string } |
  { 'Unknown' : null } |
  { 'Submitted' : string } |
  { 'Pending' : null };
export type Result = { 'Ok' : Array<string> } |
  { 'Err' : CustomsError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : CustomsError };
export type Result_2 = { 'Ok' : string } |
  { 'Err' : CustomsError };
export type Result_3 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : bigint } |
  { 'Err' : CustomsError };
export interface RpcConfig { 'url' : string, 'api_key' : [] | [string] }
export interface SendTicketResult {
  'txid' : Uint8Array | number[],
  'success' : boolean,
  'time_at' : bigint,
}
export interface StateProfile {
  'next_consume_ticket_seq' : bigint,
  'fee_token' : string,
  'hub_principal' : Principal,
  'ecdsa_key_name' : string,
  'doge_chain' : number,
  'next_directive_seq' : bigint,
  'doge_fee_rate' : [] | [bigint],
  'deposited_utxo' : Array<[Utxo, Destination]>,
  'fee_collector' : string,
  'ecdsa_public_key' : [] | [EcdsaPublicKeyResponse],
  'chain_id' : string,
  'pending_lock_ticket_requests' : Array<[string, LockTicketRequest]>,
  'tokens' : Array<[string, Token]>,
  'admins' : Array<Principal>,
  'target_chain_factor' : Array<[string, bigint]>,
  'multi_rpc_config' : MultiRpcConfig,
  'counterparties' : Array<[string, Chain]>,
  'min_deposit_amount' : bigint,
  'next_ticket_seq' : bigint,
  'chain_state' : ChainState,
  'min_confirmations' : number,
  'tatum_rpc_config' : RpcConfig,
  'fee_payment_utxo' : Array<Utxo>,
  'flight_unlock_ticket_map' : Array<[bigint, SendTicketResult]>,
  'fee_token_factor' : [] | [bigint],
}
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
  'symbol' : string,
}
export interface Utxo {
  'value' : bigint,
  'txid' : Uint8Array | number[],
  'vout' : number,
}
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result>,
  'generate_ticket_by_txid' : ActorMethod<
    [GenerateTicketWithTxidArgs],
    Result_1
  >,
  'get_deposit_address' : ActorMethod<[string, string], Result_2>,
  'get_fee_payment_address' : ActorMethod<[], Result_2>,
  'get_finalized_lock_ticket_txids' : ActorMethod<[], Array<string>>,
  'get_finalized_unlock_ticket_results' : ActorMethod<
    [],
    Array<SendTicketResult>
  >,
  'get_platform_fee' : ActorMethod<[string], [[] | [bigint], [] | [string]]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'init_ecdsa_public_key' : ActorMethod<[], Result_1>,
  'pending_unlock_tickets' : ActorMethod<[bigint], string>,
  'query_finalized_lock_tickets' : ActorMethod<
    [string],
    [] | [LockTicketRequest]
  >,
  'query_state' : ActorMethod<[], StateProfile>,
  'release_token_status' : ActorMethod<[string], ReleaseTokenStatus>,
  'resend_unlock_ticket' : ActorMethod<[bigint, [] | [bigint]], Result_3>,
  'save_utxo_for_payment_address' : ActorMethod<[string], Result_4>,
  'set_default_doge_rpc_config' : ActorMethod<
    [string, [] | [string]],
    undefined
  >,
  'set_fee_collector' : ActorMethod<[string], undefined>,
  'set_min_deposit_amount' : ActorMethod<[bigint], undefined>,
  'set_multi_rpc_config' : ActorMethod<[MultiRpcConfig], undefined>,
  'set_tatum_api_config' : ActorMethod<[string, [] | [string]], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
