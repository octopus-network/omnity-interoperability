import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AccountInfo {
  'status' : TxStatus,
  'signature' : [] | [string],
  'retry_4_building' : bigint,
  'account' : string,
  'retry_4_status' : bigint,
}
export interface AccountKey {
  'writable' : boolean,
  'source' : [] | [AccountKeySource],
  'pubkey' : string,
  'signer' : boolean,
}
export type AccountKeySource = { 'Transaction' : null } |
  { 'LookupTable' : null };
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
export interface HttpHeader { 'value' : string, 'name' : string }
export interface InitArgs {
  'admin' : Principal,
  'hub_principal' : Principal,
  'fee_account' : [] | [string],
  'sol_canister' : Principal,
  'chain_id' : string,
  'schnorr_key_name' : [] | [string],
  'providers' : Array<RpcProvider>,
  'chain_state' : ChainState,
  'proxy' : string,
  'minimum_response_count' : number,
}
export type InstructionError = { 'ModifiedProgramId' : null } |
  { 'CallDepth' : null } |
  { 'Immutable' : null } |
  { 'GenericError' : null } |
  { 'ExecutableAccountNotRentExempt' : null } |
  { 'IncorrectAuthority' : null } |
  { 'PrivilegeEscalation' : null } |
  { 'ReentrancyNotAllowed' : null } |
  { 'InvalidInstructionData' : null } |
  { 'RentEpochModified' : null } |
  { 'IllegalOwner' : null } |
  { 'ComputationalBudgetExceeded' : null } |
  { 'ExecutableDataModified' : null } |
  { 'ExecutableLamportChange' : null } |
  { 'UnbalancedInstruction' : null } |
  { 'ProgramEnvironmentSetupFailure' : null } |
  { 'IncorrectProgramId' : null } |
  { 'UnsupportedSysvar' : null } |
  { 'UnsupportedProgramId' : null } |
  { 'AccountDataTooSmall' : null } |
  { 'NotEnoughAccountKeys' : null } |
  { 'AccountBorrowFailed' : null } |
  { 'InvalidRealloc' : null } |
  { 'AccountNotExecutable' : null } |
  { 'AccountNotRentExempt' : null } |
  { 'Custom' : number } |
  { 'AccountDataSizeChanged' : null } |
  { 'MaxAccountsDataAllocationsExceeded' : null } |
  { 'ExternalAccountLamportSpend' : null } |
  { 'ExternalAccountDataModified' : null } |
  { 'MissingAccount' : null } |
  { 'ProgramFailedToComplete' : null } |
  { 'MaxInstructionTraceLengthExceeded' : null } |
  { 'InvalidAccountData' : null } |
  { 'ProgramFailedToCompile' : null } |
  { 'ExecutableModified' : null } |
  { 'InvalidAccountOwner' : null } |
  { 'MaxSeedLengthExceeded' : null } |
  { 'AccountAlreadyInitialized' : null } |
  { 'AccountBorrowOutstanding' : null } |
  { 'ReadonlyDataModified' : null } |
  { 'UninitializedAccount' : null } |
  { 'InvalidArgument' : null } |
  { 'BorshIoError' : string } |
  { 'BuiltinProgramsMustConsumeComputeUnits' : null } |
  { 'MissingRequiredSignature' : null } |
  { 'DuplicateAccountOutOfSync' : null } |
  { 'MaxAccountsExceeded' : null } |
  { 'ArithmeticOverflow' : null } |
  { 'InvalidError' : null } |
  { 'InvalidSeeds' : null } |
  { 'DuplicateAccountIndex' : null } |
  { 'ReadonlyLamportChange' : null } |
  { 'InsufficientFunds' : null };
export interface MessageHeader {
  'numReadonlySignedAccounts' : number,
  'numRequiredSignatures' : number,
  'numReadonlyUnsignedAccounts' : number,
}
export interface MintTokenRequest {
  'status' : TxStatus,
  'signature' : [] | [string],
  'associated_account' : string,
  'retry_4_building' : bigint,
  'ticket_id' : string,
  'retry_4_status' : bigint,
  'amount' : bigint,
  'token_mint' : string,
}
export interface ParsedAccount {
  'space' : bigint,
  'parsed' : string,
  'program' : string,
}
export interface ParsedInstruction {
  'stackHeight' : [] | [number],
  'programId' : string,
  'parsed' : Uint8Array | number[],
  'program' : string,
}
export type Reason = { 'QueueIsFull' : null } |
  { 'CanisterError' : string } |
  { 'OutOfCycles' : null } |
  { 'Rejected' : string } |
  { 'TxError' : TxError };
export type Result = { 'Ok' : GenerateTicketOk } |
  { 'Err' : GenerateTicketError };
export type Result_1 = { 'Ok' : [] | [UiAccount] } |
  { 'Err' : CallError };
export type Result_2 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : string } |
  { 'Err' : CallError };
export type Result_4 = { 'Ok' : null } |
  { 'Err' : TransactionError };
export type Result_5 = { 'Ok' : Array<[] | [TransactionStatus]> } |
  { 'Err' : CallError };
export type Result_6 = { 'Ok' : UiTransaction } |
  { 'Err' : CallError };
export type Result_7 = { 'Ok' : MintTokenRequest } |
  { 'Err' : CallError };
export type Result_8 = { 'Ok' : TxStatus } |
  { 'Err' : CallError };
export type Result_9 = { 'Ok' : [] | [string] } |
  { 'Err' : CallError };
export type RouteArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
export interface RpcProvider {
  'host' : string,
  'headers' : [] | [Array<HttpHeader>],
  'api_key_param' : [] | [string],
}
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'rune_id' : [] | [string],
  'symbol' : string,
}
export type TransactionConfirmationStatus = { 'finalized' : null } |
  { 'confirmed' : null } |
  { 'processed' : null };
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
  { 'InstructionError' : [number, InstructionError] } |
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
  'confirmationStatus' : [] | [TransactionConfirmationStatus],
  'slot' : bigint,
}
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export interface TxError {
  'signature' : string,
  'block_hash' : string,
  'error' : string,
}
export type TxStatus = { 'New' : null } |
  { 'Finalized' : null } |
  { 'TxFailed' : { 'e' : TxError } } |
  { 'Pending' : null };
export interface UiAccount {
  'executable' : boolean,
  'owner' : string,
  'lamports' : bigint,
  'data' : UiAccountData,
  'space' : [] | [bigint],
  'rentEpoch' : bigint,
}
export type UiAccountData = { 'json' : ParsedAccount } |
  { 'legacyBinary' : string } |
  { 'binary' : [string, UiAccountEncoding] };
export type UiAccountEncoding = { 'base64+zstd' : null } |
  { 'jsonParsed' : null } |
  { 'base58' : null } |
  { 'base64' : null } |
  { 'binary' : null };
export interface UiAddressTableLookup {
  'accountKey' : string,
  'writableIndexes' : Uint8Array | number[],
  'readonlyIndexes' : Uint8Array | number[],
}
export interface UiCompiledInstruction {
  'data' : string,
  'accounts' : Uint8Array | number[],
  'programIdIndex' : number,
  'stackHeight' : [] | [number],
}
export type UiInstruction = { 'Parsed' : UiParsedInstruction } |
  { 'Compiled' : UiCompiledInstruction };
export type UiMessage = { 'raw' : UiRawMessage } |
  { 'parsed' : UiParsedMessage };
export type UiParsedInstruction = { 'Parsed' : ParsedInstruction } |
  { 'PartiallyDecoded' : UiPartiallyDecodedInstruction };
export interface UiParsedMessage {
  'addressTableLookups' : [] | [Array<UiAddressTableLookup>],
  'instructions' : Array<UiInstruction>,
  'accountKeys' : Array<AccountKey>,
  'recentBlockhash' : string,
}
export interface UiPartiallyDecodedInstruction {
  'data' : string,
  'accounts' : Array<string>,
  'stackHeight' : [] | [number],
  'programId' : string,
}
export interface UiRawMessage {
  'addressTableLookups' : [] | [Array<UiAddressTableLookup>],
  'instructions' : Array<UiCompiledInstruction>,
  'accountKeys' : Array<string>,
  'recentBlockhash' : string,
  'header' : MessageHeader,
}
export interface UiTransaction {
  'message' : UiMessage,
  'signatures' : Array<string>,
}
export interface UpgradeArgs {
  'admin' : [] | [Principal],
  'hub_principal' : [] | [Principal],
  'fee_account' : [] | [string],
  'sol_canister' : [] | [Principal],
  'chain_id' : [] | [string],
  'schnorr_key_name' : [] | [string],
  'providers' : [] | [Array<RpcProvider>],
  'chain_state' : [] | [ChainState],
  'proxy' : [] | [string],
  'minimum_response_count' : [] | [number],
}
export interface _SERVICE {
  'generate_ticket' : ActorMethod<[GenerateTicketReq], Result>,
  'get_account_info' : ActorMethod<[string], Result_1>,
  'get_balance' : ActorMethod<[string], Result_2>,
  'get_chain_list' : ActorMethod<[], Array<Chain>>,
  'get_fee_account' : ActorMethod<[], string>,
  'get_latest_blockhash' : ActorMethod<[], Result_3>,
  'get_raw_transaction' : ActorMethod<[string], Result_3>,
  'get_redeem_fee' : ActorMethod<[string], [] | [bigint]>,
  'get_signature_status' : ActorMethod<[Array<string>], Result_5>,
  'get_token_list' : ActorMethod<[], Array<TokenResp>>,
  'get_transaction' : ActorMethod<[string], Result_6>,
  'mint_token_req' : ActorMethod<[string], Result_7>,
  'mint_token_status' : ActorMethod<[string], Result_8>,
  'mint_token_tx_hash' : ActorMethod<[string], Result_9>,
  'query_mint_account' : ActorMethod<[string], [] | [AccountInfo]>,
  'query_mint_address' : ActorMethod<[string], [] | [string]>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
