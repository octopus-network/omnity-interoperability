import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AccountInfo {
  'status' : TxStatus,
  'signature' : [] | [string],
  'account' : string,
  'retry' : bigint,
}
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
  'signature' : string,
  'action' : TxAction,
  'token_id' : string,
  'memo' : [] | [string],
  'sender' : string,
  'target_chain_id' : string,
  'amount' : bigint,
  'receiver' : string,
}
export interface InitArgs {
  'admin' : Principal,
  'hub_principal' : Principal,
  'fee_account' : [] | [string],
  'sol_canister' : Principal,
  'chain_id' : string,
  'schnorr_key_name' : [] | [string],
  'multi_rpc_config' : MultiRpcConfig,
  'chain_state' : ChainState,
  'forward' : [] | [string],
}
export interface MintTokenRequest {
  'status' : TxStatus,
  'signature' : [] | [string],
  'associated_account' : string,
  'ticket_id' : string,
  'amount' : bigint,
  'token_mint' : string,
  'retry' : bigint,
}
export interface MultiRpcConfig {
  'rpc_list' : Array<string>,
  'minimum_response_count' : number,
}
export type Permission = { 'Update' : null } |
  { 'Query' : null };
export type Reason = { 'QueueIsFull' : null } |
  { 'CanisterError' : string } |
  { 'OutOfCycles' : null } |
  { 'Rejected' : string };
export type Result = { 'Ok' : AccountInfo } |
  { 'Err' : CallError };
export type Result_1 = { 'Ok' : string } |
  { 'Err' : CallError };
export type Result_10 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type Result_3 = { 'Ok' : [] | [string] } |
  { 'Err' : CallError };
export type Result_4 = { 'Ok' : null } |
  { 'Err' : TransactionError };
export type Result_5 = { 'Ok' : Array<TransactionStatus> } |
  { 'Err' : CallError };
export type Result_6 = { 'Ok' : TxStatus } |
  { 'Err' : CallError };
export type Result_7 = { 'Ok' : MintTokenRequest } |
  { 'Err' : CallError };
export type Result_8 = { 'Ok' : null } |
  { 'Err' : GenerateTicketError };
export type Result_9 = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type RouteArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
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
export interface TokenInfo {
  'uri' : string,
  'decimals' : number,
  'token_id' : string,
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
export type TransactionConfirmationStatus = { 'Finalized' : null } |
  { 'Confirmed' : null } |
  { 'Processed' : null };
export type TransactionError = { 'InvalidAccountForFee' : null } |
  { 'AddressLookupTableNotFound' : null } |
  { 'MissingSignatureForFee' : null } |
  { 'WouldExceedAccountDataBlockLimit' : null } |
  { 'AccountInUse' : null } |
  { 'DuplicateInstruction' : number } |
  { 'AccountNotFound' : null } |
  { 'TooManyAccountLocks' : null } |
  { 'InvalidAccountIndex' : null } |
  { 'AlreadyProcessed' : null } |
  { 'WouldExceedAccountDataTotalLimit' : null } |
  { 'InvalidAddressLookupTableIndex' : null } |
  { 'SanitizeFailure' : null } |
  { 'ResanitizationNeeded' : null } |
  { 'InvalidRentPayingAccount' : null } |
  { 'MaxLoadedAccountsDataSizeExceeded' : null } |
  { 'InvalidAddressLookupTableData' : null } |
  { 'InvalidWritableAccount' : null } |
  { 'WouldExceedMaxAccountCostLimit' : null } |
  { 'InvalidLoadedAccountsDataSizeLimit' : null } |
  { 'InvalidProgramForExecution' : null } |
  { 'InstructionError' : [number, string] } |
  { 'InsufficientFundsForRent' : { 'account_index' : number } } |
  { 'UnsupportedVersion' : null } |
  { 'ClusterMaintenance' : null } |
  { 'WouldExceedMaxVoteCostLimit' : null } |
  { 'SignatureFailure' : null } |
  { 'ProgramAccountNotFound' : null } |
  { 'AccountLoadedTwice' : null } |
  { 'ProgramExecutionTemporarilyRestricted' : { 'account_index' : number } } |
  { 'AccountBorrowOutstanding' : null } |
  { 'WouldExceedMaxBlockCostLimit' : null } |
  { 'InvalidAddressLookupTableOwner' : null } |
  { 'InsufficientFundsForFee' : null } |
  { 'CallChainTooDeep' : null } |
  { 'UnbalancedTransaction' : null } |
  { 'BlockhashNotFound' : null };
export interface TransactionStatus {
  'err' : [] | [TransactionError],
  'confirmations' : [] | [bigint],
  'status' : Result_4,
  'slot' : bigint,
  'confirmation_status' : [] | [TransactionConfirmationStatus],
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export type TxStatus = { 'New' : null } |
  { 'Finalized' : null } |
  { 'TxFailed' : { 'e' : string } } |
  { 'Pending' : null };
export interface UpgradeArgs {
  'admin' : [] | [Principal],
  'hub_principal' : [] | [Principal],
  'fee_account' : [] | [string],
  'sol_canister' : [] | [Principal],
  'chain_id' : [] | [string],
  'schnorr_key_name' : [] | [string],
  'multi_rpc_config' : [] | [MultiRpcConfig],
  'chain_state' : [] | [ChainState],
  'forward' : [] | [string],
}
export interface _SERVICE {
  'cancel_schedule' : ActorMethod<[], undefined>,
  'create_aossicated_account' : ActorMethod<[string, string], Result>,
  'create_mint_account' : ActorMethod<[TokenInfo], Result>,
  'derive_aossicated_account' : ActorMethod<[string, string], Result_1>,
  'derive_mint_account' : ActorMethod<[TokenInfo], Result_1>,
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result_2>,
  'get_account_info' : ActorMethod<[string], Result_3>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_fee_account' : ActorMethod<[], string>,
  'get_latest_blockhash' : ActorMethod<[], Result_1>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_signature_status' : ActorMethod<[Array<string>], Result_5>,
  'get_ticket_from_queue' : ActorMethod<[string], [] | [[bigint, Ticket]]>,
  'get_tickets_failed_to_hub' : ActorMethod<[], Array<Ticket>>,
  'get_tickets_from_queue' : ActorMethod<[], Array<[bigint, Ticket]>>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'get_transaction' : ActorMethod<[string, [] | [string]], Result_1>,
  'mint_token' : ActorMethod<[MintTokenRequest], Result_6>,
  'mint_token_req' : ActorMethod<[string], Result_7>,
  'mint_token_status' : ActorMethod<[string], Result_6>,
  'mint_token_tx_hash' : ActorMethod<[string], Result_3>,
  'multi_rpc_config' : ActorMethod<[], MultiRpcConfig>,
  'query_aossicated_account' : ActorMethod<
    [string, string],
    [] | [AccountInfo]
  >,
  'query_aossicated_account_address' : ActorMethod<
    [string, string],
    [] | [string]
  >,
  'query_mint_account' : ActorMethod<[string], [] | [AccountInfo]>,
  'query_mint_address' : ActorMethod<[string], [] | [string]>,
  'remove_ticket_from_quene' : ActorMethod<[string], [] | [Ticket]>,
  'resend_tickets' : ActorMethod<[], Result_8>,
  'set_permissions' : ActorMethod<[Principal, Permission], undefined>,
  'sign' : ActorMethod<[string], Result_9>,
  'signer' : ActorMethod<[], Result_10>,
  'start_schedule' : ActorMethod<[], undefined>,
  'transfer_to' : ActorMethod<[string, bigint], Result_1>,
  'update_associated_account' : ActorMethod<
    [string, string, AccountInfo],
    Result
  >,
  'update_forward' : ActorMethod<[[] | [string]], undefined>,
  'update_mint_token_req' : ActorMethod<[MintTokenRequest], Result_7>,
  'update_multi_rpc' : ActorMethod<[MultiRpcConfig], undefined>,
  'update_schnorr_key' : ActorMethod<[string], undefined>,
  'update_token_metadata' : ActorMethod<[TokenInfo], Result_1>,
  'valid_tx_from_multi_rpc' : ActorMethod<[string], Result_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
