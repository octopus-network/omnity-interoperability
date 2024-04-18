import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

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
export interface ChainWithSeq {
  'fee_token' : [] | [string],
  'canister_id' : string,
  'chain_id' : string,
  'latest_ticket_seq' : [] | [bigint],
  'counterparties' : [] | [Array<string>],
  'chain_state' : ChainState,
  'chain_type' : ChainType,
  'contract_address' : [] | [string],
  'latest_dire_seq' : [] | [bigint],
}
export type Directive = { 'UpdateFee' : Factor } |
  { 'AddToken' : Token } |
  { 'AddChain' : Chain } |
  { 'ToggleChainState' : ToggleState };
export type Error = { 'AlreadyExistingTicketId' : string } |
  { 'MalformedMessageBytes' : null } |
  { 'NotFoundChain' : string } |
  { 'DeactiveChain' : string } |
  { 'ChainAlreadyExisting' : string } |
  { 'ProposalError' : string } |
  { 'NotFoundAccountToken' : [string, string, string] } |
  { 'NotSupportedProposal' : null } |
  { 'SighWithEcdsaError' : string } |
  { 'Unauthorized' : null } |
  { 'TicketAmountParseError' : [string, string] } |
  { 'NotFoundChainToken' : [string, string] } |
  { 'TokenAlreadyExisting' : string } |
  { 'GenerateDirectiveError' : string } |
  { 'EcdsaPublicKeyError' : string } |
  { 'NotFoundToken' : string } |
  { 'CustomError' : string } |
  { 'NotSufficientTokens' : [string, string] };
export type Event = {
    'toggled_chain_state' : { 'chain' : ChainWithSeq, 'state' : ToggleState }
  } |
  { 'updated_fee' : Factor } |
  { 'added_token_position' : { 'position' : TokenKey, 'amount' : bigint } } |
  { 'added_token' : TokenMeta } |
  { 'post_upgrade' : Uint8Array | number[] } |
  { 'init' : Principal } |
  {
    'received_directive' : { 'dst_chain' : ChainWithSeq, 'dire' : Directive }
  } |
  { 'added_chain' : ChainWithSeq } |
  { 'updated_token_position' : { 'position' : TokenKey, 'amount' : bigint } } |
  { 'updated_chain' : ChainWithSeq } |
  { 'received_ticket' : { 'ticket' : Ticket, 'dst_chain' : ChainWithSeq } } |
  { 'pre_upgrade' : Uint8Array | number[] };
export type Factor = { 'UpdateFeeTokenFactor' : FeeTokenFactor } |
  { 'UpdateTargetChainFactor' : TargetChainFactor };
export interface FeeTokenFactor {
  'fee_token' : string,
  'fee_token_factor' : bigint,
}
export interface GetEventsArg { 'start' : bigint, 'length' : bigint }
export type Proposal = { 'UpdateFee' : Factor } |
  { 'AddToken' : TokenMeta } |
  { 'AddChain' : Chain } |
  { 'ToggleChainState' : ToggleState };
export type Result = { 'Ok' : null } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : Chain } |
  { 'Err' : Error };
export type Result_10 = { 'Ok' : Array<[bigint, Ticket]> } |
  { 'Err' : Error };
export type Result_11 = { 'Ok' : Array<string> } |
  { 'Err' : Error };
export type Result_2 = { 'Ok' : Array<TokenOnChain> } |
  { 'Err' : Error };
export type Result_3 = { 'Ok' : Array<Chain> } |
  { 'Err' : Error };
export type Result_4 = { 'Ok' : Array<[string, string, bigint]> } |
  { 'Err' : Error };
export type Result_5 = { 'Ok' : Array<Token> } |
  { 'Err' : Error };
export type Result_6 = { 'Ok' : bigint } |
  { 'Err' : Error };
export type Result_7 = { 'Ok' : Ticket } |
  { 'Err' : Error };
export type Result_8 = { 'Ok' : Array<Ticket> } |
  { 'Err' : Error };
export type Result_9 = { 'Ok' : Array<[bigint, Directive]> } |
  { 'Err' : Error };
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
export type Topic = { 'UpdateFee' : [] | [string] } |
  { 'ActivateChain' : null } |
  { 'AddToken' : [] | [string] } |
  { 'DeactivateChain' : null } |
  { 'AddChain' : [] | [ChainType] };
export type TxAction = { 'Redeem' : null } |
  { 'Transfer' : null };
export interface _SERVICE {
  'execute_proposal' : ActorMethod<[Array<Proposal>], Result>,
  'get_chain' : ActorMethod<[string], Result_1>,
  'get_chain_tokens' : ActorMethod<
    [[] | [string], [] | [string], bigint, bigint],
    Result_2
  >,
  'get_chains' : ActorMethod<
    [[] | [ChainType], [] | [ChainState], bigint, bigint],
    Result_3
  >,
  'get_events' : ActorMethod<[GetEventsArg], Array<Event>>,
  'get_fees' : ActorMethod<
    [[] | [string], [] | [string], bigint, bigint],
    Result_4
  >,
  'get_logs' : ActorMethod<[[] | [bigint], bigint, bigint], Array<string>>,
  'get_tokens' : ActorMethod<
    [[] | [string], [] | [string], bigint, bigint],
    Result_5
  >,
  'get_total_tx' : ActorMethod<[], Result_6>,
  'get_tx' : ActorMethod<[string], Result_7>,
  'get_txs' : ActorMethod<
    [
      [] | [string],
      [] | [string],
      [] | [string],
      [] | [[bigint, bigint]],
      bigint,
      bigint,
    ],
    Result_8
  >,
  'query_directives' : ActorMethod<
    [[] | [string], [] | [Topic], bigint, bigint],
    Result_9
  >,
  'query_tickets' : ActorMethod<[[] | [string], bigint, bigint], Result_10>,
  'send_ticket' : ActorMethod<[Ticket], Result>,
  'set_logger_filter' : ActorMethod<[string], undefined>,
  'update_fee' : ActorMethod<[Array<Factor>], Result>,
  'validate_proposal' : ActorMethod<[Array<Proposal>], Result_11>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
