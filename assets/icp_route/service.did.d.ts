import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
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
export interface FeatureFlags { 'icrc2' : boolean }
export type GenerateTicketError = {
    'InsufficientRedeemFee' : { 'provided' : bigint, 'required' : bigint }
  } |
  { 'SendTicketErr' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'InsufficientAllowance' : { 'allowance' : bigint } } |
  { 'TransferFailure' : string } |
  { 'UnsupportedAction' : string } |
  { 'RedeemFeeNotSet' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export interface GenerateTicketOk { 'ticket_id' : string }
export interface GenerateTicketReq {
  'action' : TxAction,
  'token_id' : string,
  'from_subaccount' : [] | [Uint8Array | number[]],
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export type IcpChainKeyToken = { 'CKBTC' : null };
export interface InitArgs {
  'hub_principal' : Principal,
  'chain_id' : string,
  'chain_state' : ChainState,
}
export type MetadataValue = { 'Int' : bigint } |
  { 'Nat' : bigint } |
  { 'Blob' : Uint8Array | number[] } |
  { 'Text' : string };
export type MintTokenStatus = { 'Finalized' : { 'block_index' : bigint } } |
  { 'Unknown' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type Result_2 = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export type RouteArg = { 'Upgrade' : [] | [UpgradeArgs_1] } |
  { 'Init' : InitArgs };
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
export interface TokenResp {
  'principal' : [] | [Principal],
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'RedeemIcpChainKeyAssets' : IcpChainKeyToken } |
  { 'Transfer' : null };
export interface UpgradeArgs {
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
export interface UpgradeArgs_1 {
  'hub_principal' : [] | [Principal],
  'chain_id' : [] | [string],
  'chain_state' : [] | [ChainState],
}
export interface _SERVICE {
  'collect_ledger_fee' : ActorMethod<
    [Principal, [] | [bigint], Account],
    Result
  >,
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result_1>,
  'generate_ticket_v2' : ActorMethod<[GenerateTicketReq], Result_1>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_fee_account' : ActorMethod<[[] | [Principal]], Uint8Array | number[]>,
  'get_readable_fee_account' : ActorMethod<[[] | [Principal]], string>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_token_ledger' : ActorMethod<[string], [] | [Principal]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'mint_token_status' : ActorMethod<[string], MintTokenStatus>,
  'query_failed_tickets' : ActorMethod<[], Array<Ticket>>,
  'remove_controller' : ActorMethod<[Principal, Principal], Result>,
  'resend_tickets' : ActorMethod<[], Result_2>,
  'update_icrc_ledger' : ActorMethod<[Principal, UpgradeArgs], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
