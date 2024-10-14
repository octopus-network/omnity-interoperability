import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

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
export type CustomToBitcoinError = { 'ArgumentError' : string } |
  { 'SignFailed' : string } |
  { 'BuildTransactionFailed' : string } |
  { 'InsufficientFunds' : null };
export interface ECDSAPublicKey {
  'public_key' : Uint8Array | number[],
  'chain_code' : Uint8Array | number[],
}
export interface GenerateTicketArgs {
  'token_id' : string,
  'txid' : string,
  'target_chain_id' : string,
  'amount' : string,
  'receiver' : string,
}
export type GenerateTicketError = { 'SendTicketErr' : string } |
  { 'RpcError' : string } |
  { 'TemporarilyUnavailable' : string } |
  { 'AlreadyProcessed' : null } |
  { 'OrdTxError' : string } |
  { 'NotBridgeTx' : null } |
  { 'AmountIsZero' : null } |
  { 'InvalidRuneId' : string } |
  { 'InvalidArgs' : string } |
  { 'AlreadySubmitted' : null } |
  { 'InvalidTxId' : null } |
  { 'TxNotFoundInMemPool' : null } |
  { 'Unknown' : null } |
  { 'NoNewUtxos' : null } |
  { 'UnsupportedChainId' : string } |
  { 'UnsupportedToken' : string };
export type IcpChainKeyToken = { 'CKBTC' : null };
export interface InitArgs {
  'hub_principal' : Principal,
  'network' : Network_1,
  'chain_id' : string,
  'admins' : Array<Principal>,
  'indexer_principal' : Principal,
}
export interface LockTicketRequest {
  'received_at' : bigint,
  'ticker' : string,
  'token_id' : string,
  'txid' : Uint8Array | number[],
  'target_chain_id' : string,
  'amount' : string,
  'receiver' : string,
}
export type Network = { 'mainnet' : null } |
  { 'regtest' : null } |
  { 'testnet' : null };
export type Network_1 = { 'mainnet' : null } |
  { 'local' : null } |
  { 'testnet' : null };
export type ReleaseTokenStatus = { 'Signing' : null } |
  { 'Confirmed' : string } |
  { 'Sending' : string } |
  { 'Unknown' : null } |
  { 'Submitted' : string } |
  { 'Pending' : null };
export type Result = { 'Ok' : string } |
  { 'Err' : CustomToBitcoinError };
export type Result_1 = { 'Ok' : Array<string> } |
  { 'Err' : CustomToBitcoinError };
export type Result_2 = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export interface StateProfile {
  'next_consume_ticket_seq' : bigint,
  'finalized_lock_ticket_requests' : Array<
    [Uint8Array | number[], LockTicketRequest]
  >,
  'next_consume_directive_seq' : bigint,
  'hub_principal' : Principal,
  'ecdsa_key_name' : string,
  'deposit_addr' : [] | [string],
  'next_directive_seq' : bigint,
  'ecdsa_public_key' : [] | [ECDSAPublicKey],
  'chain_id' : string,
  'pending_lock_ticket_requests' : Array<
    [Uint8Array | number[], LockTicketRequest]
  >,
  'tokens' : Array<[string, Token]>,
  'btc_network' : Network,
  'admins' : Array<Principal>,
  'counterparties' : Array<[string, Chain]>,
  'next_ticket_seq' : bigint,
  'chain_state' : ChainState,
  'min_confirmations' : number,
  'indexer_principal' : Principal,
  'deposit_pubkey' : [] | [string],
}
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
  'symbol' : string,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'RedeemIcpChainKeyAssets' : IcpChainKeyToken } |
  { 'Transfer' : null };
export interface UtxoArgs { 'id' : string, 'index' : number, 'amount' : bigint }
export interface _SERVICE {
  'brc20_state' : ActorMethod<[], StateProfile>,
  'build_commit_tx' : ActorMethod<
    [string, Array<UtxoArgs>, string, string, string, string, string],
    Result
  >,
  'build_reveal_transfer' : ActorMethod<[string, string], Result_1>,
  'finalized_unlock_tickets' : ActorMethod<[bigint], string>,
  'generate_deposit_addr' : ActorMethod<[], [[] | [string], [] | [string]]>,
  'generate_ticket' : ActorMethod<[GenerateTicketArgs], Result_2>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'pending_unlock_tickets' : ActorMethod<[bigint], string>,
  'release_token_status' : ActorMethod<[string], ReleaseTokenStatus>,
  'resend_unlock_ticket' : ActorMethod<[bigint], string>,
  'test_create_tx' : ActorMethod<[Ticket, bigint], undefined>,
  'test_update_utxos' : ActorMethod<[], string>,
  'transfer_fee' : ActorMethod<[string], bigint>,
  'update_brc20_indexer' : ActorMethod<[Principal], undefined>,
  'update_fees' : ActorMethod<[Array<UtxoArgs>], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
