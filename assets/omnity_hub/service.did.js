export const idlFactory = ({ IDL }) => {
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
  const TokenMeta = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'icon' : IDL.Opt(IDL.Text),
    'settlement_chain' : IDL.Text,
    'symbol' : IDL.Text,
    'dst_chains' : IDL.Vec(IDL.Text),
  });
  const ChainState = IDL.Variant({
    'Active' : IDL.Null,
    'Deactive' : IDL.Null,
  });
  const ChainType = IDL.Variant({
    'SettlementChain' : IDL.Null,
    'ExecutionChain' : IDL.Null,
  });
  const ChainMeta = IDL.Record({
    'fee_token' : IDL.Opt(IDL.Text),
    'canister_id' : IDL.Text,
    'chain_id' : IDL.Text,
    'counterparties' : IDL.Opt(IDL.Vec(IDL.Text)),
    'chain_state' : ChainState,
    'chain_type' : ChainType,
    'contract_address' : IDL.Opt(IDL.Text),
  });
  const ToggleAction = IDL.Variant({
    'Deactivate' : IDL.Null,
    'Activate' : IDL.Null,
  });
  const ToggleState = IDL.Record({
    'action' : ToggleAction,
    'chain_id' : IDL.Text,
  });
  const Proposal = IDL.Variant({
    'UpdateFee' : Factor,
    'AddToken' : TokenMeta,
    'AddChain' : ChainMeta,
    'ToggleChainState' : ToggleState,
  });
  const Error = IDL.Variant({
    'AlreadyExistingTicketId' : IDL.Text,
    'MalformedMessageBytes' : IDL.Null,
    'NotFoundChain' : IDL.Text,
    'DeactiveChain' : IDL.Text,
    'ChainAlreadyExisting' : IDL.Text,
    'ProposalError' : IDL.Text,
    'NotFoundAccountToken' : IDL.Tuple(IDL.Text, IDL.Text, IDL.Text),
    'NotSupportedProposal' : IDL.Null,
    'SighWithEcdsaError' : IDL.Text,
    'Unauthorized' : IDL.Null,
    'TicketAmountParseError' : IDL.Tuple(IDL.Text, IDL.Text),
    'NotFoundChainToken' : IDL.Tuple(IDL.Text, IDL.Text),
    'TokenAlreadyExisting' : IDL.Text,
    'GenerateDirectiveError' : IDL.Text,
    'EcdsaPublicKeyError' : IDL.Text,
    'NotFoundToken' : IDL.Text,
    'CustomError' : IDL.Text,
    'NotSufficientTokens' : IDL.Tuple(IDL.Text, IDL.Text),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : Error });
  const Chain = IDL.Record({
    'fee_token' : IDL.Opt(IDL.Text),
    'chain_id' : IDL.Text,
    'chain_state' : ChainState,
    'chain_type' : ChainType,
    'contract_address' : IDL.Opt(IDL.Text),
  });
  const Result_1 = IDL.Variant({ 'Ok' : Chain, 'Err' : Error });
  const TokenOnChain = IDL.Record({
    'token_id' : IDL.Text,
    'chain_id' : IDL.Text,
    'amount' : IDL.Nat,
  });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Vec(TokenOnChain), 'Err' : Error });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Vec(Chain), 'Err' : Error });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text, IDL.Nat)),
    'Err' : Error,
  });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'icon' : IDL.Opt(IDL.Text),
    'issue_chain' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Vec(Token), 'Err' : Error });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : Error });
  const TxAction = IDL.Variant({ 'Redeem' : IDL.Null, 'Transfer' : IDL.Null });
  const Ticket = IDL.Record({
    'token' : IDL.Text,
    'action' : TxAction,
    'dst_chain' : IDL.Text,
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'ticket_id' : IDL.Text,
    'sender' : IDL.Opt(IDL.Text),
    'ticket_time' : IDL.Nat64,
    'src_chain' : IDL.Text,
    'amount' : IDL.Text,
    'receiver' : IDL.Text,
  });
  const Result_7 = IDL.Variant({ 'Ok' : Ticket, 'Err' : Error });
  const Result_8 = IDL.Variant({ 'Ok' : IDL.Vec(Ticket), 'Err' : Error });
  const Topic = IDL.Variant({
    'UpdateFee' : IDL.Opt(IDL.Text),
    'ActivateChain' : IDL.Null,
    'AddToken' : IDL.Opt(IDL.Text),
    'DeactivateChain' : IDL.Null,
    'AddChain' : IDL.Opt(ChainType),
  });
  const Directive = IDL.Variant({
    'UpdateFee' : Factor,
    'AddToken' : Token,
    'AddChain' : Chain,
    'ToggleChainState' : ToggleState,
  });
  const Result_9 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Nat64, Directive)),
    'Err' : Error,
  });
  const Result_10 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket)),
    'Err' : Error,
  });
  const Log = IDL.Record({ 'log' : IDL.Text, 'offset' : IDL.Nat64 });
  const Logs = IDL.Record({
    'logs' : IDL.Vec(Log),
    'all_logs_count' : IDL.Nat64,
  });
  const Result_11 = IDL.Variant({ 'Ok' : IDL.Vec(IDL.Text), 'Err' : Error });
  return IDL.Service({
    'execute_proposal' : IDL.Func([IDL.Vec(Proposal)], [Result], []),
    'get_chain' : IDL.Func([IDL.Text], [Result_1], ['query']),
    'get_chain_tokens' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Opt(IDL.Text), IDL.Nat64, IDL.Nat64],
        [Result_2],
        ['query'],
      ),
    'get_chains' : IDL.Func(
        [IDL.Opt(ChainType), IDL.Opt(ChainState), IDL.Nat64, IDL.Nat64],
        [Result_3],
        ['query'],
      ),
    'get_fees' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Opt(IDL.Text), IDL.Nat64, IDL.Nat64],
        [Result_4],
        ['query'],
      ),
    'get_logs' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Text)],
        ['query'],
      ),
    'get_tokens' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Opt(IDL.Text), IDL.Nat64, IDL.Nat64],
        [Result_5],
        ['query'],
      ),
    'get_total_tx' : IDL.Func([], [Result_6], ['query']),
    'get_tx' : IDL.Func([IDL.Text], [Result_7], ['query']),
    'get_txs' : IDL.Func(
        [
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Tuple(IDL.Nat64, IDL.Nat64)),
          IDL.Nat64,
          IDL.Nat64,
        ],
        [Result_8],
        ['query'],
      ),
    'query_directives' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Opt(Topic), IDL.Nat64, IDL.Nat64],
        [Result_9],
        ['query'],
      ),
    'query_tickets' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Nat64, IDL.Nat64],
        [Result_10],
        ['query'],
      ),
    'send_ticket' : IDL.Func([Ticket], [Result], []),
    'set_logger_filter' : IDL.Func([IDL.Text], [], []),
    'take_memory_records' : IDL.Func([IDL.Nat64, IDL.Nat64], [Logs], ['query']),
    'update_fee' : IDL.Func([IDL.Vec(Factor)], [Result], []),
    'validate_proposal' : IDL.Func([IDL.Vec(Proposal)], [Result_11], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
