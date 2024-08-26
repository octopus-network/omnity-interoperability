import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'hub_principal' : Principal,
  'cw_rpc_url' : string,
  'cw_rest_url' : string,
  'cosmoswasm_port_contract_address' : string,
  'chain_id' : string,
}
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export interface _SERVICE {
  'cache_public_key_and_start_timer' : ActorMethod<[], undefined>,
  'redeem' : ActorMethod<[string], Result>,
  'tendermint_address' : ActorMethod<[], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
