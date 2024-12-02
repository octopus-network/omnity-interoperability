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
export interface CustomsState {
  'ckbtc_ledger_principal' : Principal,
  'hub_principal' : Principal,
  'is_timer_running' : boolean,
  'next_directive_seq' : bigint,
  'ckbtc_minter_principal' : [] | [Principal],
  'icp_token_id' : [] | [string],
  'chain_id' : string,
  'next_ticket_seq' : bigint,
  'ckbtc_token_id' : [] | [string],
}
export type GenerateTicketError = { 'SendTicketErr' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'InsufficientIcp' : { 'provided' : bigint, 'required' : bigint } } |
  { 'InsufficientAllowance' : { 'allowance' : bigint } } |
  { 'TransferIcpFailure' : string } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string } |
  { 'CustomError' : string } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export interface GenerateTicketOk { 'ticket_id' : string }
export interface GenerateTicketReq {
  'token_id' : string,
  'memo' : [] | [string],
  'from_subaccount' : [] | [Uint8Array | number[]],
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export type IcpChainKeyToken = { 'CKBTC' : null };
export interface InitArgs {
  'ckbtc_ledger_principal' : Principal,
  'hub_principal' : Principal,
  'chain_id' : string,
}
export type MintTokenStatus = { 'Finalized' : { 'tx_hash' : string } } |
  { 'Unknown' : null };
export type Result = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type Result_1 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : [bigint, bigint] } |
  { 'Err' : string };
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
  { 'RedeemIcpChainKeyAssets' : IcpChainKeyToken } |
  { 'Transfer' : null };
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result>,
  'get_account_identifier' : ActorMethod<[Principal], Uint8Array | number[]>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_state' : ActorMethod<[], CustomsState>,
  'get_token_list' : ActorMethod<[], Array<Token>>,
  'handle_ticket' : ActorMethod<[bigint], Result_1>,
  'mint_token_status' : ActorMethod<[string], MintTokenStatus>,
  'query_hub_tickets' : ActorMethod<[bigint, bigint], Array<[bigint, Ticket]>>,
  'refund_icp' : ActorMethod<[Principal], Result_2>,
  'set_ckbtc_token' : ActorMethod<[string], undefined>,
  'set_icp_token' : ActorMethod<[string], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
