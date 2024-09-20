export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'ckbtc_minter_principal' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const Result_1 = IDL.Variant({ 'Ok' : Account, 'Err' : IDL.Text });
  const UpdateBalanceJob = IDL.Record({
    'osmosis_account_id' : IDL.Text,
    'failed_times' : IDL.Nat32,
    'next_execute_time' : IDL.Nat64,
  });
  const Settings = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'update_balances_jobs' : IDL.Vec(UpdateBalanceJob),
    'is_timer_running' : IDL.Vec(IDL.Text),
    'ckbtc_minter_principal' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
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
    'generate_ticket_from_subaccount' : IDL.Func([IDL.Text], [Result], []),
    'get_btc_mint_address' : IDL.Func([IDL.Text], [Result], []),
    'get_identity_by_osmosis_account_id' : IDL.Func(
        [IDL.Text],
        [Result_1],
        ['query'],
      ),
    'query_scheduled_osmosis_account_id_list' : IDL.Func(
        [],
        [IDL.Vec(IDL.Text)],
        ['query'],
      ),
    'query_settings' : IDL.Func([], [Settings], ['query']),
    'query_ticket_records' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(TicketRecord)],
        ['query'],
      ),
    'query_utxo_records' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(UtxoRecord)],
        ['query'],
      ),
    'trigger_update_balance' : IDL.Func([IDL.Text], [Result], []),
    'update_balance_after_finalization' : IDL.Func([IDL.Text], [], []),
    'update_settings' : IDL.Func(
        [
          IDL.Opt(IDL.Principal),
          IDL.Opt(IDL.Principal),
          IDL.Opt(IDL.Principal),
        ],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ckbtc_ledger_principal' : IDL.Principal,
    'ckbtc_minter_principal' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
  });
  return [InitArgs];
};
