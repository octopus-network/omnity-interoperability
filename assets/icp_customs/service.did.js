export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
  });
  const GenerateTicketReq = IDL.Record({
    'token_id' : IDL.Text,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
  });
  const GenerateTicketOk = IDL.Record({ 'ticket_id' : IDL.Text });
  const GenerateTicketError = IDL.Variant({
    'SendTicketErr' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'InsufficientIcp' : IDL.Record({
      'provided' : IDL.Nat64,
      'required' : IDL.Nat64,
    }),
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat64 }),
    'TransferIcpFailure' : IDL.Text,
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
    'canister_id' : IDL.Text,
    'chain_id' : IDL.Text,
    'counterparties' : IDL.Opt(IDL.Vec(IDL.Text)),
    'chain_state' : ChainState,
    'chain_type' : ChainType,
    'contract_address' : IDL.Opt(IDL.Text),
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_token_list' : IDL.Func([], [IDL.Vec(Token)], ['query']),
    'set_ckbtc_token' : IDL.Func([IDL.Text], [], []),
    'set_icp_token' : IDL.Func([IDL.Text], [], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
  });
  return [InitArgs];
};
