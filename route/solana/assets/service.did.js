export const idlFactory = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'admin' : IDL.Opt(IDL.Principal),
    'hub_principal' : IDL.Opt(IDL.Principal),
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  const TokenInfo = IDL.Record({
    'uri' : IDL.Text,
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const SnorKeyType = IDL.Variant({
    'Native' : IDL.Null,
    'ChainKey' : IDL.Null,
  });
  const AtaKey = IDL.Record({ 'owner' : IDL.Text, 'token_mint' : IDL.Text });
  const TxError = IDL.Record({
    'signature' : IDL.Text,
    'block_hash' : IDL.Text,
    'error' : IDL.Text,
  });
  const TxStatus = IDL.Variant({
    'New' : IDL.Null,
    'Finalized' : IDL.Null,
    'TxFailed' : IDL.Record({ 'e' : TxError }),
    'Pending' : IDL.Null,
  });
  const AccountInfo = IDL.Record({
    'status' : TxStatus,
    'signature' : IDL.Opt(IDL.Text),
    'retry_4_building' : IDL.Nat64,
    'account' : IDL.Text,
    'retry_4_status' : IDL.Nat64,
  });
  const MintTokenRequest = IDL.Record({
    'status' : TxStatus,
    'signature' : IDL.Opt(IDL.Text),
    'associated_account' : IDL.Text,
    'retry_4_building' : IDL.Nat64,
    'ticket_id' : IDL.Text,
    'retry_4_status' : IDL.Nat64,
    'amount' : IDL.Nat64,
    'token_mint' : IDL.Text,
  });
  const TxAction = IDL.Variant({
    'Burn' : IDL.Null,
    'Redeem' : IDL.Null,
    'Mint' : IDL.Null,
    'Transfer' : IDL.Null,
  });
  const GenerateTicketReq = IDL.Record({
    'signature' : IDL.Text,
    'action' : TxAction,
    'token_id' : IDL.Text,
    'memo' : IDL.Opt(IDL.Text),
    'sender' : IDL.Text,
    'target_chain_id' : IDL.Text,
    'amount' : IDL.Nat64,
    'receiver' : IDL.Text,
  });
  const GenerateTicketOk = IDL.Record({ 'ticket_id' : IDL.Text });
  const GenerateTicketError = IDL.Variant({
    'InsufficientRedeemFee' : IDL.Record({
      'provided' : IDL.Nat64,
      'required' : IDL.Nat64,
    }),
    'SendTicketErr' : IDL.Text,
    'TemporarilyUnavailable' : IDL.Text,
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat64 }),
    'TransferFailure' : IDL.Text,
    'UnsupportedAction' : IDL.Text,
    'RedeemFeeNotSet' : IDL.Null,
    'UnsupportedChainId' : IDL.Text,
    'UnsupportedToken' : IDL.Text,
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat64 }),
  });
  const Result = IDL.Variant({
    'Ok' : GenerateTicketOk,
    'Err' : GenerateTicketError,
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
  const TicketType = IDL.Variant({
    'Resubmit' : IDL.Null,
    'Normal' : IDL.Null,
  });
  const Ticket = IDL.Record({
    'token' : IDL.Text,
    'action' : TxAction,
    'dst_chain' : IDL.Text,
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'ticket_id' : IDL.Text,
    'sender' : IDL.Opt(IDL.Text),
    'ticket_time' : IDL.Nat64,
    'ticket_type' : TicketType,
    'src_chain' : IDL.Text,
    'amount' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const Reason = IDL.Variant({
    'QueueIsFull' : IDL.Null,
    'CanisterError' : IDL.Text,
    'OutOfCycles' : IDL.Null,
    'Rejected' : IDL.Text,
    'TxError' : TxError,
  });
  const CallError = IDL.Record({ 'method' : IDL.Text, 'reason' : Reason });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : CallError });
  const Result_2 = IDL.Variant({ 'Ok' : MintTokenRequest, 'Err' : CallError });
  const Result_3 = IDL.Variant({ 'Ok' : TxStatus, 'Err' : CallError });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Opt(IDL.Text), 'Err' : CallError });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : CallError });
  const Result_6 = IDL.Variant({ 'Ok' : AccountInfo, 'Err' : CallError });
  return IDL.Service({
    'create_token_with_metaplex_delay' : IDL.Func(
        [TokenInfo, SnorKeyType, IDL.Nat64],
        [],
        [],
      ),
    'failed_ata' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(AtaKey, AccountInfo))],
        ['query'],
      ),
    'failed_mint_accounts' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, AccountInfo))],
        ['query'],
      ),
    'failed_mint_reqs' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, MintTokenRequest))],
        ['query'],
      ),
    'generate_ticket' : IDL.Func([GenerateTicketReq], [Result], []),
    'get_chain_list' : IDL.Func([], [IDL.Vec(Chain)], ['query']),
    'get_fee_account' : IDL.Func([], [IDL.Text], ['query']),
    'get_redeem_fee' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Nat)], ['query']),
    'get_tickets_from_queue' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket))],
        ['query'],
      ),
    'get_token_list' : IDL.Func([], [IDL.Vec(TokenResp)], ['query']),
    'mint_to' : IDL.Func([IDL.Text, IDL.Text, IDL.Nat64], [Result_1], []),
    'mint_token_req' : IDL.Func([IDL.Text], [Result_2], ['query']),
    'mint_token_status' : IDL.Func([IDL.Text], [Result_3], ['query']),
    'mint_token_tx_hash' : IDL.Func([IDL.Text], [Result_4], ['query']),
    'mint_token_with_req' : IDL.Func([MintTokenRequest], [Result_3], []),
    'query_aossicated_account' : IDL.Func(
        [IDL.Text, IDL.Text],
        [IDL.Opt(AccountInfo)],
        ['query'],
      ),
    'query_mint_account' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(AccountInfo)],
        ['query'],
      ),
    'query_mint_address' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
    'rebuild_aossicated_account' : IDL.Func(
        [IDL.Text, IDL.Text],
        [Result_1],
        [],
      ),
    'retry_mint_token' : IDL.Func([IDL.Text], [Result_1], []),
    'search_signature_from_address' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Opt(IDL.Nat64)],
        [Result_5],
        [],
      ),
    'update_associated_account' : IDL.Func(
        [IDL.Text, IDL.Text, AccountInfo],
        [Result_6],
        [],
      ),
    'update_ata_status' : IDL.Func([IDL.Text, AtaKey], [Result_6], []),
    'update_mint_account' : IDL.Func(
        [IDL.Text, AccountInfo],
        [IDL.Opt(AccountInfo)],
        [],
      ),
    'update_mint_account_status' : IDL.Func(
        [IDL.Text, IDL.Text],
        [Result_6],
        [],
      ),
    'update_mint_token_req' : IDL.Func([MintTokenRequest], [Result_2], []),
    'update_token_metaplex' : IDL.Func([TokenInfo], [Result_1], []),
  });
};
export const init = ({ IDL }) => {
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const UpgradeArgs = IDL.Record({
    'admin' : IDL.Opt(IDL.Principal),
    'hub_principal' : IDL.Opt(IDL.Principal),
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Opt(IDL.Principal),
    'chain_id' : IDL.Opt(IDL.Text),
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : IDL.Opt(ChainState),
  });
  const InitArgs = IDL.Record({
    'admin' : IDL.Principal,
    'hub_principal' : IDL.Principal,
    'fee_account' : IDL.Opt(IDL.Text),
    'sol_canister' : IDL.Principal,
    'chain_id' : IDL.Text,
    'schnorr_key_name' : IDL.Opt(IDL.Text),
    'chain_state' : ChainState,
  });
  const RouteArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [RouteArg];
};
