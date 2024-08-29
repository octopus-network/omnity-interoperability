import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AddDestChainArgs { 'dest_chain' : string, 'token_id' : string }
export interface AddRunesTokenReq {
  'dest_chain' : string,
  'icon' : string,
  'rune_id' : string,
  'symbol' : string,
}
export interface Chain {
  'fee_token' : [] | [string],
  'canister_id' : string,
  'chain_id' : string,
  'counterparties' : [] | [Array<string>],
  'chain_state' : ChainState,
  'chain_type' : ChainType,
  'contract_address' : [] | [string],
}
export interface ChainMeta {
  'fee_token' : [] | [string],
  'canister_id' : string,
  'chain_id' : string,
  'counterparties' : [] | [Array<string>],
  'chain_state' : ChainState,
  'chain_type' : ChainType,
  'contract_address' : [] | [string],
}
export type ChainState = { 'Active' : null } |
  { 'Deactive' : null };
export type ChainType = { 'SettlementChain' : null } |
  { 'ExecutionChain' : null };
export type Directive = { 'UpdateChain' : Chain } |
  { 'UpdateFee' : Factor } |
  { 'AddToken' : Token } |
  { 'AddChain' : Chain } |
  { 'ToggleChainState' : ToggleState } |
  { 'UpdateToken' : Token };
export type Error = { 'AlreadyExistingTicketId' : string } |
  { 'MalformedMessageBytes' : null } |
  { 'NotFoundChain' : string } |
  { 'DeactiveChain' : string } |
  { 'ChainAlreadyExisting' : string } |
  { 'ResubmitTicketIdMustExist' : null } |
  { 'ProposalError' : string } |
  { 'ResubmitTicketMustSame' : null } |
  { 'NotFoundAccountToken' : [string, string, string] } |
  { 'NotFoundTicketId' : string } |
  { 'NotSupportedProposal' : null } |
  { 'SighWithEcdsaError' : string } |
  { 'Unauthorized' : null } |
  { 'TicketAmountParseError' : [string, string] } |
  { 'NotFoundChainToken' : [string, string] } |
  { 'TokenAlreadyExisting' : string } |
  { 'ResubmitTicketSentTooOften' : null } |
  { 'GenerateDirectiveError' : string } |
  { 'EcdsaPublicKeyError' : string } |
  { 'RepeatSubscription' : string } |
  { 'NotFoundToken' : string } |
  { 'CustomError' : string } |
  { 'NotSufficientTokens' : [string, string] };
export type Event = {
    'updated_tx_hash' : { 'ticket_id' : string, 'tx_hash' : string }
  } |
  { 'toggled_chain_state' : { 'chain' : Chain, 'state' : ToggleState } } |
  { 'Unsubscribed_topic' : { 'sub' : string, 'topic' : Topic } } |
  { 'updated_fee' : Factor } |
  { 'added_token_position' : { 'position' : TokenKey, 'amount' : bigint } } |
  { 'added_token' : TokenMeta } |
  { 'init' : InitArgs } |
  { 'pending_ticket' : { 'ticket' : Ticket } } |
  { 'published_directive' : { 'dire' : Directive, 'seq_key' : SeqKey } } |
  { 'upgrade' : UpgradeArgs } |
  { 'added_chain' : Chain } |
  { 'updated_token_position' : { 'position' : TokenKey, 'amount' : bigint } } |
  { 'updated_chain' : Chain } |
  { 'saved_directive' : Directive } |
  { 'received_ticket' : { 'ticket' : Ticket, 'seq_key' : SeqKey } } |
  { 'resubmit_ticket' : { 'ticket_id' : string, 'timestamp' : bigint } } |
  { 'deleted_directive' : SeqKey } |
  { 'finaize_ticket' : { 'ticket_id' : string } } |
  { 'Subscribed_topic' : { 'topic' : Topic, 'subs' : Subscribers } };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeeTokenFactor {
  'fee_token' : string,
  'fee_token_factor' : bigint,
}
export interface FinalizeAddRunesArgs {
  'name' : string,
  'rune_id' : string,
  'decimal' : number,
}
export interface GetEventsArg { 'start' : bigint, 'length' : bigint }
export type HubArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
export interface InitArgs { 'admin' : Principal }
export interface LinkChainReq { 'chain1' : string, 'chain2' : string }
export type Permission = { 'Update' : null } |
  { 'Query' : null };
export type Proposal = { 'UpdateChain' : Chain } |
  { 'UpdateFee' : Factor } |
  { 'AddToken' : TokenMeta } |
  { 'AddChain' : Chain } |
  { 'ToggleChainState' : ToggleState } |
  { 'UpdateToken' : TokenMeta };
export type Result = { 'Ok' : null } |
  { 'Err' : SelfServiceError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : Error };
export type Result_10 = { 'Ok' : Array<TokenMeta> } |
  { 'Err' : Error };
export type Result_11 = { 'Ok' : Array<TokenResp> } |
  { 'Err' : Error };
export type Result_12 = { 'Ok' : Ticket } |
  { 'Err' : Error };
export type Result_13 = { 'Ok' : Array<[string, string]> } |
  { 'Err' : Error };
export type Result_14 = { 'Ok' : Array<Ticket> } |
  { 'Err' : Error };
export type Result_15 = { 'Ok' : Array<[bigint, Directive]> } |
  { 'Err' : Error };
export type Result_16 = { 'Ok' : Array<[Topic, Subscribers]> } |
  { 'Err' : Error };
export type Result_17 = { 'Ok' : Array<[bigint, Ticket]> } |
  { 'Err' : Error };
export type Result_18 = { 'Ok' : string } |
  { 'Err' : Error };
export type Result_19 = { 'Ok' : Array<string> } |
  { 'Err' : Error };
export type Result_2 = { 'Ok' : Chain } |
  { 'Err' : Error };
export type Result_3 = { 'Ok' : Array<Chain> } |
  { 'Err' : Error };
export type Result_4 = { 'Ok' : bigint } |
  { 'Err' : Error };
export type Result_5 = { 'Ok' : Array<TokenOnChain> } |
  { 'Err' : Error };
export type Result_6 = { 'Ok' : Array<Chain> } |
  { 'Err' : Error };
export type Result_7 = { 'Ok' : Array<Directive> } |
  { 'Err' : Error };
export type Result_8 = { 'Ok' : Array<[string, string, bigint]> } |
  { 'Err' : Error };
export type Result_9 = { 'Ok' : Array<[string, Ticket]> } |
  { 'Err' : Error };
export type SelfServiceError = { 'TemporarilyUnavailable' : string } |
  { 'InsufficientFee' : { 'provided' : bigint, 'required' : bigint } } |
  { 'TokenNotFound' : null } |
  { 'TransferFailure' : string } |
  { 'InvalidProposal' : string } |
  { 'InvalidRuneId' : string } |
  { 'RequestNotFound' : null } |
  { 'ChainNotFound' : string } |
  { 'TokenAlreadyExisting' : null } |
  { 'LinkError' : Error } |
  { 'EmptyArgument' : null };
export interface SelfServiceFee {
  'add_token_fee' : bigint,
  'add_chain_fee' : bigint,
}
export interface SeqKey { 'seq' : bigint, 'chain_id' : string }
export interface Subscribers { 'subs' : Array<string> }
export interface TargetChainFactor {
  'target_chain_id' : string,
  'target_chain_factor' : bigint,
}
export interface Ticket {
  'token' : string,
  'action' : TxAction,
  'dst_chain' : string,
  'memo' : [] | [Uint8Array | number[]],
  'ticket_id' : string,
  'sender' : [] | [string],
  'ticket_time' : bigint,
  'ticket_type' : TicketType,
  'src_chain' : string,
  'amount' : string,
  'receiver' : string,
}
export type TicketType = { 'Resubmit' : null } |
  { 'Normal' : null };
export type ToggleAction = { 'Deactivate' : null } |
  { 'Activate' : null };
export interface ToggleState { 'action' : ToggleAction, 'chain_id' : string }
export interface Token {
  'decimals' : number,
  'token_id' : string,
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'name' : string,
  'symbol' : string,
}
export interface TokenKey { 'token_id' : string, 'chain_id' : string }
export interface TokenMeta {
  'decimals' : number,
  'token_id' : string,
  'metadata' : Array<[string, string]>,
  'icon' : [] | [string],
  'name' : string,
  'issue_chain' : string,
  'symbol' : string,
  'dst_chains' : Array<string>,
}
export interface TokenOnChain {
  'token_id' : string,
  'chain_id' : string,
  'amount' : bigint,
}
export interface TokenResp {
  'decimals' : number,
  'token_id' : string,
  'icon' : [] | [string],
  'name' : string,
  'rune_id' : [] | [string],
  'symbol' : string,
}
export type Topic = { 'UpdateChain' : null } |
  { 'UpdateFee' : null } |
  { 'AddToken' : null } |
  { 'AddChain' : null } |
  { 'ToggleChainState' : null } |
  { 'UpdateToken' : null };
export type TxAction = { 'Burn' : null } |
  { 'Redeem' : null } |
  { 'Mint' : null } |
  { 'Transfer' : null };
export interface UpgradeArgs { 'admin' : [] | [Principal] }
export interface _SERVICE {
  'add_dest_chain_for_token' : ActorMethod<[AddDestChainArgs], Result>,
  'add_runes_token' : ActorMethod<[AddRunesTokenReq], Result>,
  'batch_update_tx_hash' : ActorMethod<[Array<string>, string], Result_1>,
  'execute_proposal' : ActorMethod<[Array<Proposal>], Result_1>,
  'finalize_add_runes_token_req' : ActorMethod<[FinalizeAddRunesArgs], Result>,
  'finalize_ticket' : ActorMethod<[string], Result_1>,
  'get_add_runes_token_requests' : ActorMethod<[], Array<AddRunesTokenReq>>,
  'get_chain' : ActorMethod<[string], Result_2>,
  'get_chain_metas' : ActorMethod<[bigint, bigint], Result_3>,
  'get_chain_size' : ActorMethod<[], Result_4>,
  'get_chain_tokens' : ActorMethod<
    [[] | [string], [] | [string], bigint, bigint],
    Result_5
  >,
  'get_chains' : ActorMethod<
    [[] | [ChainType], [] | [ChainState], bigint, bigint],
    Result_6
  >,
  'get_directive_size' : ActorMethod<[], Result_4>,
  'get_directives' : ActorMethod<[bigint, bigint], Result_7>,
  'get_events' : ActorMethod<[GetEventsArg], Array<Event>>,
  'get_fee_account' : ActorMethod<[[] | [Principal]], Uint8Array | number[]>,
  'get_fees' : ActorMethod<
    [[] | [string], [] | [string], bigint, bigint],
    Result_8
  >,
  'get_logs' : ActorMethod<[[] | [bigint], bigint, bigint], Array<string>>,
  'get_pending_ticket_size' : ActorMethod<[], Result_4>,
  'get_pending_tickets' : ActorMethod<[bigint, bigint], Result_9>,
  'get_self_service_fee' : ActorMethod<[], SelfServiceFee>,
  'get_token_metas' : ActorMethod<[bigint, bigint], Result_10>,
  'get_token_position_size' : ActorMethod<[], Result_4>,
  'get_token_size' : ActorMethod<[], Result_4>,
  'get_tokens' : ActorMethod<
    [[] | [string], [] | [string], bigint, bigint],
    Result_11
  >,
  'get_total_tx' : ActorMethod<[], Result_4>,
  'get_tx' : ActorMethod<[string], Result_12>,
  'get_tx_hash_size' : ActorMethod<[], Result_4>,
  'get_tx_hashes' : ActorMethod<[bigint, bigint], Result_13>,
  'get_txs' : ActorMethod<[bigint, bigint], Result_14>,
  'get_txs_with_account' : ActorMethod<
    [
      [] | [string],
      [] | [string],
      [] | [string],
      [] | [[bigint, bigint]],
      bigint,
      bigint,
    ],
    Result_14
  >,
  'get_txs_with_chain' : ActorMethod<
    [
      [] | [string],
      [] | [string],
      [] | [string],
      [] | [[bigint, bigint]],
      bigint,
      bigint,
    ],
    Result_14
  >,
  'handle_chain' : ActorMethod<[Array<Proposal>], Result_1>,
  'handle_token' : ActorMethod<[Array<Proposal>], Result_1>,
  'link_chains' : ActorMethod<[LinkChainReq], Result>,
  'pending_ticket' : ActorMethod<[Ticket], Result_1>,
  'query_directives' : ActorMethod<
    [[] | [string], [] | [Topic], bigint, bigint],
    Result_15
  >,
  'query_subscribers' : ActorMethod<[[] | [Topic]], Result_16>,
  'query_tickets' : ActorMethod<[[] | [string], bigint, bigint], Result_17>,
  'query_tx_hash' : ActorMethod<[string], Result_18>,
  'remove_runes_oracle' : ActorMethod<[Principal], undefined>,
  'resubmit_ticket' : ActorMethod<[Ticket], Result_1>,
  'send_ticket' : ActorMethod<[Ticket], Result_1>,
  'set_logger_filter' : ActorMethod<[string], undefined>,
  'set_permissions' : ActorMethod<[Principal, Permission], undefined>,
  'set_runes_oracle' : ActorMethod<[Principal], undefined>,
  'sub_directives' : ActorMethod<[[] | [string], Array<Topic>], Result_1>,
  'sync_ticket_size' : ActorMethod<[], Result_4>,
  'sync_tickets' : ActorMethod<[bigint, bigint], Result_17>,
  'unsub_directives' : ActorMethod<[[] | [string], Array<Topic>], Result_1>,
  'update_fee' : ActorMethod<[Array<Factor>], Result_1>,
  'update_tx_hash' : ActorMethod<[string, string], Result_1>,
  'validate_proposal' : ActorMethod<[Array<Proposal>], Result_19>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
