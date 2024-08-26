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
export type GenerateTicketError = { 'SendTicketErr' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'InsufficientIcp' : { 'provided' : bigint, 'required' : bigint } } |
  { 'InsufficientAllowance' : { 'allowance' : bigint } } |
  { 'TransferIcpFailure' : string } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export interface GenerateTicketOk { 'ticket_id' : string }
export interface GenerateTicketReq {
  'token_id' : string,
  'from_subaccount' : [] | [Uint8Array | number[]],
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface InitArgs {
  'ckbtc_ledger_principal' : Principal,
  'hub_principal' : Principal,
  'chain_id' : string,
}
export type Result = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export interface Token {
  'decimals' : number,
  'token_id' : string,
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'name' : string,
  'symbol' : string,
}
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_token_list' : ActorMethod<[], Array<Token>>,
  'set_ckbtc_token' : ActorMethod<[string], undefined>,
  'set_icp_token' : ActorMethod<[string], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
