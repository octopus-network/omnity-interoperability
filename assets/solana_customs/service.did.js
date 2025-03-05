export const idlFactory = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'sol_canister' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'rpc_list' : IDL.Vec(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'port_program_id' : IDL.Text,
    'schnorr_key_name' : IDL.Text,
    'chain_state' : ChainState,
    'forward' : IDL.Opt(IDL.Text),
    'min_response_count' : IDL.Nat32,
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const GenerateTicketArgs = IDL.Record({
    'signature' : IDL.Text,
    'token_id' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat64,
    'receiver' : IDL.Text,
  });
  const GenerateTicketError = IDL.Variant({
    'SendTicketErr' : IDL.Text,
    'RpcError' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'AlreadyProcessed' : IDL.Null,
    'DecodeTxError' : IDL.Text,
    'MismatchWithGenTicketReq' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : GenerateTicketError });
  const GenTicketStatus = IDL.Variant({
    'Finalized' : GenerateTicketArgs,
    'Unknown' : IDL.Null,
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
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const ReleaseTokenStatus = IDL.Variant({
    'Finalized' : IDL.Text,
    'Unknown' : IDL.Null,
    'Submitted' : IDL.Text,
    'Pending' : IDL.Null,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'generate_ticket_status' : IDL.Func(
        [IDL.Text],
        [GenTicketStatus],
        ['query'],
      ),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_payer_address' : IDL.Func([], [IDL.Text], []),
    'get_token_list' : IDL.Func([], [IDL.Vec(Token)], ['query']),
    'redeem_from_fee_address' : IDL.Func([IDL.Text, IDL.Nat64], [Result_1], []),
    'release_token_status' : IDL.Func(
        [IDL.Text],
        [ReleaseTokenStatus],
        ['query'],
      ),
    'resubmit_release_token_tx' : IDL.Func([IDL.Text], [Result_1], []),
  });
};
export const init = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'hub_principal' : IDL.Opt(IDL.Principal),
    'sol_canister' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'hub_principal' : IDL.Principal,
    'rpc_list' : IDL.Vec(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'port_program_id' : IDL.Text,
    'schnorr_key_name' : IDL.Text,
    'chain_state' : ChainState,
    'forward' : IDL.Opt(IDL.Text),
    'min_response_count' : IDL.Nat32,
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [CustomArg];
};
