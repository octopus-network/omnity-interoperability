export const idlFactory = ({ IDL }) => {
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const RpcApi = IDL.Record({
    'url' : IDL.Text,
    'headers' : IDL.Opt(IDL.Vec(HttpHeader)),
  });
  const EvmTxType = IDL.Variant({ 'Eip1559' : IDL.Null, 'Legacy' : IDL.Null });
  const Network = IDL.Variant({
    'mainnet' : IDL.Null,
    'local' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'evm_chain_id' : IDL.Nat64,
    'hub_principal' : IDL.Principal,
    'rpcs' : IDL.Vec(RpcApi),
    'evm_tx_type' : EvmTxType,
    'network' : Network,
    'fee_token_id' : IDL.Text,
    'block_interval_secs' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
    'evm_rpc_canister_addr' : IDL.Principal,
    'port_addr' : IDL.Opt(IDL.Text),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
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
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'evm_contract' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'status_code' : IDL.Nat16,
  });
  const MetricsStatus = IDL.Record({
    'chainkey_addr_balance' : IDL.Nat,
    'latest_scan_interval_secs' : IDL.Nat64,
  });
  const MintTokenStatus = IDL.Variant({
    'Finalized' : IDL.Record({ 'tx_hash' : IDL.Text }),
    'Unknown' : IDL.Null,
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
  const PendingDirectiveStatus = IDL.Record({
    'seq' : IDL.Nat64,
    'evm_tx_hash' : IDL.Opt(IDL.Text),
    'error' : IDL.Opt(IDL.Text),
  });
  const PendingTicketStatus = IDL.Record({
    'seq' : IDL.Nat64,
    'evm_tx_hash' : IDL.Opt(IDL.Text),
    'ticket_id' : IDL.Text,
    'error' : IDL.Opt(IDL.Text),
  });
  const EcdsaCurve = IDL.Variant({ 'secp256k1' : IDL.Null });
  const EcdsaKeyId = IDL.Record({ 'name' : IDL.Text, 'curve' : EcdsaCurve });
  const StateProfile = IDL.Record({
    'next_consume_ticket_seq' : IDL.Nat64,
    'evm_chain_id' : IDL.Nat64,
    'finality_blocks' : IDL.Opt(IDL.Nat64),
    'omnity_port_contract' : IDL.Vec(IDL.Nat8),
    'evm_gasfee_percent' : IDL.Nat64,
    'next_consume_directive_seq' : IDL.Nat64,
    'hub_principal' : IDL.Principal,
    'key_id' : EcdsaKeyId,
    'token_contracts' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'next_directive_seq' : IDL.Nat64,
    'evm_tx_type' : EvmTxType,
    'pubkey' : IDL.Vec(IDL.Nat8),
    'key_derivation_path' : IDL.Vec(IDL.Vec(IDL.Nat8)),
    'omnity_chain_id' : IDL.Text,
    'tokens' : IDL.Vec(IDL.Tuple(IDL.Text, Token)),
    'admins' : IDL.Vec(IDL.Principal),
    'target_chain_factor' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat)),
    'evm_rpc_addr' : IDL.Principal,
    'counterparties' : IDL.Vec(IDL.Tuple(IDL.Text, Chain)),
    'next_ticket_seq' : IDL.Nat64,
    'rpc_providers' : IDL.Vec(RpcApi),
    'chain_state' : ChainState,
    'minimum_response_count' : IDL.Nat64,
    'total_required_count' : IDL.Nat64,
    'fee_token_factor' : IDL.Opt(IDL.Nat),
  });
  const HttpResponse_1 = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponse_1,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([IDL.Text], [Result], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat64)], ['query']),
    'get_ticket' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'insert_pending_hash' : IDL.Func([IDL.Text], [], []),
    'metrics' : IDL.Func([], [MetricsStatus], []),
    'mint_token_status' : IDL.Func([IDL.Text], [MintTokenStatus], ['query']),
    'pubkey_and_evm_addr' : IDL.Func([], [IDL.Text, IDL.Text], []),
    'query_directives' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Directive))],
        ['query'],
      ),
    'query_handled_event' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], []),
    'query_hub_tickets' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket))],
        [],
      ),
    'query_pending_directive' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, PendingDirectiveStatus))],
        ['query'],
      ),
    'query_pending_ticket' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Text, PendingTicketStatus))],
        ['query'],
      ),
    'query_tickets' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'resend_directive' : IDL.Func([IDL.Nat64], [], []),
    'resend_ticket' : IDL.Func([IDL.Nat64], [], []),
    'rewrite_tx_hash' : IDL.Func([IDL.Text, IDL.Text], [], []),
    'route_state' : IDL.Func([], [StateProfile], ['query']),
    'set_finality_blocks' : IDL.Func([IDL.Nat64], [], []),
    'set_port_address' : IDL.Func([IDL.Text], [], []),
    'sync_mint_status' : IDL.Func([IDL.Text], [], []),
    'transform' : IDL.Func([TransformArgs], [HttpResponse_1], ['query']),
    'update_admins' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'update_consume_directive_seq' : IDL.Func([IDL.Nat64], [], []),
    'update_fee_token' : IDL.Func([IDL.Text], [], []),
    'update_rpc_check_rate' : IDL.Func([IDL.Nat64, IDL.Nat64], [], []),
    'update_rpcs' : IDL.Func([IDL.Vec(RpcApi)], [], []),
  });
};
export const init = ({ IDL }) => {
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const RpcApi = IDL.Record({
    'url' : IDL.Text,
    'headers' : IDL.Opt(IDL.Vec(HttpHeader)),
  });
  const EvmTxType = IDL.Variant({ 'Eip1559' : IDL.Null, 'Legacy' : IDL.Null });
  const Network = IDL.Variant({
    'mainnet' : IDL.Null,
    'local' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'evm_chain_id' : IDL.Nat64,
    'hub_principal' : IDL.Principal,
    'rpcs' : IDL.Vec(RpcApi),
    'evm_tx_type' : EvmTxType,
    'network' : Network,
    'fee_token_id' : IDL.Text,
    'block_interval_secs' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
    'evm_rpc_canister_addr' : IDL.Principal,
    'port_addr' : IDL.Opt(IDL.Text),
  });
  return [InitArgs];
};
