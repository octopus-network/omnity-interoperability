export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
  });
  const GenerateTicketArgs = IDL.Record({
    'token_id' : IDL.Text,
    'sender' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'tx_hash' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
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
  const Result = IDL.Variant({ 'Ok' : Ticket, 'Err' : IDL.Text });
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
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'ton_contract' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const MintTokenStatus = IDL.Variant({
    'Finalized' : IDL.Record({ 'tx_hash' : IDL.Text }),
    'Unknown' : IDL.Null,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Int32, 'Err' : IDL.Text });
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
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
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
  const Directive = IDL.Variant({
    'UpdateChain' : Chain,
    'UpdateFee' : Factor,
    'AddToken' : Token,
    'AddChain' : Chain,
    'ToggleChainState' : ToggleState,
    'UpdateToken' : Token,
  });
  const PendingDirectiveStatus = IDL.Record({
    'seq' : IDL.Nat64,
    'ton_tx_hash' : IDL.Opt(IDL.Text),
    'error' : IDL.Opt(IDL.Text),
  });
  const PendingTicketStatus = IDL.Record({
    'seq' : IDL.Nat64,
    'pending_time' : IDL.Nat64,
    'ticket_id' : IDL.Text,
    'ton_tx_hash' : IDL.Opt(IDL.Text),
    'error' : IDL.Opt(IDL.Text),
  });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Opt(IDL.Text), 'Err' : IDL.Text });
  const StateProfile = IDL.Record({
    'next_consume_ticket_seq' : IDL.Nat64,
    'next_consume_directive_seq' : IDL.Nat64,
    'hub_principal' : IDL.Principal,
    'last_success_seqno' : IDL.Int32,
    'token_contracts' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'next_directive_seq' : IDL.Nat64,
    'pubkey' : IDL.Vec(IDL.Nat8),
    'omnity_chain_id' : IDL.Text,
    'tokens' : IDL.Vec(IDL.Tuple(IDL.Text, Token)),
    'admins' : IDL.Vec(IDL.Principal),
    'target_chain_factor' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat)),
    'counterparties' : IDL.Vec(IDL.Tuple(IDL.Text, Chain)),
    'next_ticket_seq' : IDL.Nat64,
    'fee_token_factor' : IDL.Opt(IDL.Nat),
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponse,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat64), IDL.Text], ['query']),
    'get_ticket' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [MintTokenStatus], ['query']),
    'pubkey_and_ton_addr' : IDL.Func([], [IDL.Text, IDL.Text], []),
    'query_account_seqno' : IDL.Func([IDL.Text], [Result_1], []),
    'query_directives' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Directive))],
        ['query'],
      ),
    'query_pending_directive' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, PendingDirectiveStatus))],
        ['query'],
      ),
    'query_pending_ticket' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, PendingTicketStatus))],
        ['query'],
      ),
    'query_tickets' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'resend_ticket' : IDL.Func([IDL.Nat64], [Result_2], []),
    'route_state' : IDL.Func([], [StateProfile], ['query']),
    'set_token_master' : IDL.Func([IDL.Text, IDL.Text], [], []),
    'transform' : IDL.Func([TransformArgs], [HttpResponse], ['query']),
    'update_admins' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'update_consume_directive_seq' : IDL.Func([IDL.Nat64], [], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
  });
  return [InitArgs];
};
