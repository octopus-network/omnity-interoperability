export const idlFactory = ({ IDL }) => {
  const Mode = IDL.Variant({
    'ReadOnly' : IDL.Null,
    'GeneralAvailability' : IDL.Null,
    'ReleaseRestricted' : IDL.Null,
    'TransportRestricted' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'mode' : IDL.Opt(Mode),
    'runes_oracle_principal' : IDL.Opt(IDL.Principal),
    'max_time_in_queue_nanos' : IDL.Opt(IDL.Nat64),
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
    'mode' : Mode,
    'runes_oracle_principal' : IDL.Principal,
    'max_time_in_queue_nanos' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'btc_network' : BtcNetwork,
    'min_confirmations' : IDL.Opt(IDL.Nat32),
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const RuneId = IDL.Record({ 'tx' : IDL.Nat32, 'block' : IDL.Nat64 });
  const EstimateFeeArgs = IDL.Record({
    'amount' : IDL.Opt(IDL.Nat),
    'rune_id' : RuneId,
  });
  const RedeemFee = IDL.Record({ 'bitcoin_fee' : IDL.Nat64 });
  const GenerateTicketArgs = IDL.Record({
    'txid' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat,
    'receiver' : IDL.Text,
    'rune_id' : IDL.Text,
  });
  const GenerateTicketError = IDL.Variant({
    'TemporarilyUnavailable' : IDL.Text,
    'InvalidRuneId' : IDL.Text,
    'AlreadySubmitted' : IDL.Null,
    'InvalidTxId' : IDL.Null,
    'AleardyProcessed' : IDL.Null,
    'NoNewUtxos' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : GenerateTicketError });
  const GenTicketStatusArgs = IDL.Record({ 'txid' : IDL.Vec(IDL.Nat8) });
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
  const GenTicketStatus = IDL.Variant({
    'Invalid' : IDL.Null,
    'Finalized' : IDL.Null,
    'Unknown' : IDL.Null,
    'Pending' : GenTicketRequest,
  });
  const GetBtcAddressArgs = IDL.Record({
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
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
  const CustomsInfo = IDL.Record({ 'min_confirmations' : IDL.Nat32 });
  const GetEventsArg = IDL.Record({
    'start' : IDL.Nat64,
    'length' : IDL.Nat64,
  });
  const Destination = IDL.Record({
    'token' : IDL.Opt(IDL.Text),
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
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
  const ToggleAction = IDL.Variant({
    'Deactivate' : IDL.Null,
    'Activate' : IDL.Null,
  });
  const ToggleState = IDL.Record({
    'action' : ToggleAction,
    'chain_id' : IDL.Text,
  });
  const Event = IDL.Variant({
    'received_utxos' : IDL.Record({
      'is_runes' : IDL.Bool,
      'destination' : Destination,
      'utxos' : IDL.Vec(Utxo),
    }),
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
    'confirmed_transaction' : GenTicketStatusArgs,
    'replaced_transaction' : IDL.Record({
      'fee' : IDL.Nat64,
      'btc_change_output' : BtcChangeOutput,
      'old_txid' : IDL.Vec(IDL.Nat8),
      'new_txid' : IDL.Vec(IDL.Nat8),
      'runes_change_output' : RunesChangeOutput,
      'submitted_at' : IDL.Nat64,
    }),
    'accepted_generate_ticket_request' : GenTicketRequest,
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
  const ReleaseTokenStatusArgs = IDL.Record({ 'ticket_id' : IDL.Text });
  const ReleaseTokenStatus = IDL.Variant({
    'Signing' : IDL.Null,
    'Confirmed' : IDL.Vec(IDL.Nat8),
    'Sending' : IDL.Vec(IDL.Nat8),
    'Unknown' : IDL.Null,
    'Submitted' : IDL.Vec(IDL.Nat8),
    'Pending' : IDL.Null,
  });
  const UpdateBtcUtxosErr = IDL.Variant({
    'TemporarilyUnavailable' : IDL.Text,
  });
  const Result_1 = IDL.Variant({
    'Ok' : IDL.Vec(Utxo),
    'Err' : UpdateBtcUtxosErr,
  });
  const UpdateRunesBalanceArgs = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'balances' : IDL.Vec(RunesBalance),
  });
  const UpdateRunesBalanceError = IDL.Variant({
    'SendTicketErr' : IDL.Text,
    'UtxoNotFound' : IDL.Null,
    'RequestNotFound' : IDL.Null,
    'AleardyProcessed' : IDL.Null,
    'MismatchWithGenTicketReq' : IDL.Null,
  });
  const Result_2 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : UpdateRunesBalanceError,
  });
  return IDL.Service({
    'estimate_redeem_fee' : IDL.Func([EstimateFeeArgs], [RedeemFee], ['query']),
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'generate_ticket_status' : IDL.Func(
        [GenTicketStatusArgs],
        [GenTicketStatus],
        ['query'],
      ),
    'get_btc_address' : IDL.Func([GetBtcAddressArgs], [IDL.Text], []),
    'get_canister_status' : IDL.Func([], [CanisterStatusResponse], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_customs_info' : IDL.Func([], [CustomsInfo], ['query']),
    'get_events' : IDL.Func([GetEventsArg], [IDL.Vec(Event)], ['query']),
    'get_main_btc_address' : IDL.Func([IDL.Text], [IDL.Text], []),
    'get_pending_gen_ticket_requests' : IDL.Func(
        [GetGenTicketReqsArgs],
        [IDL.Vec(GenTicketRequest)],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'release_token_status' : IDL.Func(
        [ReleaseTokenStatusArgs],
        [ReleaseTokenStatus],
        ['query'],
      ),
    'update_btc_utxos' : IDL.Func([], [Result_1], []),
    'update_runes_balance' : IDL.Func([UpdateRunesBalanceArgs], [Result_2], []),
  });
};
export const init = ({ IDL }) => {
  const Mode = IDL.Variant({
    'ReadOnly' : IDL.Null,
    'GeneralAvailability' : IDL.Null,
    'ReleaseRestricted' : IDL.Null,
    'TransportRestricted' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'mode' : IDL.Opt(Mode),
    'runes_oracle_principal' : IDL.Opt(IDL.Principal),
    'max_time_in_queue_nanos' : IDL.Opt(IDL.Nat64),
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
    'mode' : Mode,
    'runes_oracle_principal' : IDL.Principal,
    'max_time_in_queue_nanos' : IDL.Nat64,
    'chain_id' : IDL.Text,
    'btc_network' : BtcNetwork,
    'min_confirmations' : IDL.Opt(IDL.Nat32),
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [CustomArg];
};
