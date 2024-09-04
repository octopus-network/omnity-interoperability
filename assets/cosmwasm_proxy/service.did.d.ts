import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
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
  'ckbtc_minter_principal' : Principal,
  'icp_customs_principal' : Principal,
}
export interface TicketRecord {
  'ticket_id' : string,
  'minted_utxos' : Array<MintedUtxo>,
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
  'generate_ticket_from_subaccount' : ActorMethod<[string], Result>,
  'get_btc_mint_address' : ActorMethod<[string], Result>,
  'get_identity_by_osmosis_account_id' : ActorMethod<[string], Result_1>,
  'query_scheduled_osmosis_account_id_list' : ActorMethod<[], Array<string>>,
  'query_settings' : ActorMethod<[], Settings>,
  'query_ticket_records' : ActorMethod<[string], Array<TicketRecord>>,
  'query_utxo_records' : ActorMethod<[string], Array<UtxoRecord>>,
  'trigger_update_balance' : ActorMethod<[string], Result>,
  'update_balance_after_seven_block' : ActorMethod<[string], undefined>,
  'update_settings' : ActorMethod<
    [[] | [Principal], [] | [Principal], [] | [Principal]],
    undefined
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
