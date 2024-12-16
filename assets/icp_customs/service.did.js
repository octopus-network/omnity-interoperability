export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
  });
  const GenerateTicketReq = IDL.Record({
    'token_id' : IDL.Text,
    'memo' : IDL.Opt(IDL.Text),
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
    'CustomError' : IDL.Text,
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
  const CustomsState = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'is_timer_running' : IDL.Bool,
    'next_directive_seq' : IDL.Nat64,
    'ckbtc_minter_principal' : IDL.Opt(IDL.Principal),
    'icp_token_id' : IDL.Opt(IDL.Text),
    'chain_id' : IDL.Text,
    'next_ticket_seq' : IDL.Nat64,
    'ckbtc_token_id' : IDL.Opt(IDL.Text),
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const MintTokenStatus = IDL.Variant({
    'Finalized' : IDL.Record({ 'tx_hash' : IDL.Text }),
    'Unknown' : IDL.Null,
  });
  const IcpChainKeyToken = IDL.Variant({ 'CKBTC' : IDL.Null });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
    'RedeemIcpChainKeyAssets' : IcpChainKeyToken,
    'Transfer' : IDL.Null,
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
    'Ok' : IDL.Tuple(IDL.Nat64, IDL.Nat64),
    'Err' : IDL.Text,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result], []),
    'generate_ticket_v2' : IDL.Func([GenerateTicketReq], [Result], []),
    'get_account_identifier' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(IDL.Nat8)],
        ['query'],
      ),
    'get_account_identifier_text' : IDL.Func(
        [IDL.Principal],
        [IDL.Text],
        ['query'],
      ),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_state' : IDL.Func([], [CustomsState], ['query']),
    'get_token_list' : IDL.Func([], [IDL.Vec(Token)], ['query']),
    'handle_ticket' : IDL.Func([IDL.Nat64], [Result_1], []),
    'mint_token_status' : IDL.Func([IDL.Text], [MintTokenStatus], ['query']),
    'query_hub_tickets' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket))],
        [],
      ),
    'refund_icp' : IDL.Func([IDL.Principal], [Result_2], []),
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
