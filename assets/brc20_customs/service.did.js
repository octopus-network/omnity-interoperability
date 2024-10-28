export const idlFactory = ({ IDL }) => {
  const Network_1 = IDL.Variant({
    'mainnet' : IDL.Null,
    'local' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'network' : Network_1,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
    'indexer_principal' : IDL.Principal,
  });
  const LockTicketRequest = IDL.Record({
    'received_at' : IDL.Nat64,
    'ticker' : IDL.Text,
    'token_id' : IDL.Text,
    'txid' : IDL.Vec(IDL.Nat8),
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const ECDSAPublicKey = IDL.Record({
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
  const Network = IDL.Variant({
    'mainnet' : IDL.Null,
    'regtest' : IDL.Null,
    'testnet' : IDL.Null,
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
    'finalized_lock_ticket_requests' : IDL.Vec(
      IDL.Tuple(IDL.Vec(IDL.Nat8), LockTicketRequest)
    ),
    'next_consume_directive_seq' : IDL.Nat64,
    'hub_principal' : IDL.Principal,
    'ecdsa_key_name' : IDL.Text,
    'deposit_addr' : IDL.Opt(IDL.Text),
    'next_directive_seq' : IDL.Nat64,
    'ecdsa_public_key' : IDL.Opt(ECDSAPublicKey),
    'chain_id' : IDL.Text,
    'pending_lock_ticket_requests' : IDL.Vec(
      IDL.Tuple(IDL.Vec(IDL.Nat8), LockTicketRequest)
    ),
    'tokens' : IDL.Vec(IDL.Tuple(IDL.Text, Token)),
    'btc_network' : Network,
    'admins' : IDL.Vec(IDL.Principal),
    'counterparties' : IDL.Vec(IDL.Tuple(IDL.Text, Chain)),
    'next_ticket_seq' : IDL.Nat64,
    'chain_state' : ChainState,
    'min_confirmations' : IDL.Nat8,
    'indexer_principal' : IDL.Principal,
    'deposit_pubkey' : IDL.Opt(IDL.Text),
  });
  const GenerateTicketArgs = IDL.Record({
    'token_id' : IDL.Text,
    'txid' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const GenerateTicketError = IDL.Variant({
    'SendTicketErr' : IDL.Text,
    'RpcError' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'AlreadyProcessed' : IDL.Null,
    'OrdTxError' : IDL.Text,
    'NotBridgeTx' : IDL.Null,
    'AmountIsZero' : IDL.Null,
    'InvalidRuneId' : IDL.Text,
    'InvalidArgs' : IDL.Text,
    'AlreadySubmitted' : IDL.Null,
    'InvalidTxId' : IDL.Null,
    'TxNotFoundInMemPool' : IDL.Null,
    'Unknown' : IDL.Null,
    'NoNewUtxos' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : GenerateTicketError });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const ReleaseTokenStatus = IDL.Variant({
    'Signing' : IDL.Null,
    'Confirmed' : IDL.Text,
    'Sending' : IDL.Text,
    'Unknown' : IDL.Null,
    'Submitted' : IDL.Text,
    'Pending' : IDL.Null,
  });
  const UtxoArgs = IDL.Record({
    'id' : IDL.Text,
    'index' : IDL.Nat32,
    'amount' : IDL.Nat64,
  });
  return IDL.Service({
    'brc20_state' : IDL.Func([], [StateProfile], ['query']),
    'finalize_lock_request' : IDL.Func([IDL.Text], [], []),
    'finalized_unlock_tickets' : IDL.Func([IDL.Nat64], [IDL.Text], ['query']),
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'get_deposit_addr' : IDL.Func([], [IDL.Text, IDL.Text], []),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'pending_unlock_tickets' : IDL.Func([IDL.Nat64], [IDL.Text], ['query']),
    'release_token_status' : IDL.Func(
        [IDL.Text],
        [ReleaseTokenStatus],
        ['query'],
      ),
    'resend_unlock_ticket' : IDL.Func([IDL.Nat64, IDL.Nat64], [IDL.Text], []),
    'update_fees' : IDL.Func([IDL.Vec(UtxoArgs)], [], []),
  });
};
export const init = ({ IDL }) => {
  const Network_1 = IDL.Variant({
    'mainnet' : IDL.Null,
    'local' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'network' : Network_1,
    'chain_id' : IDL.Text,
    'admins' : IDL.Vec(IDL.Principal),
    'indexer_principal' : IDL.Principal,
  });
  return [InitArgs];
};
