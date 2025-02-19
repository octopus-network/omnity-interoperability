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
    'schnorr_key_name' : IDL.Text,
    'chain_state' : ChainState,
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
    'AmountIsZero' : IDL.Null,
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
  const GetSolAddressArgs = IDL.Record({
    'target_chain_id' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const ReleaseTokenStatus = IDL.Variant({
    'Failed' : IDL.Text,
    'Finalized' : IDL.Null,
    'Unknown' : IDL.Null,
    'Submitted' : IDL.Null,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const CollectionTx = IDL.Record({
    'signature' : IDL.Opt(IDL.Text),
    'from_path' : IDL.Vec(IDL.Vec(IDL.Nat8)),
    'from' : IDL.Vec(IDL.Nat8),
    'try_cnt' : IDL.Nat32,
    'last_sent_at' : IDL.Nat64,
    'source_signature' : IDL.Text,
    'amount' : IDL.Nat64,
  });
  return IDL.Service({
    'generate_ticket' : IDL.Func([GenerateTicketArgs], [Result], []),
    'generate_ticket_status' : IDL.Func(
        [IDL.Text],
        [GenTicketStatus],
        ['query'],
      ),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee_address' : IDL.Func([], [IDL.Text], []),
    'get_main_address' : IDL.Func([], [IDL.Text], []),
    'get_sol_address' : IDL.Func([GetSolAddressArgs], [IDL.Text], []),
    'get_token_list' : IDL.Func([], [IDL.Vec(Token)], ['query']),
    'release_token_status' : IDL.Func(
        [IDL.Text],
        [ReleaseTokenStatus],
        ['query'],
      ),
    'resubmit_release_token_tx' : IDL.Func([IDL.Text], [Result_1], []),
    'submitted_collection_txs' : IDL.Func(
        [],
        [IDL.Vec(CollectionTx)],
        ['query'],
      ),
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
    'schnorr_key_name' : IDL.Text,
    'chain_state' : ChainState,
    'min_response_count' : IDL.Nat32,
  });
  const CustomArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [CustomArg];
};
