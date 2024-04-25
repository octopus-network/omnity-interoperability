export const idlFactory = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({ 'admin' : IDL.Opt(IDL.Principal) });
  const InitArgs = IDL.Record({ 'admin' : IDL.Principal });
  const HubArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
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
  const TokenMeta = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'issue_chain' : IDL.Text,
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
  const Chain = IDL.Record({
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
    'AddChain' : Chain,
    'ToggleChainState' : ToggleState,
  });
  const Error = IDL.Variant({
    'AlreadyExistingTicketId' : IDL.Text,
    'MalformedMessageBytes' : IDL.Null,
    'NotFoundChain' : IDL.Text,
    'DeactiveChain' : IDL.Text,
    'ChainAlreadyExisting' : IDL.Text,
    'ResubmitTicketIdMustExist' : IDL.Null,
    'ProposalError' : IDL.Text,
    'ResubmitTicketMustSame' : IDL.Null,
    'NotFoundAccountToken' : IDL.Tuple(IDL.Text, IDL.Text, IDL.Text),
    'NotSupportedProposal' : IDL.Null,
    'SighWithEcdsaError' : IDL.Text,
    'Unauthorized' : IDL.Null,
    'TicketAmountParseError' : IDL.Tuple(IDL.Text, IDL.Text),
    'NotFoundChainToken' : IDL.Tuple(IDL.Text, IDL.Text),
    'TokenAlreadyExisting' : IDL.Text,
    'ResubmitTicketSentTooOften' : IDL.Null,
    'GenerateDirectiveError' : IDL.Text,
    'EcdsaPublicKeyError' : IDL.Text,
    'NotFoundToken' : IDL.Text,
    'CustomError' : IDL.Text,
    'NotSufficientTokens' : IDL.Tuple(IDL.Text, IDL.Text),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : Chain, 'Err' : Error });
  const TokenOnChain = IDL.Record({
    'token_id' : IDL.Text,
    'chain_id' : IDL.Text,
    'amount' : IDL.Nat,
  });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Vec(TokenOnChain), 'Err' : Error });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Vec(Chain), 'Err' : Error });
  const GetEventsArg = IDL.Record({
    'start' : IDL.Nat64,
    'length' : IDL.Nat64,
  });
  const Topic = IDL.Variant({
    'UpdateFee' : IDL.Null,
    'AddToken' : IDL.Null,
    'AddChain' : IDL.Null,
    'ToggleChainState' : IDL.Null,
  });
  const TokenKey = IDL.Record({ 'token_id' : IDL.Text, 'chain_id' : IDL.Text });
  const Token = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'icon' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Directive = IDL.Variant({
    'UpdateFee' : Factor,
    'AddToken' : Token,
    'AddChain' : Chain,
    'ToggleChainState' : ToggleState,
  });
  const SeqKey = IDL.Record({ 'seq' : IDL.Nat64, 'chain_id' : IDL.Text });
  const TxAction = IDL.Variant({ 'Redeem' : IDL.Null, 'Transfer' : IDL.Null });
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
  const Subscribers = IDL.Record({ 'subs' : IDL.Vec(IDL.Text) });
  const Event = IDL.Variant({
    'toggled_chain_state' : IDL.Record({
      'chain' : Chain,
      'state' : ToggleState,
    }),
    'Unsubscribed_topic' : IDL.Record({ 'sub' : IDL.Text, 'topic' : Topic }),
    'updated_fee' : Factor,
    'added_token_position' : IDL.Record({
      'position' : TokenKey,
      'amount' : IDL.Nat,
    }),
    'added_token' : TokenMeta,
    'init' : InitArgs,
    'published_directive' : IDL.Record({
      'dire' : Directive,
      'seq_key' : SeqKey,
    }),
    'upgrade' : UpgradeArgs,
    'added_chain' : Chain,
    'updated_token_position' : IDL.Record({
      'position' : TokenKey,
      'amount' : IDL.Nat,
    }),
    'updated_chain' : Chain,
    'saved_directive' : Directive,
    'received_ticket' : IDL.Record({ 'ticket' : Ticket, 'seq_key' : SeqKey }),
    'resubmit_ticket' : IDL.Record({
      'ticket_id' : IDL.Text,
      'timestamp' : IDL.Nat64,
    }),
    'deleted_directive' : SeqKey,
    'Subscribed_topic' : IDL.Record({ 'topic' : Topic, 'subs' : Subscribers }),
  });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text, IDL.Nat)),
    'Err' : Error,
  });
  const TokenResp = IDL.Record({
    'decimals' : IDL.Nat8,
    'token_id' : IDL.Text,
    'icon' : IDL.Opt(IDL.Text),
    'rune_id' : IDL.Opt(IDL.Text),
    'symbol' : IDL.Text,
  });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Vec(TokenResp), 'Err' : Error });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : Error });
  const Result_7 = IDL.Variant({ 'Ok' : Ticket, 'Err' : Error });
  const Result_8 = IDL.Variant({ 'Ok' : IDL.Vec(Ticket), 'Err' : Error });
  const Result_9 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Nat64, Directive)),
    'Err' : Error,
  });
  const Result_10 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(Topic, Subscribers)),
    'Err' : Error,
  });
  const Result_11 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Nat64, Ticket)),
    'Err' : Error,
  });
  const Result_12 = IDL.Variant({ 'Ok' : IDL.Vec(IDL.Text), 'Err' : Error });
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
    'get_events' : IDL.Func([GetEventsArg], [IDL.Vec(Event)], ['query']),
    'get_fees' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Opt(IDL.Text), IDL.Nat64, IDL.Nat64],
        [Result_4],
        ['query'],
      ),
    'get_logs' : IDL.Func(
        [IDL.Opt(IDL.Nat64), IDL.Nat64, IDL.Nat64],
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
    'get_txs_with_account' : IDL.Func(
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
    'get_txs_with_chain' : IDL.Func(
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
    'query_subscribers' : IDL.Func([IDL.Opt(Topic)], [Result_10], ['query']),
    'query_tickets' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Nat64, IDL.Nat64],
        [Result_11],
        ['query'],
      ),
    'resubmit_ticket' : IDL.Func([Ticket], [Result], []),
    'send_ticket' : IDL.Func([Ticket], [Result], []),
    'set_logger_filter' : IDL.Func([IDL.Text], [], []),
    'sub_directives' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Vec(Topic)],
        [Result],
        [],
      ),
    'unsub_directives' : IDL.Func(
        [IDL.Opt(IDL.Text), IDL.Vec(Topic)],
        [Result],
        [],
      ),
    'update_fee' : IDL.Func([IDL.Vec(Factor)], [Result], []),
    'validate_proposal' : IDL.Func([IDL.Vec(Proposal)], [Result_12], ['query']),
  });
};
export const init = ({ IDL }) => {
  const UpgradeArgs = IDL.Record({ 'admin' : IDL.Opt(IDL.Principal) });
  const InitArgs = IDL.Record({ 'admin' : IDL.Principal });
  const HubArg = IDL.Variant({
    'Upgrade' : IDL.Opt(UpgradeArgs),
    'Init' : InitArgs,
  });
  return [HubArg];
};
