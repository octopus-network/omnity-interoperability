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
export type CustomArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
export type GenTicketStatus = { 'Finalized' : GenerateTicketArgs } |
  { 'Unknown' : null };
export interface GenerateTicketArgs {
  'signature' : string,
  'token_id' : string,
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export type GenerateTicketError = { 'SendTicketErr' : string } |
  { 'RpcError' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'AlreadyProcessed' : null } |
  { 'DecodeTxError' : string } |
  { 'MismatchWithGenTicketReq' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string };
export interface InitArgs {
  'hub_principal' : Principal,
  'rpc_list' : Array<string>,
  'sol_canister' : Principal,
  'chain_id' : string,
  'port_program_id' : string,
  'schnorr_key_name' : string,
  'chain_state' : ChainState,
  'forward' : [] | [string],
  'min_response_count' : number,
}
export type ReleaseTokenStatus = { 'Finalized' : string } |
  { 'Unknown' : null } |
  { 'Submitted' : string } |
  { 'Pending' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : string };
export interface Token {
  'decimals' : number,
  'token_id' : string,
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'name' : string,
  'symbol' : string,
}
export interface UpgradeArgs {
  'hub_principal' : [] | [Principal],
  'sol_canister' : [] | [Principal],
  'chain_id' : [] | [string],
  'schnorr_key_name' : [] | [string],
  'chain_state' : [] | [ChainState],
}
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result>,
  'generate_ticket_status' : ActorMethod<[string], GenTicketStatus>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_payer_address' : ActorMethod<[], string>,
  'get_token_list' : ActorMethod<[], Array<Token>>,
  'redeem_from_fee_address' : ActorMethod<[string, bigint], Result_1>,
  'release_token_status' : ActorMethod<[string], ReleaseTokenStatus>,
  'resubmit_release_token_tx' : ActorMethod<[string], Result_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
