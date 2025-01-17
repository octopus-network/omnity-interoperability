export const idlFactory = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs_1 = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs_1),
    'Init' : InitArgs,
  });
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const IcpChainKeyToken = IDL.Variant({ 'CKBTC' : IDL.Null });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
    'RedeemIcpChainKeyAssets' : IcpChainKeyToken,
    'Transfer' : IDL.Null,
  });
  const GenerateTicketReq = IDL.Record({
    'action' : TxAction,
    'token_id' : IDL.Text,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat,
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
  const Result_1 = IDL.Variant({
    'Ok' : GenerateTicketOk,
    'Err' : GenerateTicketError,
  });
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
  const TokenResp = IDL.Record({
    'principal' : IDL.Opt(IDL.Principal),
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const MintTokenStatus = IDL.Variant({
    'Finalized' : IDL.Record({ 'block_index' : IDL.Nat64 }),
    'Unknown' : IDL.Null,
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
  const Result_2 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : GenerateTicketError,
  });
  const MetadataValue = IDL.Variant({
    'Int' : IDL.Int,
    'Nat' : IDL.Nat,
    'Blob' : IDL.Vec(IDL.Nat8),
    'Text' : IDL.Text,
  });
  const ChangeFeeCollector = IDL.Variant({
    'SetTo' : Account,
    'Unset' : IDL.Null,
  });
  const FeatureFlags = IDL.Record({ 'icrc2' : IDL.Bool });
  const UpgradeArgs = IDL.Record({
    'token_symbol' : IDL.Opt(IDL.Text),
    'transfer_fee' : IDL.Opt(IDL.Nat),
    'metadata' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, MetadataValue))),
    'maximum_number_of_accounts' : IDL.Opt(IDL.Nat64),
    'accounts_overflow_trim_quantity' : IDL.Opt(IDL.Nat64),
    'change_fee_collector' : IDL.Opt(ChangeFeeCollector),
    'max_memo_length' : IDL.Opt(IDL.Nat16),
    'token_name' : IDL.Opt(IDL.Text),
    'feature_flags' : IDL.Opt(FeatureFlags),
  });
  return IDL.Service({
    'collect_ledger_fee' : IDL.Func(
        [IDL.Principal, IDL.Opt(IDL.Nat), Account],
        [Result],
        [],
      ),
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result_1], []),
    'generate_ticket_v2' : IDL.Func([GenerateTicketReq], [Result_1], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee_account' : IDL.Func(
        [IDL.Opt(IDL.Principal)],
        [IDL.Vec(IDL.Nat8)],
        ['query'],
      ),
    'get_readable_fee_account' : IDL.Func(
        [IDL.Opt(IDL.Principal)],
        [IDL.Text],
        ['query'],
      ),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat64)], ['query']),
    'get_token_ledger' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [MintTokenStatus], ['query']),
    'query_failed_tickets' : IDL.Func([], [IDL.Vec(Ticket)], ['query']),
    'remove_controller' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result],
        [],
      ),
    'resend_tickets' : IDL.Func([], [Result_2], []),
    'update_icrc_ledger' : IDL.Func([IDL.Principal, UpgradeArgs], [Result], []),
  });
};
export const init = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs_1 = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs_1),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
