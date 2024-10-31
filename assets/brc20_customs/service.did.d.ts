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
export interface EcdsaPublicKeyResponse {
  'public_key' : Uint8Array | number[],
  'chain_code' : Uint8Array | number[],
}
export interface GenerateTicketArgs {
  'token_id' : string,
  'txid' : string,
  'target_chain_id' : string,
  'amount' : string,
  'receiver' : string,
}
export type GenerateTicketError = { 'SendTicketErr' : string } |
  { 'RpcError' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'AlreadyProcessed' : null } |
  { 'OrdTxError' : string } |
  { 'NotBridgeTx' : null } |
  { 'AmountIsZero' : null } |
  { 'InvalidRuneId' : string } |
  { 'InvalidArgs' : string } |
  { 'AlreadySubmitted' : null } |
  { 'InvalidTxId' : null } |
  { 'NotPayFees' : null } |
  { 'TxNotFoundInMemPool' : null } |
  { 'Unknown' : null } |
  { 'NoNewUtxos' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string };
export interface InitArgs {
  'hub_principal' : Principal,
  'network' : Network_1,
  'chain_id' : string,
  'admins' : Array<Principal>,
  'indexer_principal' : Principal,
}
export interface LockTicketRequest {
  'received_at' : bigint,
  'ticker' : string,
  'token_id' : string,
  'txid' : Uint8Array | number[],
  'target_chain_id' : string,
  'amount' : string,
  'receiver' : string,
}
export type Network = { 'mainnet' : null } |
  { 'regtest' : null } |
  { 'testnet' : null };
export type Network_1 = { 'mainnet' : null } |
  { 'local' : null } |
  { 'testnet' : null };
export type ReleaseTokenStatus = { 'Signing' : null } |
  { 'Confirmed' : string } |
  { 'Sending' : string } |
  { 'Unknown' : null } |
  { 'Submitted' : string } |
  { 'Pending' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export interface StateProfile {
  'next_consume_ticket_seq' : bigint,
  'next_consume_directive_seq' : bigint,
  'hub_principal' : Principal,
  'ecdsa_key_name' : string,
  'deposit_addr' : [] | [string],
  'next_directive_seq' : bigint,
  'fee_collector' : string,
  'ecdsa_public_key' : [] | [EcdsaPublicKeyResponse],
  'chain_id' : string,
  'pending_lock_ticket_requests' : Array<
    [Uint8Array | number[], LockTicketRequest]
  >,
  'tokens' : Array<[string, Token]>,
  'btc_network' : Network,
  'admins' : Array<Principal>,
  'target_chain_factor' : Array<[string, bigint]>,
  'counterparties' : Array<[string, Chain]>,
  'next_ticket_seq' : bigint,
  'chain_state' : ChainState,
  'min_confirmations' : number,
  'indexer_principal' : Principal,
  'deposit_pubkey' : [] | [string],
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
export interface UtxoArgs { 'id' : string, 'index' : number, 'amount' : bigint }
export interface _SERVICE {
  'brc20_state' : ActorMethod<[], StateProfile>,
  'finalize_lock_request' : ActorMethod<[string], undefined>,
  'finalized_unlock_tickets' : ActorMethod<[bigint], string>,
  'generate_deposit_addr' : ActorMethod<[], [string, string]>,
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result>,
  'get_deposit_addr' : ActorMethod<[], [string, string]>,
  'get_platform_fee' : ActorMethod<[string], [[] | [bigint], [] | [string]]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'pending_unlock_tickets' : ActorMethod<[bigint], string>,
  'query_finalized_lock_tickets' : ActorMethod<
    [Uint8Array | number[]],
    [] | [LockTicketRequest]
  >,
  'release_token_status' : ActorMethod<[string], ReleaseTokenStatus>,
  'resend_unlock_ticket' : ActorMethod<[bigint, bigint], string>,
  'set_fee_collector' : ActorMethod<[string], undefined>,
  'update_fees' : ActorMethod<[Array<UtxoArgs>], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
