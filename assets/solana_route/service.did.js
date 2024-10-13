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
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
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
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const TxStatus = IDL.Variant({
    'New' : IDL.Null,
    'Finalized' : IDL.Null,
    'TxFailed' : IDL.Record({ 'e' : IDL.Text }),
    'Pending' : IDL.Null,
  });
  const Reason = IDL.Variant({
    'QueueIsFull' : IDL.Null,
    'CanisterError' : IDL.Text,
    'OutOfCycles' : IDL.Null,
    'Rejected' : IDL.Text,
  });
  const CallError = IDL.Record({ 'method' : IDL.Text, 'reason' : Reason });
  const Result_1 = IDL.Variant({ 'Ok' : TxStatus, 'Err' : CallError });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Opt(IDL.Text), 'Err' : CallError });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee_account' : IDL.Func([], [IDL.Text], ['query']),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat)], ['query']),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [Result_1], ['query']),
    'mint_token_tx_hash' : IDL.Func([IDL.Text], [Result_2], ['query']),
    'query_mint_address' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
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
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
