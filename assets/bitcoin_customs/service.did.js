export const idlFactory = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'max_time_in_queue_nanos' : IDL.Opt(IDL.Nat64),
    'chain_state' : IDL.Opt(ChainState),
    'min_confirmations' : IDL.Opt(IDL.Nat32),
  });
  const BtcNetwork = IDL.Variant({
    'Mainnet' : IDL.Null,
    'Regtest' : IDL.Null,
    'Testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'ecdsa_key_name' : IDL.Text,
    'runes_oracle_principal' : IDL.Principal,
    'max_time_in_queue_nanos' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'btc_network' : BtcNetwork,
    'chain_state' : ChainState,
    'min_confirmations' : IDL.Opt(IDL.Nat32),
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const LogoParams = IDL.Record({
    'content_type' : IDL.Text,
    'content_base64' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IDL.Text });
  const RuneId = IDL.Record({ 'tx' : IDL.Nat32, 'block' : IDL.Nat64 });
  const EstimateFeeArgs = IDL.Record({
    'amount' : IDL.Opt(IDL.Nat),
    'rune_id' : RuneId,
  });
  const RedeemFee = IDL.Record({ 'bitcoin_fee' : IDL.Nat64 });
  const OrdinalsTerms = IDL.Record({
    'cap' : IDL.Nat,
    'height' : IDL.Tuple(IDL.Opt(IDL.Nat64), IDL.Opt(IDL.Nat64)),
    'offset' : IDL.Tuple(IDL.Opt(IDL.Nat64), IDL.Opt(IDL.Nat64)),
    'amount' : IDL.Nat,
  });
  const EtchingArgs = IDL.Record({
    'terms' : IDL.Opt(OrdinalsTerms),
    'turbo' : IDL.Bool,
    'premine' : IDL.Opt(IDL.Nat),
    'logo' : IDL.Opt(LogoParams),
    'rune_name' : IDL.Text,
    'divisibility' : IDL.Opt(IDL.Nat8),
    'symbol' : IDL.Opt(IDL.Text),
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const GenerateTicketArgs = IDL.Record({
    'txid' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
    'rune_id' : IDL.Text,
  });
  const GenerateTicketError = IDL.Variant({
    'SendTicketErr' : IDL.Text,
    'RpcError' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'AlreadyProcessed' : IDL.Null,
    'AmountIsZero' : IDL.Null,
    'InvalidRuneId' : IDL.Text,
    'AlreadySubmitted' : IDL.Null,
    'InvalidTxId' : IDL.Null,
    'NotPayFees' : IDL.Null,
    'TxNotFoundInMemPool' : IDL.Null,
    'NoNewUtxos' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
  });
  const Result_2 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : GenerateTicketError,
  });
  const OutPoint = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'vout' : IDL.Nat32,
  });
  const Utxo = IDL.Record({
    'height' : IDL.Nat32,
    'value' : IDL.Nat64,
    'outpoint' : OutPoint,
  });
  const GenTicketRequestV2 = IDL.Record({
    'received_at' : IDL.Nat64,
    'token_id' : IDL.Text,
    'new_utxos' : IDL.Vec(Utxo),
    'txid' : IDL.Vec(IDL.Nat8),
    'target_chain_id' : IDL.Text,
    'address' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
    'rune_id' : RuneId,
  });
  const GenTicketStatus = IDL.Variant({
    'Finalized' : GenTicketRequestV2,
    'Confirmed' : GenTicketRequestV2,
    'Unknown' : IDL.Null,
    'Pending' : GenTicketRequestV2,
  });
  const GetBtcAddressArgs = IDL.Record({
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const TokenPrice = IDL.Record({
    'name' : IDL.Text,
    'priceUSD' : IDL.Float64,
    'standard' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const CanisterStatusType = IDL.Variant({
    'stopped' : IDL.Null,
    'stopping' : IDL.Null,
    'running' : IDL.Null,
  });
  const DefiniteCanisterSettings = IDL.Record({
    'freezing_threshold' : IDL.Nat,
    'controllers' : IDL.Vec(IDL.Principal),
    'reserved_cycles_limit' : IDL.Nat,
    'memory_allocation' : IDL.Nat,
    'compute_allocation' : IDL.Nat,
  });
  const QueryStats = IDL.Record({
    'response_payload_bytes_total' : IDL.Nat,
    'num_instructions_total' : IDL.Nat,
    'num_calls_total' : IDL.Nat,
    'request_payload_bytes_total' : IDL.Nat,
  });
  const CanisterStatusResponse = IDL.Record({
    'status' : CanisterStatusType,
    'memory_size' : IDL.Nat,
    'cycles' : IDL.Nat,
    'settings' : DefiniteCanisterSettings,
    'query_stats' : QueryStats,
    'idle_cycles_burned_per_day' : IDL.Nat,
    'module_hash' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'reserved_cycles' : IDL.Nat,
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
  const EtchingAccountInfo = IDL.Record({
    'derive_path' : IDL.Text,
    'pubkey' : IDL.Text,
    'address' : IDL.Text,
  });
  const ECDSAPublicKey = IDL.Record({
    'public_key' : IDL.Vec(IDL.Nat8),
    'chain_code' : IDL.Vec(IDL.Nat8),
  });
  const Network = IDL.Variant({
    'mainnet' : IDL.Null,
    'regtest' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const CustomsInfo = IDL.Record({
    'runes_oracles' : IDL.Vec(IDL.Principal),
    'last_fee_per_vbyte' : IDL.Vec(IDL.Nat64),
    'etching_acount_info' : EtchingAccountInfo,
    'hub_principal' : IDL.Principal,
    'ecdsa_key_name' : IDL.Text,
    'next_directive_seq' : IDL.Nat64,
    'fee_collector_address' : IDL.Text,
    'icpswap_principal' : IDL.Opt(IDL.Principal),
    'ecdsa_public_key' : IDL.Opt(ECDSAPublicKey),
    'max_time_in_queue_nanos' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'rpc_url' : IDL.Opt(IDL.Text),
    'generate_ticket_counter' : IDL.Nat64,
    'btc_network' : Network,
    'target_chain_factor' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat)),
    'ord_indexer_principal' : IDL.Opt(IDL.Principal),
    'next_ticket_seq' : IDL.Nat64,
    'chain_state' : ChainState,
    'min_confirmations' : IDL.Nat32,
    'prod_ecdsa_public_key' : IDL.Opt(ECDSAPublicKey),
    'release_token_counter' : IDL.Nat64,
    'fee_token_factor' : IDL.Opt(IDL.Nat),
  });
  const EtchingStatus = IDL.Variant({
    'SendRevealSuccess' : IDL.Null,
    'SendRevealFailed' : IDL.Null,
    'SendCommitFailed' : IDL.Null,
    'TokenAdded' : IDL.Null,
    'SendCommitSuccess' : IDL.Null,
    'Final' : IDL.Null,
  });
  const SendEtchingInfo = IDL.Record({
    'status' : EtchingStatus,
    'script_out_address' : IDL.Text,
    'err_info' : IDL.Text,
    'commit_txid' : IDL.Text,
    'time_at' : IDL.Nat64,
    'etching_args' : EtchingArgs,
    'reveal_txid' : IDL.Text,
  });
  const UtxoArgs = IDL.Record({
    'id' : IDL.Text,
    'index' : IDL.Nat32,
    'amount' : IDL.Nat64,
  });
  const GetEventsArg = IDL.Record({
    'start' : IDL.Nat64,
    'length' : IDL.Nat64,
  });
  const Destination = IDL.Record({
    'token' : IDL.Opt(IDL.Text),
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
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
  const BtcChangeOutput = IDL.Record({
    'value' : IDL.Nat64,
    'vout' : IDL.Nat32,
  });
  const RunesChangeOutput = IDL.Record({
    'value' : IDL.Nat,
    'vout' : IDL.Nat32,
    'rune_id' : RuneId,
  });
  const RunesBalance = IDL.Record({
    'vout' : IDL.Nat32,
    'amount' : IDL.Nat,
    'rune_id' : RuneId,
  });
  const RunesUtxo = IDL.Record({ 'raw' : Utxo, 'runes' : RunesBalance });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const BitcoinAddress = IDL.Variant({
    'OpReturn' : IDL.Vec(IDL.Nat8),
    'p2wsh_v0' : IDL.Vec(IDL.Nat8),
    'p2tr_v1' : IDL.Vec(IDL.Nat8),
    'p2sh' : IDL.Vec(IDL.Nat8),
    'p2wpkh_v0' : IDL.Vec(IDL.Nat8),
    'p2pkh' : IDL.Vec(IDL.Nat8),
  });
  const ReleaseTokenRequest = IDL.Record({
    'received_at' : IDL.Nat64,
    'ticket_id' : IDL.Text,
    'address' : BitcoinAddress,
    'amount' : IDL.Nat,
    'rune_id' : RuneId,
  });
  const GenTicketRequest = IDL.Record({
    'received_at' : IDL.Nat64,
    'token_id' : IDL.Text,
    'txid' : IDL.Vec(IDL.Nat8),
    'target_chain_id' : IDL.Text,
    'address' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
    'rune_id' : RuneId,
  });
  const IcpChainKeyToken = IDL.Variant({ 'CKBTC' : IDL.Null });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
    'RedeemIcpChainKeyAssets' : IcpChainKeyToken,
    'Transfer' : IDL.Null,
  });
  const RuneTxRequest = IDL.Record({
    'received_at' : IDL.Nat64,
    'action' : TxAction,
    'ticket_id' : IDL.Text,
    'address' : BitcoinAddress,
    'amount' : IDL.Nat,
    'rune_id' : RuneId,
  });
  const ToggleAction = IDL.Variant({
    'Deactivate' : IDL.Null,
    'Activate' : IDL.Null,
  });
  const ToggleState = IDL.Record({
    'action' : ToggleAction,
    'chain_id' : IDL.Text,
  });
  const Event = IDL.Variant({
    'update_icpswap' : IDL.Record({ 'principal' : IDL.Principal }),
    'confirmed_generate_ticket_request' : GenTicketRequestV2,
    'received_utxos' : IDL.Record({
      'is_runes' : IDL.Bool,
      'destination' : Destination,
      'utxos' : IDL.Vec(Utxo),
    }),
    'added_runes_oracle' : IDL.Record({ 'principal' : IDL.Principal }),
    'removed_ticket_request' : IDL.Record({ 'txid' : IDL.Vec(IDL.Nat8) }),
    'update_ord_indexer' : IDL.Record({ 'principal' : IDL.Principal }),
    'removed_runes_oracle' : IDL.Record({ 'principal' : IDL.Principal }),
    'updated_fee' : IDL.Record({ 'fee' : Factor }),
    'sent_transaction' : IDL.Record({
      'fee' : IDL.Opt(IDL.Nat64),
      'txid' : IDL.Vec(IDL.Nat8),
      'btc_change_output' : BtcChangeOutput,
      'btc_utxos' : IDL.Vec(Utxo),
      'requests' : IDL.Vec(IDL.Text),
      'runes_change_output' : RunesChangeOutput,
      'runes_utxos' : IDL.Vec(RunesUtxo),
      'rune_id' : RuneId,
      'submitted_at' : IDL.Nat64,
    }),
    'added_token' : IDL.Record({ 'token' : Token, 'rune_id' : RuneId }),
    'finalized_ticket_request' : IDL.Record({
      'txid' : IDL.Vec(IDL.Nat8),
      'balances' : IDL.Vec(RunesBalance),
    }),
    'accepted_release_token_request' : ReleaseTokenRequest,
    'init' : InitArgs,
    'updated_runes_balance' : IDL.Record({
      'balance' : RunesBalance,
      'txid' : IDL.Vec(IDL.Nat8),
    }),
    'upgrade' : UpgradeArgs,
    'added_chain' : Chain,
    'update_next_ticket_seq' : IDL.Nat64,
    'update_next_directive_seq' : IDL.Nat64,
    'accepted_generate_ticket_request_v2' : GenTicketRequestV2,
    'accepted_generate_ticket_request_v3' : GenTicketRequestV2,
    'confirmed_transaction' : IDL.Record({ 'txid' : IDL.Vec(IDL.Nat8) }),
    'upate_fee_collector' : IDL.Record({ 'addr' : IDL.Text }),
    'replaced_transaction' : IDL.Record({
      'fee' : IDL.Nat64,
      'btc_change_output' : BtcChangeOutput,
      'old_txid' : IDL.Vec(IDL.Nat8),
      'new_txid' : IDL.Vec(IDL.Nat8),
      'runes_change_output' : RunesChangeOutput,
      'submitted_at' : IDL.Nat64,
    }),
    'accepted_generate_ticket_request' : GenTicketRequest,
    'accepted_rune_tx_request' : RuneTxRequest,
    'updated_rpc_url' : IDL.Record({ 'rpc_url' : IDL.Text }),
    'toggle_chain_state' : ToggleState,
  });
  const GetGenTicketReqsArgs = IDL.Record({
    'max_count' : IDL.Nat64,
    'start_txid' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Text,
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
  const UpdateBtcUtxosErr = IDL.Variant({
    'TemporarilyUnavailable' : IDL.Text,
  });
  const Result_3 = IDL.Variant({
    'Ok' : IDL.Vec(Utxo),
    'Err' : UpdateBtcUtxosErr,
  });
  const UpdateRunesBalanceArgs = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'balances' : IDL.Vec(RunesBalance),
  });
  const UpdateRunesBalanceError = IDL.Variant({
    'RequestNotConfirmed' : IDL.Null,
    'BalancesIsEmpty' : IDL.Null,
    'UtxoNotFound' : IDL.Null,
    'RequestNotFound' : IDL.Null,
    'AleardyProcessed' : IDL.Null,
    'MismatchWithGenTicketReq' : IDL.Null,
    'FinalizeTicketErr' : IDL.Text,
  });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : UpdateRunesBalanceError,
  });
  return IDL.Service({
    'clear_etching' : IDL.Func([], [], []),
    'estimate_etching_fee' : IDL.Func(
        [IDL.Nat64, IDL.Text, IDL.Opt(LogoParams)],
        [Result],
        [],
      ),
    'estimate_redeem_fee' : IDL.Func([EstimateFeeArgs], [RedeemFee], ['query']),
    'etching' : IDL.Func([IDL.Nat64, EtchingArgs], [Result_1], []),
    'etching_reveal' : IDL.Func([IDL.Text], [], []),
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result_2], []),
    'generate_ticket_status' : IDL.Func(
        [IDL.Text],
        [GenTicketStatus],
        ['query'],
      ),
    'get_btc_address' : IDL.Func([GetBtcAddressArgs], [IDL.Text], []),
    'get_btc_icp_price' : IDL.Func(
        [],
        [IDL.Opt(TokenPrice), IDL.Opt(TokenPrice)],
        [],
      ),
    'get_canister_status' : IDL.Func([], [CanisterStatusResponse], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_customs_info' : IDL.Func([], [CustomsInfo], ['query']),
    'get_etching' : IDL.Func([IDL.Text], [IDL.Opt(SendEtchingInfo)], ['query']),
    'get_etching_by_user' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(SendEtchingInfo)],
        ['query'],
      ),
    'get_etching_fee_utxos' : IDL.Func([], [IDL.Vec(UtxoArgs)], ['query']),
    'get_events' : IDL.Func([GetEventsArg], [IDL.Vec(Event)], ['query']),
    'get_main_btc_address' : IDL.Func([IDL.Text], [IDL.Text], []),
    'get_pending_gen_ticket_requests' : IDL.Func(
        [GetGenTicketReqsArgs],
        [IDL.Vec(GenTicketRequestV2)],
        ['query'],
      ),
    'get_platform_fee' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(IDL.Nat), IDL.Opt(IDL.Text)],
        ['query'],
      ),
    'get_runes_oracles' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'query_bitcoin_balance' : IDL.Func([IDL.Text, IDL.Nat32], [IDL.Nat64], []),
    'release_token_status' : IDL.Func(
        [IDL.Text],
        [ReleaseTokenStatus],
        ['query'],
      ),
    'remove_error_ticket' : IDL.Func([IDL.Text], [], []),
    'remove_runes_oracle' : IDL.Func([IDL.Principal], [], []),
    'set_fee_collector' : IDL.Func([IDL.Text], [], []),
    'set_icpswap' : IDL.Func([IDL.Principal], [], []),
    'set_ord_indexer' : IDL.Func([IDL.Principal], [], []),
    'set_runes_oracle' : IDL.Func([IDL.Principal], [], []),
    'transform' : IDL.Func([TransformArgs], [HttpResponse], ['query']),
    'update_btc_utxos' : IDL.Func([], [Result_3], []),
    'update_fees' : IDL.Func([IDL.Vec(UtxoArgs)], [], []),
    'update_rpc_url' : IDL.Func([IDL.Text], [], []),
    'update_runes_balance' : IDL.Func([UpdateRunesBalanceArgs], [Result_4], []),
  });
};
export const init = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'max_time_in_queue_nanos' : IDL.Opt(IDL.Nat64),
    'chain_state' : IDL.Opt(ChainState),
    'min_confirmations' : IDL.Opt(IDL.Nat32),
  });
  const BtcNetwork = IDL.Variant({
    'Mainnet' : IDL.Null,
    'Regtest' : IDL.Null,
    'Testnet' : IDL.Null,
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'ecdsa_key_name' : IDL.Text,
    'runes_oracle_principal' : IDL.Principal,
    'max_time_in_queue_nanos' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'btc_network' : BtcNetwork,
    'chain_state' : ChainState,
    'min_confirmations' : IDL.Opt(IDL.Nat32),
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [CustomArg];
};
