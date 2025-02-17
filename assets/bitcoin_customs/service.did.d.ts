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
  'runes_oracles' : Array<Principal>,
  'last_fee_per_vbyte' : BigUint64Array | bigint[],
  'etching_acount_info' : EtchingAccountInfo,
  'hub_principal' : Principal,
  'ecdsa_key_name' : string,
  'next_directive_seq' : bigint,
  'fee_collector_address' : string,
  'icpswap_principal' : [] | [Principal],
  'ecdsa_public_key' : [] | [ECDSAPublicKey],
  'max_time_in_queue_nanos' : bigint,
  'chain_id' : string,
  'rpc_url' : [] | [string],
  'generate_ticket_counter' : bigint,
  'btc_network' : Network,
  'target_chain_factor' : Array<[string, bigint]>,
  'ord_indexer_principal' : [] | [Principal],
  'next_ticket_seq' : bigint,
  'chain_state' : ChainState,
  'min_confirmations' : number,
  'prod_ecdsa_public_key' : [] | [ECDSAPublicKey],
  'release_token_counter' : bigint,
  'fee_token_factor' : [] | [bigint],
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
export interface ECDSAPublicKey {
  'public_key' : Uint8Array | number[],
  'chain_code' : Uint8Array | number[],
}
export interface EstimateFeeArgs {
  'amount' : [] | [bigint],
  'rune_id' : RuneId,
}
export interface EtchingAccountInfo {
  'derive_path' : string,
  'pubkey' : string,
  'address' : string,
}
export interface EtchingArgs {
  'terms' : [] | [OrdinalsTerms],
  'turbo' : boolean,
  'premine' : [] | [bigint],
  'logo' : [] | [LogoParams],
  'rune_name' : string,
  'divisibility' : [] | [number],
  'symbol' : [] | [string],
}
export type EtchingStatus = { 'SendRevealSuccess' : null } |
  { 'SendRevealFailed' : null } |
  { 'SendCommitFailed' : null } |
  { 'TokenAdded' : null } |
  { 'SendCommitSuccess' : null } |
  { 'Final' : null };
export type Event = { 'update_icpswap' : { 'principal' : Principal } } |
  { 'confirmed_generate_ticket_request' : GenTicketRequestV2 } |
  {
    'received_utxos' : {
      'is_runes' : boolean,
      'destination' : Destination,
      'utxos' : Array<Utxo>,
    }
  } |
  { 'added_runes_oracle' : { 'principal' : Principal } } |
  { 'removed_ticket_request' : { 'txid' : Uint8Array | number[] } } |
  { 'update_ord_indexer' : { 'principal' : Principal } } |
  { 'removed_runes_oracle' : { 'principal' : Principal } } |
  { 'updated_fee' : { 'fee' : Factor } } |
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
  { 'accepted_generate_ticket_request_v2' : GenTicketRequestV2 } |
  { 'accepted_generate_ticket_request_v3' : GenTicketRequestV2 } |
  { 'confirmed_transaction' : { 'txid' : Uint8Array | number[] } } |
  { 'upate_fee_collector' : { 'addr' : string } } |
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
  { 'accepted_rune_tx_request' : RuneTxRequest } |
  { 'updated_rpc_url' : { 'rpc_url' : string } } |
  { 'toggle_chain_state' : ToggleState };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeeTokenFactor {
  'fee_token' : string,
  'fee_token_factor' : bigint,
}
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
export interface GenTicketRequestV2 {
  'received_at' : bigint,
  'token_id' : string,
  'new_utxos' : Array<Utxo>,
  'txid' : Uint8Array | number[],
  'target_chain_id' : string,
  'address' : string,
  'amount' : bigint,
  'receiver' : string,
  'rune_id' : RuneId,
}
export type GenTicketStatus = { 'Finalized' : GenTicketRequestV2 } |
  { 'Confirmed' : GenTicketRequestV2 } |
  { 'Unknown' : null } |
  { 'Pending' : GenTicketRequestV2 };
export interface GenerateTicketArgs {
  'txid' : string,
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
  'rune_id' : string,
}
export type GenerateTicketError = { 'SendTicketErr' : string } |
  { 'RpcError' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'AlreadyProcessed' : null } |
  { 'AmountIsZero' : null } |
  { 'InvalidRuneId' : string } |
  { 'AlreadySubmitted' : null } |
  { 'InvalidTxId' : null } |
  { 'NotPayFees' : null } |
  { 'TxNotFoundInMemPool' : null } |
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
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export type IcpChainKeyToken = { 'CKBTC' : null };
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
export interface LogoParams {
  'content_type' : string,
  'content_base64' : string,
}
export type Network = { 'mainnet' : null } |
  { 'regtest' : null } |
  { 'testnet' : null };
export interface OrdinalsTerms {
  'cap' : bigint,
  'height' : [[] | [bigint], [] | [bigint]],
  'offset' : [[] | [bigint], [] | [bigint]],
  'amount' : bigint,
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
export type Result = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export type Result_3 = { 'Ok' : Array<Utxo> } |
  { 'Err' : UpdateBtcUtxosErr };
export type Result_4 = { 'Ok' : null } |
  { 'Err' : UpdateRunesBalanceError };
export interface RuneId { 'tx' : number, 'block' : bigint }
export interface RuneTxRequest {
  'received_at' : bigint,
  'action' : TxAction,
  'ticket_id' : string,
  'address' : BitcoinAddress,
  'amount' : bigint,
  'rune_id' : RuneId,
}
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
export interface SendEtchingInfo {
  'status' : EtchingStatus,
  'script_out_address' : string,
  'err_info' : string,
  'commit_txid' : string,
  'time_at' : bigint,
  'etching_args' : EtchingArgs,
  'reveal_txid' : string,
}
export interface TargetChainFactor {
  'target_chain_id' : string,
  'target_chain_factor' : bigint,
}
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
export interface TokenPrice {
  'name' : string,
  'priceUSD' : number,
  'standard' : string,
  'symbol' : string,
}
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'rune_id' : string,
  'symbol' : string,
}
export interface TransformArgs {
  'context' : Uint8Array | number[],
  'response' : HttpResponse,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'RedeemIcpChainKeyAssets' : IcpChainKeyToken } |
  { 'Transfer' : null };
export type UpdateBtcUtxosErr = { 'TemporarilyUnavailable' : string };
export interface UpdateRunesBalanceArgs {
  'txid' : Uint8Array | number[],
  'balances' : Array<RunesBalance>,
}
export type UpdateRunesBalanceError = { 'RequestNotConfirmed' : null } |
  { 'BalancesIsEmpty' : null } |
  { 'UtxoNotFound' : null } |
  { 'RequestNotFound' : null } |
  { 'AleardyProcessed' : null } |
  { 'MismatchWithGenTicketReq' : null } |
  { 'FinalizeTicketErr' : string };
export interface UpgradeArgs {
  'hub_principal' : [] | [Principal],
  'max_time_in_queue_nanos' : [] | [bigint],
  'chain_state' : [] | [ChainState],
  'min_confirmations' : [] | [number],
}
export interface Utxo {
  'height' : number,
  'value' : bigint,
  'outpoint' : OutPoint,
}
export interface UtxoArgs { 'id' : string, 'index' : number, 'amount' : bigint }
export interface _SERVICE {
  'clear_etching' : ActorMethod<[], undefined>,
  'estimate_etching_fee' : ActorMethod<
    [bigint, string, [] | [LogoParams]],
    Result
  >,
  'estimate_redeem_fee' : ActorMethod<[EstimateFeeArgs], RedeemFee>,
  'etching' : ActorMethod<[bigint, EtchingArgs], Result_1>,
  'etching_reveal' : ActorMethod<[string], undefined>,
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result_2>,
  'generate_ticket_status' : ActorMethod<[string], GenTicketStatus>,
  'get_btc_address' : ActorMethod<[GetBtcAddressArgs], string>,
  'get_btc_icp_price' : ActorMethod<[], [[] | [TokenPrice], [] | [TokenPrice]]>,
  'get_canister_status' : ActorMethod<[], CanisterStatusResponse>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_customs_info' : ActorMethod<[], CustomsInfo>,
  'get_etching' : ActorMethod<[string], [] | [SendEtchingInfo]>,
  'get_etching_by_user' : ActorMethod<[Principal], Array<SendEtchingInfo>>,
  'get_etching_fee_utxos' : ActorMethod<[], Array<UtxoArgs>>,
  'get_events' : ActorMethod<[GetEventsArg], Array<Event>>,
  'get_main_btc_address' : ActorMethod<[string], string>,
  'get_pending_gen_ticket_requests' : ActorMethod<
    [GetGenTicketReqsArgs],
    Array<GenTicketRequestV2>
  >,
  'get_platform_fee' : ActorMethod<[string], [[] | [bigint], [] | [string]]>,
  'get_runes_oracles' : ActorMethod<[], Array<Principal>>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'query_bitcoin_balance' : ActorMethod<[string, number], bigint>,
  'release_token_status' : ActorMethod<[string], ReleaseTokenStatus>,
  'remove_error_ticket' : ActorMethod<[string], undefined>,
  'remove_runes_oracle' : ActorMethod<[Principal], undefined>,
  'set_fee_collector' : ActorMethod<[string], undefined>,
  'set_icpswap' : ActorMethod<[Principal], undefined>,
  'set_ord_indexer' : ActorMethod<[Principal], undefined>,
  'set_runes_oracle' : ActorMethod<[Principal], undefined>,
  'transform' : ActorMethod<[TransformArgs], HttpResponse>,
  'update_btc_utxos' : ActorMethod<[], Result_3>,
  'update_fees' : ActorMethod<[Array<UtxoArgs>], undefined>,
  'update_rpc_url' : ActorMethod<[string], undefined>,
  'update_runes_balance' : ActorMethod<[UpdateRunesBalanceArgs], Result_4>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
