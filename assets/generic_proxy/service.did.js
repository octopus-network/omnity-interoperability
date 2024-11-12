export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'token_id' : IDL.Text,
    'ckbtc_minter_principal' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
  });
  const OmnityAccount = IDL.Record({
    'chain_id' : IDL.Text,
    'account' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const Errors = IDL.Variant({
    'AccountIdParseError' : IDL.Tuple(IDL.Text, IDL.Text),
    'CkBtcUpdateBalanceError' : IDL.Tuple(IDL.Text, IDL.Text),
    'CallError' : IDL.Tuple(IDL.Text, IDL.Principal, IDL.Text, IDL.Text),
    'CanisterCallError' : IDL.Tuple(IDL.Text, IDL.Text, IDL.Text, IDL.Text),
    'NatConversionError' : IDL.Text,
    'CustomError' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : Errors });
  const UpdateBalanceJob = IDL.Record({
    'failed_times' : IDL.Nat32,
    'omnity_account' : OmnityAccount,
    'next_execute_time' : IDL.Nat64,
  });
  const State = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'update_balances_jobs' : IDL.Vec(UpdateBalanceJob),
    'token_id' : IDL.Text,
    'is_timer_running' : IDL.Vec(IDL.Text),
    'ckbtc_minter_principal' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
  });
  const Result_2 = IDL.Variant({ 'Ok' : State, 'Err' : Errors });
  const OutPoint = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'vout' : IDL.Nat32,
  });
  const Utxo = IDL.Record({
    'height' : IDL.Nat32,
    'value' : IDL.Nat64,
    'outpoint' : OutPoint,
  });
  const MintedUtxo = IDL.Record({
    'minted_amount' : IDL.Nat64,
    'block_index' : IDL.Nat64,
    'utxo' : Utxo,
  });
  const TicketRecord = IDL.Record({
    'ticket_id' : IDL.Text,
    'minted_utxos' : IDL.Vec(MintedUtxo),
  });
  const UtxoRecord = IDL.Record({
    'ticket_id' : IDL.Opt(IDL.Text),
    'minted_utxo' : MintedUtxo,
  });
  return IDL.Service({
    'generate_ticket_from_subaccount' : IDL.Func([OmnityAccount], [Result], []),
    'query_btc_mint_address_by_omnity_account' : IDL.Func(
        [OmnityAccount],
        [Result_1],
        [],
      ),
    'query_state' : IDL.Func([], [Result_2], ['query']),
    'query_ticket_records' : IDL.Func(
        [OmnityAccount],
        [IDL.Vec(TicketRecord)],
        ['query'],
      ),
    'query_utxo_records' : IDL.Func(
        [OmnityAccount],
        [IDL.Vec(UtxoRecord)],
        ['query'],
      ),
    'trigger_update_balance' : IDL.Func([OmnityAccount], [Result], []),
    'update_balance_after_finalization' : IDL.Func([OmnityAccount], [], []),
    'update_settings' : IDL.Func(
        [
          IDL.Opt(IDL.Principal),
          IDL.Opt(IDL.Principal),
          IDL.Opt(IDL.Principal),
          IDL.Opt(IDL.Text),
        ],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'token_id' : IDL.Text,
    'ckbtc_minter_principal' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
  });
  return [InitArgs];
};
