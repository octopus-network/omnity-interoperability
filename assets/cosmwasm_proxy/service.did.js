export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'trigger' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
    'ckbtc_index_principal' : IDL.Principal,
  });
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const Result = IDL.Variant({ 'Ok' : Account, 'Err' : IDL.Text });
  const OutPoint = IDL.Record({
    'txid' : IDL.Vec(IDL.Nat8),
    'vout' : IDL.Nat32,
  });
  const Utxo = IDL.Record({
    'height' : IDL.Nat32,
    'value' : IDL.Nat64,
    'outpoint' : OutPoint,
  });
  const BtcTransportRecord = IDL.Record({
    'minted_amount' : IDL.Nat64,
    'block_index' : IDL.Nat64,
    'utxo' : Utxo,
    'ticket_id' : IDL.Opt(IDL.Text),
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  return IDL.Service({
    'get_identity_by_osmosis_account_id' : IDL.Func(
        [IDL.Text],
        [Result],
        ['query'],
      ),
    'query_btc_transport_info' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(BtcTransportRecord)],
        ['query'],
      ),
    'query_status' : IDL.Func([], [IDL.Principal, IDL.Principal], ['query']),
    'set_trigger_principal' : IDL.Func([IDL.Principal], [Result_1], []),
    'trigger_transaction' : IDL.Func([IDL.Nat], [Result_1], []),
    'trigger_update_balance' : IDL.Func([IDL.Text], [Result_1], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'trigger' : IDL.Principal,
    'icp_customs_principal' : IDL.Principal,
    'ckbtc_index_principal' : IDL.Principal,
  });
  return [InitArgs];
};
