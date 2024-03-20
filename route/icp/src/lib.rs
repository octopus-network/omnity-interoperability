mod storage;
mod ic_interfaces;
mod domain;
mod util;

use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc};
use candid::Principal;
use omnity_types::{ChainId, Ticket};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use storage::*;
use ic_interfaces::*;
use domain::*;
use omnity_route_common::error::*;

use icrc_ledger_types::icrc1::transfer::{BlockIndex, NumTokens, TransferArg, TransferError};
use icrc_ledger_types::icrc1::account::Account;
use ic_cdk::{query, update};
