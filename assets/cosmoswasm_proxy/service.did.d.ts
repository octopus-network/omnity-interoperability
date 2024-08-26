import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
export interface InitArgs {
  'trigger' : Principal,
  'icp_customs_principal' : Principal,
  'ckbtc_index_principal' : Principal,
}
export type Result = { 'Ok' : Account } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : string };
export interface _SERVICE {
  'get_identity_by_osmosis_account_id' : ActorMethod<[string], Result>,
  'set_trigger_principal' : ActorMethod<[Principal], Result_1>,
  'trigger_transaction' : ActorMethod<[bigint], Result_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
