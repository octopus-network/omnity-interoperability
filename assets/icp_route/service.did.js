export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Record({}),
    'Init' : InitArgs,
  });
  const GenerateTicketReq = IDL.Record({
    'token_id' : IDL.Text,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
  });
  const GenerateTicketOk = IDL.Record({ 'block_index' : IDL.Nat64 });
  const GenerateTicketError = IDL.Variant({
    'InsufficientRedeemFee' : IDL.Record({
      'provided' : IDL.Nat64,
      'required' : IDL.Nat64,
    }),
    'SendTicketErr' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat64 }),
    'TransferFailure' : IDL.Text,
    'RedeemFeeNotSet' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat64 }),
  });
  const Result = IDL.Variant({
    'Ok' : GenerateTicketOk,
    'Err' : GenerateTicketError,
  });
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const ChainType = IDL.Variant({
    'SettlementChain' : IDL.Null,
    'ExecutionChain' : IDL.Null,
  });
  const Chain = IDL.Record({
    'fee_token' : IDL.Opt(IDL.Text),
    'chain_id' : IDL.Text,
    'chain_state' : ChainState,
    'chain_type' : ChainType,
    'contract_address' : IDL.Opt(IDL.Text),
  });
  const GetEventsArg = IDL.Record({
    'start' : IDL.Nat64,
    'length' : IDL.Nat64,
  });
  const FeeTokenFactor = IDL.Record({
    'fee_token' : IDL.Text,
    'fee_token_factor' : IDL.Nat,
  });
  const TargetChainFactor = IDL.Record({
    'target_chain_id' : IDL.Text,
    'target_chain_factor' : IDL.Nat,
  });
  const Factor = IDL.Variant({
    'UpdateFeeTokenFactor' : FeeTokenFactor,
    'UpdateTargetChainFactor' : TargetChainFactor,
  });
  const MintTokenRequest = IDL.Record({
    'token_id' : IDL.Text,
    'ticket_id' : IDL.Text,
    'finalized_block_index' : IDL.Opt(IDL.Nat64),
    'amount' : IDL.Nat,
    'receiver' : IDL.Principal,
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'icon' : IDL.Opt(IDL.Text),
    'issue_chain' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const ToggleAction = IDL.Variant({
    'Deactivate' : IDL.Null,
    'Activate' : IDL.Null,
  });
  const ToggleState = IDL.Record({
    'action' : ToggleAction,
    'chain_id' : IDL.Text,
  });
  const Event = IDL.Variant({
    'finalized_gen_ticket' : IDL.Record({
      'block_index' : IDL.Nat64,
      'request' : GenerateTicketReq,
    }),
    'updated_fee' : IDL.Record({ 'fee' : Factor }),
    'finalized_mint_token' : MintTokenRequest,
    'added_token' : IDL.Record({
      'token' : Token,
      'ledger_id' : IDL.Principal,
    }),
    'added_chain' : Chain,
    'toggle_chain_state' : ToggleState,
  });
  const Log = IDL.Record({ 'log' : IDL.Text, 'offset' : IDL.Nat64 });
  const Logs = IDL.Record({
    'logs' : IDL.Vec(Log),
    'all_logs_count' : IDL.Nat64,
  });
  const MintTokenStatus = IDL.Variant({
    'Finalized' : GenerateTicketOk,
    'Unknown' : IDL.Null,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_events' : IDL.Func([GetEventsArg], [IDL.Vec(Event)], ['query']),
    'get_fee_account' : IDL.Func(
        [IDL.Opt(IDL.Principal)],
        [IDL.Vec(IDL.Nat8)],
        ['query'],
      ),
    'get_log_records' : IDL.Func([IDL.Nat64, IDL.Nat64], [Logs], ['query']),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat64)], ['query']),
    'get_token_ledger' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(Token)], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [MintTokenStatus], ['query']),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Record({}),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
