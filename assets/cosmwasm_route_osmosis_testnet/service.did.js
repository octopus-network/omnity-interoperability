export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'cw_rpc_url' : IDL.Text,
    'cw_rest_url' : IDL.Text,
    'chain_id' : IDL.Text,
    'cosmwasm_port_contract_address' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
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
  const RouteState = IDL.Record({
    'hub_principal' : IDL.Principal,
    'cw_rpc_url' : IDL.Text,
    'cw_chain_key_derivation_path' : IDL.Vec(IDL.Vec(IDL.Nat8)),
    'is_timer_running' : IDL.Vec(IDL.Text),
    'next_directive_seq' : IDL.Nat64,
    'cw_rest_url' : IDL.Text,
    'cw_public_key_vec' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'chain_id' : IDL.Text,
    'cw_port_contract_address' : IDL.Text,
    'processing_tickets' : IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket)),
    'next_ticket_seq' : IDL.Nat64,
    'chain_state' : ChainState,
    'processing_directive' : IDL.Vec(IDL.Tuple(IDL.Nat64, Directive)),
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const Result_1 = IDL.Variant({ 'Ok' : HttpResponse, 'Err' : IDL.Text });
  const UpdateCwSettingsArgs = IDL.Record({
    'cw_rpc_url' : IDL.Opt(IDL.Text),
    'cw_rest_url' : IDL.Opt(IDL.Text),
    'cw_port_contract_address' : IDL.Opt(IDL.Text),
  });
  return IDL.Service({
    'cache_public_key_and_start_timer' : IDL.Func([], [], []),
    'osmosis_account_id' : IDL.Func([], [Result], []),
    'redeem' : IDL.Func([IDL.Text], [Result], []),
    'route_status' : IDL.Func([], [RouteState], ['query']),
    'test_execute_directive' : IDL.Func([IDL.Text, Directive], [Result], []),
    'test_http_outcall' : IDL.Func([IDL.Text], [Result_1], []),
    'update_cw_settings' : IDL.Func([UpdateCwSettingsArgs], [], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'cw_rpc_url' : IDL.Text,
    'cw_rest_url' : IDL.Text,
    'chain_id' : IDL.Text,
    'cosmwasm_port_contract_address' : IDL.Text,
  });
  return [InitArgs];
};
