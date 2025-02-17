use ic_stable_structures::StableBTreeMap;

use omnity_types::Directive;
use omnity_types::DireKey;
use omnity_types::SeqKey;
use omnity_types::Ticket;

use crate::memory::Memory;

/// Directive Queue
/// K: DstChain, V:  BTreeMap<Seq, Directive>
pub type DireQueue = StableBTreeMap<DireKey, Directive, Memory>;
/// Ticket Queue
/// K: DstChain, V: BTreeMap<Seq, Ticket>
pub type TicketQueue = StableBTreeMap<SeqKey, Ticket, Memory>;
