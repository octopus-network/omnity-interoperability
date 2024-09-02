import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
export interface BtcTransportRecord {
  'minted_amount' : bigint,
  'block_index' : bigint,
  'utxo' : Utxo,
  'ticket_id' : [] | [string],
}
export interface InitArgs {
  'trigger' : Principal,
  'icp_customs_principal' : Principal,
  'ckbtc_index_principal' : Principal,
}
export interface OutPoint { 'txid' : Uint8Array | number[], 'vout' : number }
export type Result = { 'Ok' : Account } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : string };
export interface Utxo {
  'height' : number,
  'value' : bigint,
  'outpoint' : OutPoint,
}
export interface _SERVICE {
  'get_identity_by_osmosis_account_id' : ActorMethod<[string], Result>,
  'query_btc_transport_info' : ActorMethod<[string], Array<BtcTransportRecord>>,
  'query_status' : ActorMethod<[], [Principal, Principal]>,
  'set_trigger_principal' : ActorMethod<[Principal], Result_1>,
  'trigger_transaction' : ActorMethod<[bigint], Result_1>,
  'trigger_update_balance' : ActorMethod<[string], Result_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
