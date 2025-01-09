export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'fee_token' : IDL.Text,
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
  });
  const GenerateTicketArgs = IDL.Record({
    'token_id' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const Destination = IDL.Record({
    'token' : IDL.Opt(IDL.Text),
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const CustomsError = IDL.Variant({
    'SendTicketErr' : IDL.Text,
    'RpcError' : IDL.Text,
    'HttpOutExceedLimit' : IDL.Null,
    'TemporarilyUnavailable' : IDL.Text,
    'HttpOutCallError' : IDL.Tuple(IDL.Text, IDL.Text, IDL.Text),
    'AlreadyProcessed' : IDL.Null,
    'HttpStatusError' : IDL.Tuple(IDL.Nat, IDL.Text, IDL.Text),
    'OrdTxError' : IDL.Text,
    'NotBridgeTx' : IDL.Null,
    'AmountIsZero' : IDL.Null,
    'InvalidRuneId' : IDL.Text,
    'InvalidArgs' : IDL.Text,
    'AlreadySubmitted' : IDL.Null,
    'InvalidTxId' : IDL.Null,
    'NotPayFees' : IDL.Null,
    'CallError' : IDL.Tuple(IDL.Principal, IDL.Text, IDL.Text),
    'TxNotFoundInMemPool' : IDL.Null,
    'Unknown' : IDL.Null,
    'InvalidTxReceiver' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'ECDSAPublicKeyNotFound' : IDL.Null,
    'DepositUtxoNotFound' : IDL.Tuple(IDL.Text, Destination),
    'UnsupportedToken' : IDL.Text,
    'CustomError' : IDL.Text,
  });
  const Result = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Text),
    'Err' : CustomsError,
  });
  const GenerateTicketWithTxidArgs = IDL.Record({
    'token_id' : IDL.Text,
    'txid' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : CustomsError });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : CustomsError });
  const SendTicketResult = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'success' : IDL.Bool,
    'time_at' : IDL.Nat64,
  });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const LockTicketRequest = IDL.Record({
    'received_at' : IDL.Nat64,
    'transaction_hex' : IDL.Text,
    'token_id' : IDL.Text,
    'txid' : IDL.Vec(IDL.Nat8),
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const Utxo = IDL.Record({
    'value' : IDL.Nat64,
    'txid' : IDL.Vec(IDL.Nat8),
    'vout' : IDL.Nat32,
  });
  const EcdsaPublicKeyResponse = IDL.Record({
    'public_key' : IDL.Vec(IDL.Nat8),
    'chain_code' : IDL.Vec(IDL.Nat8),
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const RpcConfig = IDL.Record({
    'url' : IDL.Text,
    'api_key' : IDL.Opt(IDL.Text),
  });
  const MultiRpcConfig = IDL.Record({
    'rpc_list' : IDL.Vec(RpcConfig),
    'minimum_response_count' : IDL.Nat32,
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
  const StateProfile = IDL.Record({
    'next_consume_ticket_seq' : IDL.Nat64,
    'fee_token' : IDL.Text,
    'hub_principal' : IDL.Principal,
    'ecdsa_key_name' : IDL.Text,
    'doge_chain' : IDL.Nat8,
    'next_directive_seq' : IDL.Nat64,
    'doge_fee_rate' : IDL.Opt(IDL.Nat64),
    'deposited_utxo' : IDL.Vec(IDL.Tuple(Utxo, Destination)),
    'fee_collector' : IDL.Text,
    'ecdsa_public_key' : IDL.Opt(EcdsaPublicKeyResponse),
    'chain_id' : IDL.Text,
    'pending_lock_ticket_requests' : IDL.Vec(
      IDL.Tuple(IDL.Text, LockTicketRequest)
    ),
    'tokens' : IDL.Vec(IDL.Tuple(IDL.Text, Token)),
    'admins' : IDL.Vec(IDL.Principal),
    'target_chain_factor' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat)),
    'multi_rpc_config' : MultiRpcConfig,
    'counterparties' : IDL.Vec(IDL.Tuple(IDL.Text, Chain)),
    'min_deposit_amount' : IDL.Nat64,
    'next_ticket_seq' : IDL.Nat64,
    'chain_state' : ChainState,
    'min_confirmations' : IDL.Nat32,
    'tatum_rpc_config' : RpcConfig,
    'fee_payment_utxo' : IDL.Vec(Utxo),
    'flight_unlock_ticket_map' : IDL.Vec(
      IDL.Tuple(IDL.Nat64, SendTicketResult)
    ),
    'fee_token_factor' : IDL.Opt(IDL.Nat),
  });
  const ReleaseTokenStatus = IDL.Variant({
    'Signing' : IDL.Null,
    'Confirmed' : IDL.Text,
    'Sending' : IDL.Text,
    'Unknown' : IDL.Null,
    'Submitted' : IDL.Text,
    'Pending' : IDL.Null,
  });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : CustomsError });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'generate_ticket_by_txid' : IDL.Func(
        [GenerateTicketWithTxidArgs],
        [Result_1],
        [],
      ),
    'get_deposit_address' : IDL.Func(
        [IDL.Text, IDL.Text],
        [Result_2],
        ['query'],
      ),
    'get_fee_payment_address' : IDL.Func([], [Result_2], ['query']),
    'get_finalized_lock_ticket_txids' : IDL.Func(
        [],
        [IDL.Vec(IDL.Text)],
        ['query'],
      ),
    'get_finalized_unlock_ticket_results' : IDL.Func(
        [],
        [IDL.Vec(SendTicketResult)],
        ['query'],
      ),
    'get_platform_fee' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Nat), IDL.Opt(IDL.Text)],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'init_ecdsa_public_key' : IDL.Func([], [Result_1], []),
    'pending_unlock_tickets' : IDL.Func([IDL.Nat64], [IDL.Text], ['query']),
    'query_finalized_lock_tickets' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(LockTicketRequest)],
        ['query'],
      ),
    'query_state' : IDL.Func([], [StateProfile], ['query']),
    'release_token_status' : IDL.Func(
        [IDL.Text],
        [ReleaseTokenStatus],
        ['query'],
      ),
    'resend_unlock_ticket' : IDL.Func(
        [IDL.Nat64, IDL.Opt(IDL.Nat64)],
        [Result_3],
        [],
      ),
    'save_utxo_for_payment_address' : IDL.Func([IDL.Text], [Result_4], []),
    'set_default_doge_rpc_config' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text)],
        [],
        [],
      ),
    'set_fee_collector' : IDL.Func([IDL.Text], [], []),
    'set_min_deposit_amount' : IDL.Func([IDL.Nat64], [], []),
    'set_tatum_api_config' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [], []),
    'tmp_fix' : IDL.Func([], [], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'fee_token' : IDL.Text,
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
  });
  return [InitArgs];
};
