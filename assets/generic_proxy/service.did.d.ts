import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type Errors = { 'AccountIdParseError' : [string, string] } |
  { 'CkBtcUpdateBalanceError' : [string, string] } |
  { 'CallError' : [string, Principal, string, string] } |
  { 'CanisterCallError' : [string, string, string, string] } |
  { 'NatConversionError' : string } |
  { 'CustomError' : string };
export interface InitArgs {
  'ckbtc_ledger_principal' : Principal,
  'token_id' : string,
  'ckbtc_minter_principal' : Principal,
  'icp_customs_principal' : Principal,
}
export interface MintedUtxo {
  'minted_amount' : bigint,
  'block_index' : bigint,
  'utxo' : Utxo,
}
export interface OmnityAccount { 'chain_id' : string, 'account' : string }
export interface OutPoint { 'txid' : Uint8Array | number[], 'vout' : number }
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : string } |
  { 'Err' : Errors };
export type Result_2 = { 'Ok' : State } |
  { 'Err' : Errors };
export interface State {
  'ckbtc_ledger_principal' : Principal,
  'update_balances_jobs' : Array<UpdateBalanceJob>,
  'token_id' : string,
  'is_timer_running' : Array<string>,
  'ckbtc_minter_principal' : Principal,
  'icp_customs_principal' : Principal,
}
export interface TicketRecord {
  'ticket_id' : string,
  'minted_utxos' : Array<MintedUtxo>,
}
export interface UpdateBalanceJob {
  'failed_times' : number,
  'omnity_account' : OmnityAccount,
  'next_execute_time' : bigint,
}
export interface Utxo {
  'height' : number,
  'value' : bigint,
  'outpoint' : OutPoint,
}
export interface UtxoRecord {
  'ticket_id' : [] | [string],
  'minted_utxo' : MintedUtxo,
}
export interface _SERVICE {
  'generate_ticket_from_subaccount' : ActorMethod<[OmnityAccount], Result>,
  'query_btc_mint_address_by_omnity_account' : ActorMethod<
    [OmnityAccount],
    Result_1
  >,
  'query_state' : ActorMethod<[], Result_2>,
  'query_ticket_records' : ActorMethod<[OmnityAccount], Array<TicketRecord>>,
  'query_utxo_records' : ActorMethod<[OmnityAccount], Array<UtxoRecord>>,
  'trigger_update_balance' : ActorMethod<[OmnityAccount], Result>,
  'update_balance_after_finalization' : ActorMethod<[OmnityAccount], undefined>,
  'update_settings' : ActorMethod<
    [[] | [Principal], [] | [Principal], [] | [Principal], [] | [string]],
    undefined
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
