import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface CallError { 'method' : string, 'reason' : Reason }
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
  'memo' : [] | [string],
  'sender' : string,
  'target_chain_id' : string,
  'digest' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface InitArgs {
  'admin' : Principal,
  'hub_principal' : Principal,
  'gas_budget' : [] | [bigint],
  'fee_account' : string,
  'rpc_provider' : [] | [Provider],
  'chain_id' : string,
  'schnorr_key_name' : [] | [string],
  'chain_state' : ChainState,
  'nodes_in_subnet' : [] | [number],
}
export type KeyType = { 'Native' : Uint8Array | number[] } |
  { 'ChainKey' : null };
export interface MintTokenRequest {
  'status' : TxStatus,
  'object' : [] | [string],
  'token_id' : string,
  'recipient' : string,
  'ticket_id' : string,
  'digest' : [] | [string],
  'amount' : bigint,
  'retry' : bigint,
}
export interface MultiRpcConfig {
  'rpc_list' : Array<string>,
  'minimum_response_count' : number,
}
export type Permission = { 'Update' : null } |
  { 'Query' : null };
export type Provider = { 'Mainnet' : null } |
  { 'Custom' : [string, string] } |
  { 'Testnet' : null } |
  { 'Devnet' : null } |
  { 'Localnet' : null };
export type Reason = { 'QueueIsFull' : null } |
  { 'CanisterError' : string } |
  { 'OutOfCycles' : null } |
  { 'Rejected' : string } |
  { 'TxError' : string };
export type Result = { 'Ok' : string } |
  { 'Err' : RpcError };
export type Result_1 = { 'Ok' : boolean } |
  { 'Err' : RpcError };
export type Result_10 = { 'Ok' : null } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type Result_3 = { 'Ok' : bigint } |
  { 'Err' : RpcError };
export type Result_4 = { 'Ok' : bigint } |
  { 'Err' : RpcError };
export type Result_5 = { 'Ok' : MintTokenRequest } |
  { 'Err' : CallError };
export type Result_6 = { 'Ok' : TxStatus } |
  { 'Err' : CallError };
export type Result_7 = { 'Ok' : [] | [string] } |
  { 'Err' : CallError };
export type Result_8 = { 'Ok' : null } |
  { 'Err' : RpcError };
export type Result_9 = { 'Ok' : string } |
  { 'Err' : string };
export type RouteArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
export type RpcError = { 'Text' : string } |
  { 'ParseError' : string } |
  {
    'RpcResponseError' : {
      'code' : bigint,
      'data' : [] | [string],
      'message' : string,
    }
  } |
  { 'RpcRequestError' : string };
export interface Seqs {
  'next_directive_seq' : bigint,
  'next_ticket_seq' : bigint,
}
export type SnorKeyType = { 'Native' : null } |
  { 'ChainKey' : null };
export interface SuiPortAction {
  'package' : string,
  'upgrade_cap' : string,
  'ticket_table' : string,
  'port_owner_cap' : string,
  'functions' : Array<string>,
  'module' : string,
}
export interface SuiRouteConfig {
  'sui_port_action' : SuiPortAction,
  'admin' : Principal,
  'hub_principal' : Principal,
  'caller_perms' : Array<[string, Permission]>,
  'active_tasks' : Array<TaskType>,
  'gas_budget' : bigint,
  'enable_debug' : boolean,
  'fee_account' : string,
  'seqs' : Seqs,
  'rpc_provider' : Provider,
  'chain_id' : string,
  'schnorr_key_name' : string,
  'target_chain_factor' : Array<[string, bigint]>,
  'multi_rpc_config' : MultiRpcConfig,
  'key_type' : KeyType,
  'chain_state' : ChainState,
  'forward' : [] | [string],
  'nodes_in_subnet' : number,
  'fee_token_factor' : [] | [bigint],
}
export interface SuiToken {
  'treasury_cap' : string,
  'metadata' : string,
  'package' : string,
  'upgrade_cap' : string,
  'functions' : Array<string>,
  'module' : string,
  'type_tag' : string,
}
export type TaskType = { 'GetTickets' : null } |
  { 'ClearTicket' : null } |
  { 'BurnToken' : null } |
  { 'GetDirectives' : null } |
  { 'MintToken' : null } |
  { 'UpdateToken' : null };
export interface Token {
  'decimals' : number,
  'token_id' : string,
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'name' : string,
  'symbol' : string,
}
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export type TxStatus = { 'New' : null } |
  { 'Finalized' : null } |
  { 'TxFailed' : { 'e' : string } } |
  { 'Pending' : null };
export type UpdateType = { 'Symbol' : string } |
  { 'Icon' : string } |
  { 'Name' : string } |
  { 'Description' : string };
export interface UpgradeArgs {
  'admin' : [] | [Principal],
  'hub_principal' : [] | [Principal],
  'gas_budget' : [] | [bigint],
  'fee_account' : [] | [string],
  'rpc_provider' : [] | [Provider],
  'chain_id' : [] | [string],
  'schnorr_key_name' : [] | [string],
  'chain_state' : [] | [ChainState],
  'nodes_in_subnet' : [] | [number],
}
export interface _SERVICE {
  'add_token' : ActorMethod<[Token], [] | [Token]>,
  'burn_token' : ActorMethod<[string, string], Result>,
  'check_object_exists' : ActorMethod<[string, string], Result_1>,
  'create_ticket_table' : ActorMethod<[string], Result>,
  'drop_ticket_table' : ActorMethod<[], Result>,
  'fetch_coin' : ActorMethod<[string, [] | [string], bigint], Result>,
  'forward' : ActorMethod<[], [] | [string]>,
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result_2>,
  'get_balance' : ActorMethod<[string, [] | [string]], Result_3>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_coins' : ActorMethod<[string, [] | [string]], Result>,
  'get_events' : ActorMethod<[string], Result>,
  'get_fee_account' : ActorMethod<[], string>,
  'get_gas_budget' : ActorMethod<[], bigint>,
  'get_gas_price' : ActorMethod<[], Result_4>,
  'get_object' : ActorMethod<[string], Result>,
  'get_owner_objects' : ActorMethod<[string, [] | [string]], Result>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_route_config' : ActorMethod<[], SuiRouteConfig>,
  'get_token' : ActorMethod<[string], [] | [Token]>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'get_transaction_block' : ActorMethod<[string], Result>,
  'merge_coin' : ActorMethod<[string, Array<string>], Result>,
  'mint_to_with_ticket' : ActorMethod<[string, string, string, bigint], Result>,
  'mint_token' : ActorMethod<[string, string, bigint], Result>,
  'mint_token_req' : ActorMethod<[string], Result_5>,
  'mint_token_reqs' : ActorMethod<[bigint, bigint], Array<MintTokenRequest>>,
  'mint_token_status' : ActorMethod<[string], Result_6>,
  'mint_token_tx_hash' : ActorMethod<[string], Result_7>,
  'parse_redeem_events' : ActorMethod<[string], Result_8>,
  'remove_ticket_from_port' : ActorMethod<[string], Result>,
  'rpc_provider' : ActorMethod<[], Provider>,
  'split_coin' : ActorMethod<[string, bigint, string], Result>,
  'sui_port_action' : ActorMethod<[], SuiPortAction>,
  'sui_route_address' : ActorMethod<[SnorKeyType], Result_9>,
  'sui_sign' : ActorMethod<[Uint8Array | number[], SnorKeyType], Result_9>,
  'sui_token' : ActorMethod<[string], [] | [SuiToken]>,
  'transfer_objects' : ActorMethod<[string, Array<string>], Result>,
  'transfer_sui' : ActorMethod<[string, bigint], Result>,
  'update_gas_budget' : ActorMethod<[bigint], undefined>,
  'update_mint_token_req' : ActorMethod<[MintTokenRequest], Result_5>,
  'update_rpc_provider' : ActorMethod<[Provider], undefined>,
  'update_sui_port_action' : ActorMethod<[SuiPortAction], undefined>,
  'update_sui_token' : ActorMethod<[string, SuiToken], Result_10>,
  'update_token_meta' : ActorMethod<[string, UpdateType], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
