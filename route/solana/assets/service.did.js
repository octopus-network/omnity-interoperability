export const idlFactory = ({ IDL }) => {
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const RpcProvider = IDL.Record({
    'host' : IDL.Text,
    'headers' : IDL.Opt(IDL.Vec(HttpHeader)),
    'api_key_param' : IDL.Opt(IDL.Text),
  });
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'admin' : IDL.Opt(IDL.Principal),
    'hub_principal' : IDL.Opt(IDL.Principal),
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'providers' : IDL.Opt(IDL.Vec(RpcProvider)),
    'chain_state' : IDL.Opt(ChainState),
    'proxy' : IDL.Opt(IDL.Text),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'providers' : IDL.Vec(RpcProvider),
    'chain_state' : ChainState,
    'proxy' : IDL.Text,
    'minimum_response_count' : IDL.Nat32,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
    'Transfer' : IDL.Null,
  });
  const GenerateTicketReq = IDL.Record({
    'signature' : IDL.Text,
    'action' : TxAction,
    'token_id' : IDL.Text,
    'memo' : IDL.Opt(IDL.Text),
    'sender' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat64,
    'receiver' : IDL.Text,
  });
  const GenerateTicketOk = IDL.Record({ 'ticket_id' : IDL.Text });
  const GenerateTicketError = IDL.Variant({
    'InsufficientRedeemFee' : IDL.Record({
      'provided' : IDL.Nat64,
      'required' : IDL.Nat64,
    }),
    'SendTicketErr' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat64 }),
    'TransferFailure' : IDL.Text,
    'UnsupportedAction' : IDL.Text,
    'RedeemFeeNotSet' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat64 }),
  });
  const Result = IDL.Variant({
    'Ok' : GenerateTicketOk,
    'Err' : GenerateTicketError,
  });
  const ParsedAccount = IDL.Record({
    'space' : IDL.Nat64,
    'parsed' : IDL.Text,
    'program' : IDL.Text,
  });
  const UiAccountEncoding = IDL.Variant({
    'base64+zstd' : IDL.Null,
    'jsonParsed' : IDL.Null,
    'base58' : IDL.Null,
    'base64' : IDL.Null,
    'binary' : IDL.Null,
  });
  const UiAccountData = IDL.Variant({
    'json' : ParsedAccount,
    'legacyBinary' : IDL.Text,
    'binary' : IDL.Tuple(IDL.Text, UiAccountEncoding),
  });
  const UiAccount = IDL.Record({
    'executable' : IDL.Bool,
    'owner' : IDL.Text,
    'lamports' : IDL.Nat64,
    'data' : UiAccountData,
    'space' : IDL.Opt(IDL.Nat64),
    'rentEpoch' : IDL.Nat64,
  });
  const TxError = IDL.Record({
    'signature' : IDL.Text,
    'block_hash' : IDL.Text,
    'error' : IDL.Text,
  });
  const Reason = IDL.Variant({
    'QueueIsFull' : IDL.Null,
    'CanisterError' : IDL.Text,
    'OutOfCycles' : IDL.Null,
    'Rejected' : IDL.Text,
    'TxError' : TxError,
  });
  const CallError = IDL.Record({ 'method' : IDL.Text, 'reason' : Reason });
  const Result_1 = IDL.Variant({
    'Ok' : IDL.Opt(UiAccount),
    'Err' : CallError,
  });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const ChainType = IDL.Variant({
    'SettlementChain' : IDL.Null,
    'ExecutionChain' : IDL.Null,
  });
  const Chain = IDL.Record({
    'fee_token' : IDL.Opt(IDL.Text),
    'canister_id' : IDL.Text,
    'chain_id' : IDL.Text,
    'counterparties' : IDL.Opt(IDL.Vec(IDL.Text)),
    'chain_state' : ChainState,
    'chain_type' : ChainType,
    'contract_address' : IDL.Opt(IDL.Text),
  });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : CallError });
  const InstructionError = IDL.Variant({
    'ModifiedProgramId' : IDL.Null,
    'CallDepth' : IDL.Null,
    'Immutable' : IDL.Null,
    'GenericError' : IDL.Null,
    'ExecutableAccountNotRentExempt' : IDL.Null,
    'IncorrectAuthority' : IDL.Null,
    'PrivilegeEscalation' : IDL.Null,
    'ReentrancyNotAllowed' : IDL.Null,
    'InvalidInstructionData' : IDL.Null,
    'RentEpochModified' : IDL.Null,
    'IllegalOwner' : IDL.Null,
    'ComputationalBudgetExceeded' : IDL.Null,
    'ExecutableDataModified' : IDL.Null,
    'ExecutableLamportChange' : IDL.Null,
    'UnbalancedInstruction' : IDL.Null,
    'ProgramEnvironmentSetupFailure' : IDL.Null,
    'IncorrectProgramId' : IDL.Null,
    'UnsupportedSysvar' : IDL.Null,
    'UnsupportedProgramId' : IDL.Null,
    'AccountDataTooSmall' : IDL.Null,
    'NotEnoughAccountKeys' : IDL.Null,
    'AccountBorrowFailed' : IDL.Null,
    'InvalidRealloc' : IDL.Null,
    'AccountNotExecutable' : IDL.Null,
    'AccountNotRentExempt' : IDL.Null,
    'Custom' : IDL.Nat32,
    'AccountDataSizeChanged' : IDL.Null,
    'MaxAccountsDataAllocationsExceeded' : IDL.Null,
    'ExternalAccountLamportSpend' : IDL.Null,
    'ExternalAccountDataModified' : IDL.Null,
    'MissingAccount' : IDL.Null,
    'ProgramFailedToComplete' : IDL.Null,
    'MaxInstructionTraceLengthExceeded' : IDL.Null,
    'InvalidAccountData' : IDL.Null,
    'ProgramFailedToCompile' : IDL.Null,
    'ExecutableModified' : IDL.Null,
    'InvalidAccountOwner' : IDL.Null,
    'MaxSeedLengthExceeded' : IDL.Null,
    'AccountAlreadyInitialized' : IDL.Null,
    'AccountBorrowOutstanding' : IDL.Null,
    'ReadonlyDataModified' : IDL.Null,
    'UninitializedAccount' : IDL.Null,
    'InvalidArgument' : IDL.Null,
    'BorshIoError' : IDL.Text,
    'BuiltinProgramsMustConsumeComputeUnits' : IDL.Null,
    'MissingRequiredSignature' : IDL.Null,
    'DuplicateAccountOutOfSync' : IDL.Null,
    'MaxAccountsExceeded' : IDL.Null,
    'ArithmeticOverflow' : IDL.Null,
    'InvalidError' : IDL.Null,
    'InvalidSeeds' : IDL.Null,
    'DuplicateAccountIndex' : IDL.Null,
    'ReadonlyLamportChange' : IDL.Null,
    'InsufficientFunds' : IDL.Null,
  });
  const TransactionError = IDL.Variant({
    'InvalidAccountForFee' : IDL.Null,
    'AddressLookupTableNotFound' : IDL.Null,
    'MissingSignatureForFee' : IDL.Null,
    'WouldExceedAccountDataBlockLimit' : IDL.Null,
    'AccountInUse' : IDL.Null,
    'DuplicateInstruction' : IDL.Nat8,
    'AccountNotFound' : IDL.Null,
    'TooManyAccountLocks' : IDL.Null,
    'InvalidAccountIndex' : IDL.Null,
    'AlreadyProcessed' : IDL.Null,
    'WouldExceedAccountDataTotalLimit' : IDL.Null,
    'InvalidAddressLookupTableIndex' : IDL.Null,
    'SanitizeFailure' : IDL.Null,
    'ResanitizationNeeded' : IDL.Null,
    'InvalidRentPayingAccount' : IDL.Null,
    'MaxLoadedAccountsDataSizeExceeded' : IDL.Null,
    'InvalidAddressLookupTableData' : IDL.Null,
    'InvalidWritableAccount' : IDL.Null,
    'WouldExceedMaxAccountCostLimit' : IDL.Null,
    'InvalidLoadedAccountsDataSizeLimit' : IDL.Null,
    'InvalidProgramForExecution' : IDL.Null,
    'InstructionError' : IDL.Tuple(IDL.Nat8, InstructionError),
    'InsufficientFundsForRent' : IDL.Record({ 'account_index' : IDL.Nat8 }),
    'UnsupportedVersion' : IDL.Null,
    'ClusterMaintenance' : IDL.Null,
    'WouldExceedMaxVoteCostLimit' : IDL.Null,
    'SignatureFailure' : IDL.Null,
    'ProgramAccountNotFound' : IDL.Null,
    'AccountLoadedTwice' : IDL.Null,
    'ProgramExecutionTemporarilyRestricted' : IDL.Record({
      'account_index' : IDL.Nat8,
    }),
    'AccountBorrowOutstanding' : IDL.Null,
    'WouldExceedMaxBlockCostLimit' : IDL.Null,
    'InvalidAddressLookupTableOwner' : IDL.Null,
    'InsufficientFundsForFee' : IDL.Null,
    'CallChainTooDeep' : IDL.Null,
    'UnbalancedTransaction' : IDL.Null,
    'BlockhashNotFound' : IDL.Null,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : TransactionError });
  const TransactionConfirmationStatus = IDL.Variant({
    'finalized' : IDL.Null,
    'confirmed' : IDL.Null,
    'processed' : IDL.Null,
  });
  const TransactionStatus = IDL.Record({
    'err' : IDL.Opt(TransactionError),
    'confirmations' : IDL.Opt(IDL.Nat64),
    'status' : Result_4,
    'confirmationStatus' : IDL.Opt(TransactionConfirmationStatus),
    'slot' : IDL.Nat64,
  });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Opt(TransactionStatus)),
    'Err' : CallError,
  });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const UiAddressTableLookup = IDL.Record({
    'accountKey' : IDL.Text,
    'writableIndexes' : IDL.Vec(IDL.Nat8),
    'readonlyIndexes' : IDL.Vec(IDL.Nat8),
  });
  const UiCompiledInstruction = IDL.Record({
    'data' : IDL.Text,
    'accounts' : IDL.Vec(IDL.Nat8),
    'programIdIndex' : IDL.Nat8,
    'stackHeight' : IDL.Opt(IDL.Nat32),
  });
  const MessageHeader = IDL.Record({
    'numReadonlySignedAccounts' : IDL.Nat8,
    'numRequiredSignatures' : IDL.Nat8,
    'numReadonlyUnsignedAccounts' : IDL.Nat8,
  });
  const UiRawMessage = IDL.Record({
    'addressTableLookups' : IDL.Opt(IDL.Vec(UiAddressTableLookup)),
    'instructions' : IDL.Vec(UiCompiledInstruction),
    'accountKeys' : IDL.Vec(IDL.Text),
    'recentBlockhash' : IDL.Text,
    'header' : MessageHeader,
  });
  const ParsedInstruction = IDL.Record({
    'stackHeight' : IDL.Opt(IDL.Nat32),
    'programId' : IDL.Text,
    'parsed' : IDL.Vec(IDL.Nat8),
    'program' : IDL.Text,
  });
  const UiPartiallyDecodedInstruction = IDL.Record({
    'data' : IDL.Text,
    'accounts' : IDL.Vec(IDL.Text),
    'stackHeight' : IDL.Opt(IDL.Nat32),
    'programId' : IDL.Text,
  });
  const UiParsedInstruction = IDL.Variant({
    'Parsed' : ParsedInstruction,
    'PartiallyDecoded' : UiPartiallyDecodedInstruction,
  });
  const UiInstruction = IDL.Variant({
    'Parsed' : UiParsedInstruction,
    'Compiled' : UiCompiledInstruction,
  });
  const AccountKeySource = IDL.Variant({
    'Transaction' : IDL.Null,
    'LookupTable' : IDL.Null,
  });
  const AccountKey = IDL.Record({
    'writable' : IDL.Bool,
    'source' : IDL.Opt(AccountKeySource),
    'pubkey' : IDL.Text,
    'signer' : IDL.Bool,
  });
  const UiParsedMessage = IDL.Record({
    'addressTableLookups' : IDL.Opt(IDL.Vec(UiAddressTableLookup)),
    'instructions' : IDL.Vec(UiInstruction),
    'accountKeys' : IDL.Vec(AccountKey),
    'recentBlockhash' : IDL.Text,
  });
  const UiMessage = IDL.Variant({
    'raw' : UiRawMessage,
    'parsed' : UiParsedMessage,
  });
  const UiTransaction = IDL.Record({
    'message' : UiMessage,
    'signatures' : IDL.Vec(IDL.Text),
  });
  const Result_6 = IDL.Variant({ 'Ok' : UiTransaction, 'Err' : CallError });
  const TxStatus = IDL.Variant({
    'New' : IDL.Null,
    'Finalized' : IDL.Null,
    'TxFailed' : IDL.Record({ 'e' : TxError }),
    'Pending' : IDL.Null,
  });
  const MintTokenRequest = IDL.Record({
    'status' : TxStatus,
    'signature' : IDL.Opt(IDL.Text),
    'associated_account' : IDL.Text,
    'retry_4_building' : IDL.Nat64,
    'ticket_id' : IDL.Text,
    'retry_4_status' : IDL.Nat64,
    'amount' : IDL.Nat64,
    'token_mint' : IDL.Text,
  });
  const Result_7 = IDL.Variant({ 'Ok' : MintTokenRequest, 'Err' : CallError });
  const Result_8 = IDL.Variant({ 'Ok' : TxStatus, 'Err' : CallError });
  const Result_9 = IDL.Variant({ 'Ok' : IDL.Opt(IDL.Text), 'Err' : CallError });
  const AccountInfo = IDL.Record({
    'status' : TxStatus,
    'signature' : IDL.Opt(IDL.Text),
    'retry_4_building' : IDL.Nat64,
    'account' : IDL.Text,
    'retry_4_status' : IDL.Nat64,
  });
  return IDL.Service({
    'gen_tickets_req' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(GenerateTicketReq)],
        ['query'],
      ),
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result], []),
    'get_account_info' : IDL.Func([IDL.Text], [Result_1], []),
    'get_balance' : IDL.Func([IDL.Text], [Result_2], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee_account' : IDL.Func([], [IDL.Text], ['query']),
    'get_latest_blockhash' : IDL.Func([], [Result_3], []),
    'get_raw_transaction' : IDL.Func([IDL.Text], [Result_3], []),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat)], ['query']),
    'get_signature_status' : IDL.Func([IDL.Vec(IDL.Text)], [Result_5], []),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'get_transaction' : IDL.Func([IDL.Text], [Result_6], []),
    'get_tx_instructions' : IDL.Func([IDL.Text], [Result_3], []),
    'mint_token_req' : IDL.Func([IDL.Text], [Result_7], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [Result_8], ['query']),
    'mint_token_tx_hash' : IDL.Func([IDL.Text], [Result_9], ['query']),
    'query_mint_account' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(AccountInfo)],
        ['query'],
      ),
    'query_mint_address' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
  });
};
export const init = ({ IDL }) => {
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const RpcProvider = IDL.Record({
    'host' : IDL.Text,
    'headers' : IDL.Opt(IDL.Vec(HttpHeader)),
    'api_key_param' : IDL.Opt(IDL.Text),
  });
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'admin' : IDL.Opt(IDL.Principal),
    'hub_principal' : IDL.Opt(IDL.Principal),
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'providers' : IDL.Opt(IDL.Vec(RpcProvider)),
    'chain_state' : IDL.Opt(ChainState),
    'proxy' : IDL.Opt(IDL.Text),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'providers' : IDL.Vec(RpcProvider),
    'chain_state' : ChainState,
    'proxy' : IDL.Text,
    'minimum_response_count' : IDL.Nat32,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
