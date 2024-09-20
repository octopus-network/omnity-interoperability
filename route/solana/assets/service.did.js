export const idlFactory = ({ IDL }) => {
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
    'schnorr_canister' : IDL.Opt(IDL.Principal),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_canister' : IDL.Opt(IDL.Principal),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const TxStatus = IDL.Variant({
    'Finalized' : IDL.Null,
    'TxFailed' : IDL.Record({ 'e' : IDL.Text }),
    'Unknown' : IDL.Null,
  });
  const AccountInfo = IDL.Record({
    'status' : TxStatus,
    'signature' : IDL.Opt(IDL.Text),
    'account' : IDL.Text,
    'retry' : IDL.Nat64,
  });
  const Reason = IDL.Variant({
    'QueueIsFull' : IDL.Null,
    'CanisterError' : IDL.Text,
    'OutOfCycles' : IDL.Null,
    'Rejected' : IDL.Text,
  });
  const CallError = IDL.Record({ 'method' : IDL.Text, 'reason' : Reason });
  const Result = IDL.Variant({ 'Ok' : AccountInfo, 'Err' : CallError });
  const TokenInfo = IDL.Record({
    'uri' : IDL.Text,
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : CallError });
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
  const Result_2 = IDL.Variant({
    'Ok' : GenerateTicketOk,
    'Err' : GenerateTicketError,
  });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Opt(IDL.Text), 'Err' : CallError });
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
    'InstructionError' : IDL.Tuple(IDL.Nat8, IDL.Text),
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
    'Finalized' : IDL.Null,
    'Confirmed' : IDL.Null,
    'Processed' : IDL.Null,
  });
  const TransactionStatus = IDL.Record({
    'err' : IDL.Opt(TransactionError),
    'confirmations' : IDL.Opt(IDL.Nat64),
    'status' : Result_4,
    'slot' : IDL.Nat64,
    'confirmation_status' : IDL.Opt(TransactionConfirmationStatus),
  });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Vec(TransactionStatus),
    'Err' : CallError,
  });
  const TicketType = IDL.Variant({
    'Resubmit' : IDL.Null,
    'Normal' : IDL.Null,
  });
  const Ticket = IDL.Record({
    'token' : IDL.Text,
    'action' : TxAction,
    'dst_chain' : IDL.Text,
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'ticket_id' : IDL.Text,
    'sender' : IDL.Opt(IDL.Text),
    'ticket_time' : IDL.Nat64,
    'ticket_type' : TicketType,
    'src_chain' : IDL.Text,
    'amount' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const MintTokenRequest = IDL.Record({
    'status' : TxStatus,
    'signature' : IDL.Opt(IDL.Text),
    'associated_account' : IDL.Text,
    'ticket_id' : IDL.Text,
    'amount' : IDL.Nat64,
    'token_mint' : IDL.Text,
    'retry' : IDL.Nat64,
  });
  const Result_6 = IDL.Variant({ 'Ok' : TxStatus, 'Err' : CallError });
  const Result_7 = IDL.Variant({ 'Ok' : MintTokenRequest, 'Err' : CallError });
  const Result_8 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : GenerateTicketError,
  });
  const Permission = IDL.Variant({ 'Update' : IDL.Null, 'Query' : IDL.Null });
  const Result_9 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'cancel_schedule' : IDL.Func([], [], []),
    'create_aossicated_account' : IDL.Func([IDL.Text, IDL.Text], [Result], []),
    'create_mint_account' : IDL.Func([TokenInfo], [Result], []),
    'derive_aossicated_account' : IDL.Func(
        [IDL.Text, IDL.Text],
        [Result_1],
        [],
      ),
    'derive_mint_account' : IDL.Func([TokenInfo], [Result_1], []),
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result_2], []),
    'get_account_info' : IDL.Func([IDL.Text], [Result_3], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee_account' : IDL.Func([], [IDL.Text], []),
    'get_latest_blockhash' : IDL.Func([], [Result_1], []),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat)], ['query']),
    'get_signature_status' : IDL.Func([IDL.Vec(IDL.Text)], [Result_5], []),
    'get_ticket_from_queue' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'get_tickets_failed_to_hub' : IDL.Func([], [IDL.Vec(Ticket)], ['query']),
    'get_tickets_from_queue' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'get_transaction' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [Result_1], []),
    'mint_token' : IDL.Func([MintTokenRequest], [Result_6], []),
    'mint_token_req' : IDL.Func([IDL.Text], [Result_7], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [Result_6], ['query']),
    'query_aossicated_account' : IDL.Func(
        [IDL.Text, IDL.Text],
        [IDL.Opt(AccountInfo)],
        ['query'],
      ),
    'query_aossicated_account_address' : IDL.Func(
        [IDL.Text, IDL.Text],
        [IDL.Opt(IDL.Text)],
        ['query'],
      ),
    'query_mint_account' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(AccountInfo)],
        ['query'],
      ),
    'query_mint_address' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
    'remove_ticket_from_quene' : IDL.Func([IDL.Text], [IDL.Opt(Ticket)], []),
    'resend_tickets' : IDL.Func([], [Result_8], []),
    'set_permissions' : IDL.Func([IDL.Principal, Permission], [], []),
    'sign' : IDL.Func([IDL.Text], [Result_9], []),
    'signer' : IDL.Func([], [Result_9], []),
    'start_schedule' : IDL.Func([], [], []),
    'transfer_to' : IDL.Func([IDL.Text, IDL.Nat64], [Result_1], []),
    'update_schnorr_info' : IDL.Func([IDL.Principal, IDL.Text], [], []),
    'update_token_metadata' : IDL.Func([TokenInfo], [Result_1], []),
  });
};
export const init = ({ IDL }) => {
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
    'schnorr_canister' : IDL.Opt(IDL.Principal),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_canister' : IDL.Opt(IDL.Principal),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
