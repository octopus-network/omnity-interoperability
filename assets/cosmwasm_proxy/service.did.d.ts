import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
export interface InitArgs {
  'ckbtc_ledger_principal' : Principal,
  'token_id' : string,
  'ckbtc_minter_principal' : Principal,
  'icp_customs_principal' : Principal,
  'target_chain_id' : string,
}
export interface MintedUtxo {
  'minted_amount' : bigint,
  'block_index' : bigint,
  'utxo' : Utxo,
}
export interface OutPoint { 'txid' : Uint8Array | number[], 'vout' : number }
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : Account } |
  { 'Err' : string };
export interface Settings {
  'ckbtc_ledger_principal' : Principal,
  'update_balances_jobs' : Array<UpdateBalanceJob>,
  'token_id' : string,
  'is_timer_running' : Array<string>,
  'ckbtc_minter_principal' : Principal,
  'icp_customs_principal' : Principal,
  'target_chain_id' : string,
}
export interface TicketRecord {
  'ticket_id' : string,
  'minted_utxos' : Array<MintedUtxo>,
}
export interface UpdateBalanceJob {
  'ticket_memo' : [] | [string],
  'osmosis_account_id' : string,
  'failed_times' : number,
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
  'generate_ticket_from_subaccount' : ActorMethod<
    [string, [] | [string]],
    Result
  >,
  'get_btc_mint_address' : ActorMethod<[string], Result>,
  'get_identity_by_osmosis_account_id' : ActorMethod<[string], Result_1>,
  'query_scheduled_osmosis_account_id_list' : ActorMethod<[], Array<string>>,
  'query_settings' : ActorMethod<[], Settings>,
  'query_ticket_records' : ActorMethod<[string], Array<TicketRecord>>,
  'query_utxo_records' : ActorMethod<[string], Array<UtxoRecord>>,
  'trigger_update_balance' : ActorMethod<[string, [] | [string]], Result>,
  'update_balance_after_finalization' : ActorMethod<
    [string, [] | [string]],
    undefined
  >,
  'update_settings' : ActorMethod<
    [
      [] | [Principal],
      [] | [Principal],
      [] | [Principal],
      [] | [string],
      [] | [string],
    ],
    undefined
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
