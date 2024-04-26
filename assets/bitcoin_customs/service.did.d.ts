import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type BitcoinAddress = { 'OpReturn' : Uint8Array | number[] } |
  { 'p2wsh_v0' : Uint8Array | number[] } |
  { 'p2tr_v1' : Uint8Array | number[] } |
  { 'p2sh' : Uint8Array | number[] } |
  { 'p2wpkh_v0' : Uint8Array | number[] } |
  { 'p2pkh' : Uint8Array | number[] };
export interface BtcChangeOutput { 'value' : bigint, 'vout' : number }
export type BtcNetwork = { 'Mainnet' : null } |
  { 'Regtest' : null } |
  { 'Testnet' : null };
export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'query_stats' : QueryStats,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
  'reserved_cycles' : bigint,
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
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
export interface CustomsInfo {
  'chain_state' : ChainState,
  'min_confirmations' : number,
}
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'reserved_cycles_limit' : bigint,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
export interface Destination {
  'token' : [] | [string],
  'target_chain_id' : string,
  'receiver' : string,
}
export interface EstimateFeeArgs {
  'amount' : [] | [bigint],
  'rune_id' : RuneId,
}
export type Event = {
    'received_utxos' : {
      'is_runes' : boolean,
      'destination' : Destination,
      'utxos' : Array<Utxo>,
    }
  } |
  {
    'sent_transaction' : {
      'fee' : [] | [bigint],
      'txid' : Uint8Array | number[],
      'btc_change_output' : BtcChangeOutput,
      'btc_utxos' : Array<Utxo>,
      'requests' : Array<string>,
      'runes_change_output' : RunesChangeOutput,
      'runes_utxos' : Array<RunesUtxo>,
      'rune_id' : RuneId,
      'submitted_at' : bigint,
    }
  } |
  { 'added_token' : { 'token' : Token, 'rune_id' : RuneId } } |
  {
    'finalized_ticket_request' : {
      'txid' : Uint8Array | number[],
      'balances' : Array<RunesBalance>,
    }
  } |
  { 'accepted_release_token_request' : ReleaseTokenRequest } |
  { 'init' : InitArgs } |
  {
    'updated_runes_balance' : {
      'balance' : RunesBalance,
      'txid' : Uint8Array | number[],
    }
  } |
  { 'upgrade' : UpgradeArgs } |
  { 'added_chain' : Chain } |
  { 'update_next_ticket_seq' : bigint } |
  { 'update_next_directive_seq' : bigint } |
  { 'confirmed_transaction' : { 'txid' : Uint8Array | number[] } } |
  {
    'replaced_transaction' : {
      'fee' : bigint,
      'btc_change_output' : BtcChangeOutput,
      'old_txid' : Uint8Array | number[],
      'new_txid' : Uint8Array | number[],
      'runes_change_output' : RunesChangeOutput,
      'submitted_at' : bigint,
    }
  } |
  { 'accepted_generate_ticket_request' : GenTicketRequest } |
  { 'toggle_chain_state' : ToggleState };
export interface GenTicketRequest {
  'received_at' : bigint,
  'token_id' : string,
  'txid' : Uint8Array | number[],
  'target_chain_id' : string,
  'address' : string,
  'amount' : bigint,
  'receiver' : string,
  'rune_id' : RuneId,
}
export type GenTicketStatus = { 'Invalid' : null } |
  { 'Finalized' : null } |
  { 'Unknown' : null } |
  { 'Pending' : GenTicketRequest };
export interface GenerateTicketArgs {
  'txid' : string,
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
  'rune_id' : string,
}
export type GenerateTicketError = { 'TemporarilyUnavailable' : string } |
  { 'InvalidRuneId' : string } |
  { 'AlreadySubmitted' : null } |
  { 'InvalidTxId' : null } |
  { 'AleardyProcessed' : null } |
  { 'NoNewUtxos' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string };
export interface GetBtcAddressArgs {
  'target_chain_id' : string,
  'receiver' : string,
}
export interface GetEventsArg { 'start' : bigint, 'length' : bigint }
export interface GetGenTicketReqsArgs {
  'max_count' : bigint,
  'start_txid' : [] | [Uint8Array | number[]],
}
export interface InitArgs {
  'hub_principal' : Principal,
  'ecdsa_key_name' : string,
  'runes_oracle_principal' : Principal,
  'max_time_in_queue_nanos' : bigint,
  'chain_id' : string,
  'btc_network' : BtcNetwork,
  'chain_state' : ChainState,
  'min_confirmations' : [] | [number],
}
export interface OutPoint { 'txid' : Uint8Array | number[], 'vout' : number }
export interface QueryStats {
  'response_payload_bytes_total' : bigint,
  'num_instructions_total' : bigint,
  'num_calls_total' : bigint,
  'request_payload_bytes_total' : bigint,
}
export interface RedeemFee { 'bitcoin_fee' : bigint }
export interface ReleaseTokenRequest {
  'received_at' : bigint,
  'ticket_id' : string,
  'address' : BitcoinAddress,
  'amount' : bigint,
  'rune_id' : RuneId,
}
export type ReleaseTokenStatus = { 'Signing' : null } |
  { 'Confirmed' : string } |
  { 'Sending' : string } |
  { 'Unknown' : null } |
  { 'Submitted' : string } |
  { 'Pending' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export type Result_1 = { 'Ok' : Array<Utxo> } |
  { 'Err' : UpdateBtcUtxosErr };
export type Result_2 = { 'Ok' : null } |
  { 'Err' : UpdateRunesBalanceError };
export interface RuneId { 'tx' : number, 'block' : bigint }
export interface RunesBalance {
  'vout' : number,
  'amount' : bigint,
  'rune_id' : RuneId,
}
export interface RunesChangeOutput {
  'value' : bigint,
  'vout' : number,
  'rune_id' : RuneId,
}
export interface RunesUtxo { 'raw' : Utxo, 'runes' : RunesBalance }
export type ToggleAction = { 'Deactivate' : null } |
  { 'Activate' : null };
export interface ToggleState { 'action' : ToggleAction, 'chain_id' : string }
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
  'rune_id' : string,
  'symbol' : string,
}
export type UpdateBtcUtxosErr = { 'TemporarilyUnavailable' : string };
export interface UpdateRunesBalanceArgs {
  'txid' : Uint8Array | number[],
  'balances' : Array<RunesBalance>,
}
export type UpdateRunesBalanceError = { 'SendTicketErr' : string } |
  { 'UtxoNotFound' : null } |
  { 'RequestNotFound' : null } |
  { 'AleardyProcessed' : null } |
  { 'MismatchWithGenTicketReq' : null };
export interface UpgradeArgs {
  'hub_principal' : [] | [Principal],
  'runes_oracle_principal' : [] | [Principal],
  'max_time_in_queue_nanos' : [] | [bigint],
  'chain_state' : [] | [ChainState],
  'min_confirmations' : [] | [number],
}
export interface Utxo {
  'height' : number,
  'value' : bigint,
  'outpoint' : OutPoint,
}
export interface _SERVICE {
  'estimate_redeem_fee' : ActorMethod<[EstimateFeeArgs], RedeemFee>,
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result>,
  'generate_ticket_status' : ActorMethod<[string], GenTicketStatus>,
  'get_btc_address' : ActorMethod<[GetBtcAddressArgs], string>,
  'get_canister_status' : ActorMethod<[], CanisterStatusResponse>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_customs_info' : ActorMethod<[], CustomsInfo>,
  'get_events' : ActorMethod<[GetEventsArg], Array<Event>>,
  'get_main_btc_address' : ActorMethod<[string], string>,
  'get_pending_gen_ticket_requests' : ActorMethod<
    [GetGenTicketReqsArgs],
    Array<GenTicketRequest>
  >,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'release_token_status' : ActorMethod<[string], ReleaseTokenStatus>,
  'update_btc_utxos' : ActorMethod<[], Result_1>,
  'update_runes_balance' : ActorMethod<[UpdateRunesBalanceArgs], Result_2>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
