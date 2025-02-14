export const idlFactory = ({ IDL }) => {
  const Provider = IDL.Variant({
    'Mainnet' : IDL.Null,
    'Custom' : IDL.Tuple(IDL.Text, IDL.Text),
    'Testnet' : IDL.Null,
    'Devnet' : IDL.Null,
    'Localnet' : IDL.Null,
  });
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'admin' : IDL.Opt(IDL.Principal),
    'hub_principal' : IDL.Opt(IDL.Principal),
    'gas_budget' : IDL.Opt(IDL.Nat64),
    'fee_account' : IDL.Opt(IDL.Text),
    'rpc_provider' : IDL.Opt(Provider),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
    'nodes_in_subnet' : IDL.Opt(IDL.Nat32),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'gas_budget' : IDL.Opt(IDL.Nat64),
    'fee_account' : IDL.Text,
    'rpc_provider' : IDL.Opt(Provider),
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
    'nodes_in_subnet' : IDL.Opt(IDL.Nat32),
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const RpcError = IDL.Variant({
    'Text' : IDL.Text,
    'ParseError' : IDL.Text,
    'RpcResponseError' : IDL.Record({
      'code' : IDL.Int64,
      'data' : IDL.Opt(IDL.Text),
      'message' : IDL.Text,
    }),
    'RpcRequestError' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : RpcError });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : RpcError });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
    'Transfer' : IDL.Null,
  });
  const GenerateTicketReq = IDL.Record({
    'action' : TxAction,
    'token_id' : IDL.Text,
    'memo' : IDL.Opt(IDL.Text),
    'sender' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'digest' : IDL.Text,
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
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : RpcError });
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
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : RpcError });
  const SuiPortAction = IDL.Record({
    'package' : IDL.Text,
    'upgrade_cap' : IDL.Text,
    'ticket_table' : IDL.Text,
    'port_owner_cap' : IDL.Text,
    'functions' : IDL.Vec(IDL.Text),
    'module' : IDL.Text,
  });
  const Permission = IDL.Variant({ 'Update' : IDL.Null, 'Query' : IDL.Null });
  const TaskType = IDL.Variant({
    'GetTickets' : IDL.Null,
    'ClearTicket' : IDL.Null,
    'BurnToken' : IDL.Null,
    'GetDirectives' : IDL.Null,
    'MintToken' : IDL.Null,
    'UpdateToken' : IDL.Null,
  });
  const Seqs = IDL.Record({
    'next_directive_seq' : IDL.Nat64,
    'next_ticket_seq' : IDL.Nat64,
  });
  const MultiRpcConfig = IDL.Record({
    'rpc_list' : IDL.Vec(IDL.Text),
    'minimum_response_count' : IDL.Nat32,
  });
  const KeyType = IDL.Variant({
    'Native' : IDL.Vec(IDL.Nat8),
    'ChainKey' : IDL.Null,
  });
  const SuiRouteConfig = IDL.Record({
    'sui_port_action' : SuiPortAction,
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'caller_perms' : IDL.Vec(IDL.Tuple(IDL.Text, Permission)),
    'active_tasks' : IDL.Vec(TaskType),
    'gas_budget' : IDL.Nat64,
    'enable_debug' : IDL.Bool,
    'fee_account' : IDL.Text,
    'seqs' : Seqs,
    'rpc_provider' : Provider,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Text,
    'target_chain_factor' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat)),
    'multi_rpc_config' : MultiRpcConfig,
    'key_type' : KeyType,
    'chain_state' : ChainState,
    'forward' : IDL.Opt(IDL.Text),
    'nodes_in_subnet' : IDL.Nat32,
    'fee_token_factor' : IDL.Opt(IDL.Nat),
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
  const MintTokenRequest = IDL.Record({
    'status' : TxStatus,
    'object' : IDL.Opt(IDL.Text),
    'token_id' : IDL.Text,
    'recipient' : IDL.Text,
    'ticket_id' : IDL.Text,
    'digest' : IDL.Opt(IDL.Text),
    'amount' : IDL.Nat64,
    'retry' : IDL.Nat64,
  });
  const Reason = IDL.Variant({
    'QueueIsFull' : IDL.Null,
    'CanisterError' : IDL.Text,
    'OutOfCycles' : IDL.Null,
    'Rejected' : IDL.Text,
    'TxError' : IDL.Text,
  });
  const CallError = IDL.Record({ 'method' : IDL.Text, 'reason' : Reason });
  const Result_5 = IDL.Variant({ 'Ok' : MintTokenRequest, 'Err' : CallError });
  const Result_6 = IDL.Variant({ 'Ok' : TxStatus, 'Err' : CallError });
  const Result_7 = IDL.Variant({ 'Ok' : IDL.Opt(IDL.Text), 'Err' : CallError });
  const Result_8 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : RpcError });
  const SnorKeyType = IDL.Variant({
    'Native' : IDL.Null,
    'ChainKey' : IDL.Null,
  });
  const Result_9 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const SuiToken = IDL.Record({
    'treasury_cap' : IDL.Text,
    'metadata' : IDL.Text,
    'package' : IDL.Text,
    'upgrade_cap' : IDL.Text,
    'functions' : IDL.Vec(IDL.Text),
    'module' : IDL.Text,
    'type_tag' : IDL.Text,
  });
  const Result_10 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const UpdateType = IDL.Variant({
    'Symbol' : IDL.Text,
    'Icon' : IDL.Text,
    'Name' : IDL.Text,
    'Description' : IDL.Text,
  });
  return IDL.Service({
    'add_token' : IDL.Func([Token], [IDL.Opt(Token)], []),
    'burn_token' : IDL.Func([IDL.Text, IDL.Text], [Result], []),
    'check_object_exists' : IDL.Func([IDL.Text, IDL.Text], [Result_1], []),
    'create_ticket_table' : IDL.Func([IDL.Text], [Result], []),
    'drop_ticket_table' : IDL.Func([], [Result], []),
    'fetch_coin' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Nat64],
        [Result],
        [],
      ),
    'forward' : IDL.Func([], [IDL.Opt(IDL.Text)], ['query']),
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result_2], []),
    'get_balance' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [Result_3], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_coins' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [Result], []),
    'get_events' : IDL.Func([IDL.Text], [Result], []),
    'get_fee_account' : IDL.Func([], [IDL.Text], ['query']),
    'get_gas_budget' : IDL.Func([], [IDL.Nat64], []),
    'get_gas_price' : IDL.Func([], [Result_4], []),
    'get_object' : IDL.Func([IDL.Text], [Result], []),
    'get_owner_objects' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [Result], []),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat)], ['query']),
    'get_route_config' : IDL.Func([], [SuiRouteConfig], ['query']),
    'get_token' : IDL.Func([IDL.Text], [IDL.Opt(Token)], ['query']),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'get_transaction_block' : IDL.Func([IDL.Text], [Result], []),
    'merge_coin' : IDL.Func([IDL.Text, IDL.Vec(IDL.Text)], [Result], []),
    'mint_to_with_ticket' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text, IDL.Nat64],
        [Result],
        [],
      ),
    'mint_token' : IDL.Func([IDL.Text, IDL.Text, IDL.Nat64], [Result], []),
    'mint_token_req' : IDL.Func([IDL.Text], [Result_5], ['query']),
    'mint_token_reqs' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(MintTokenRequest)],
        ['query'],
      ),
    'mint_token_status' : IDL.Func([IDL.Text], [Result_6], ['query']),
    'mint_token_tx_hash' : IDL.Func([IDL.Text], [Result_7], ['query']),
    'parse_redeem_events' : IDL.Func([IDL.Text], [Result_8], []),
    'remove_ticket_from_port' : IDL.Func([IDL.Text], [Result], []),
    'rpc_provider' : IDL.Func([], [Provider], ['query']),
    'split_coin' : IDL.Func([IDL.Text, IDL.Nat64, IDL.Text], [Result], []),
    'sui_port_action' : IDL.Func([], [SuiPortAction], ['query']),
    'sui_route_address' : IDL.Func([SnorKeyType], [Result_9], []),
    'sui_sign' : IDL.Func([IDL.Vec(IDL.Nat8), SnorKeyType], [Result_9], []),
    'sui_token' : IDL.Func([IDL.Text], [IDL.Opt(SuiToken)], ['query']),
    'transfer_objects' : IDL.Func([IDL.Text, IDL.Vec(IDL.Text)], [Result], []),
    'transfer_sui' : IDL.Func([IDL.Text, IDL.Nat64], [Result], []),
    'update_gas_budget' : IDL.Func([IDL.Nat64], [], []),
    'update_mint_token_req' : IDL.Func([MintTokenRequest], [Result_5], []),
    'update_rpc_provider' : IDL.Func([Provider], [], []),
    'update_sui_port_action' : IDL.Func([SuiPortAction], [], []),
    'update_sui_token' : IDL.Func([IDL.Text, SuiToken], [Result_10], []),
    'update_token_meta' : IDL.Func([IDL.Text, UpdateType], [Result], []),
  });
};
export const init = ({ IDL }) => {
  const Provider = IDL.Variant({
    'Mainnet' : IDL.Null,
    'Custom' : IDL.Tuple(IDL.Text, IDL.Text),
    'Testnet' : IDL.Null,
    'Devnet' : IDL.Null,
    'Localnet' : IDL.Null,
  });
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'admin' : IDL.Opt(IDL.Principal),
    'hub_principal' : IDL.Opt(IDL.Principal),
    'gas_budget' : IDL.Opt(IDL.Nat64),
    'fee_account' : IDL.Opt(IDL.Text),
    'rpc_provider' : IDL.Opt(Provider),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
    'nodes_in_subnet' : IDL.Opt(IDL.Nat32),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'gas_budget' : IDL.Opt(IDL.Nat64),
    'fee_account' : IDL.Text,
    'rpc_provider' : IDL.Opt(Provider),
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
    'nodes_in_subnet' : IDL.Opt(IDL.Nat32),
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
