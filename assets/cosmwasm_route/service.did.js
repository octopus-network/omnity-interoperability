export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'cw_rpc_url' : IDL.Text,
    'cw_rest_url' : IDL.Text,
    'chain_id' : IDL.Text,
    'cosmwasm_port_contract_address' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'cache_public_key_and_start_timer' : IDL.Func([], [], []),
    'redeem' : IDL.Func([IDL.Text], [Result], []),
    'tendermint_address' : IDL.Func([], [Result], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'cw_rpc_url' : IDL.Text,
    'cw_rest_url' : IDL.Text,
    'chain_id' : IDL.Text,
    'cosmwasm_port_contract_address' : IDL.Text,
  });
  return [InitArgs];
};
