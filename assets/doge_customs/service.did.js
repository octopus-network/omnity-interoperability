export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'fee_token' : IDL.Text,
    'hub_principal' : IDL.Principal,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
  });
  const GenerateTicketArgs = IDL.Record({
    'token_id' : IDL.Text,
    'txid' : IDL.Text,
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
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : CustomsError });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : CustomsError });
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
  const RpcConfig = IDL.Record({
    'url' : IDL.Text,
    'api_key' : IDL.Opt(IDL.Text),
  });
  const SendTicketResult = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'success' : IDL.Bool,
    'time_at' : IDL.Nat64,
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
    'counterparties' : IDL.Vec(IDL.Tuple(IDL.Text, Chain)),
    'min_deposit_amount' : IDL.Nat64,
    'next_ticket_seq' : IDL.Nat64,
    'chain_state' : ChainState,
    'min_confirmations' : IDL.Nat32,
    'default_rpc_config' : RpcConfig,
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
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'finalized_unlock_tickets' : IDL.Func([IDL.Nat64], [IDL.Text], ['query']),
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'get_deposit_address' : IDL.Func(
        [IDL.Text, IDL.Text],
        [Result_1],
        ['query'],
      ),
    'get_platform_fee' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Nat), IDL.Opt(IDL.Text)],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'init_ecdsa_public_key' : IDL.Func([], [Result], []),
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
        [Result_2],
        [],
      ),
    'set_fee_collector' : IDL.Func([IDL.Text], [], []),
    'set_min_deposit_amount' : IDL.Func([IDL.Nat64], [], []),
    'set_rpc_config' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [], []),
    'test_http' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Opt(IDL.Text)],
        [Result_1],
        [],
      ),
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
