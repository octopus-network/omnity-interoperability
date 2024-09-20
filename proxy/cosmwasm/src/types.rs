use std::borrow::Cow;

use crate::*;
use ic_stable_structures::{storable::Bound, Storable};
use subtle_encoding::bech32;
use utils::get_chain_time_seconds;

pub const TENDERMINT_ADDRESS_LENGTH: usize = 20;
pub type TicketId = String;

pub struct AddressData(pub [u8; 20]);

impl AddressData {
    pub const PREFIX: &'static str = "osmo";

    pub fn to_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn to_cosmos_address(&self) -> String {
        bech32::encode(AddressData::PREFIX, self.0.to_vec())
    }
}

// https://github.com/tendermint/spec/blob/master/spec/core/encoding.md
impl TryFrom<&str> for AddressData {
    type Error = Errors;

    fn try_from(account_id: &str) -> Result<Self> {
        let a = bech32::decode(account_id)
            .map_err(|err| Errors::AccountIdParseError(account_id.to_string(), err.to_string()))?;
        let bytes: [u8; 20] = a.1.try_into().map_err(|_| {
            Errors::AccountIdParseError(
                account_id.to_string(),
                "Invalid address length".to_string(),
            )
        })?;
        Ok(AddressData(bytes))
    }
}

impl From<AddressData> for Subaccount {
    fn from(data: AddressData) -> Self {
        let mut subaccount_bytes = [0_u8; 32];
        subaccount_bytes[..20].copy_from_slice(data.to_bytes());
        // Subaccount(subaccount_bytes)
        subaccount_bytes
    }
}

impl From<Subaccount> for AddressData {
    fn from(value: Subaccount) -> Self {
        let mut bytes = [0_u8; 20];
        bytes.copy_from_slice(&value[..20]);
        AddressData(bytes)
    }
}

#[derive(CandidType, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct UpdateBalanceJob {
    pub osmosis_account_id: String,
    pub failed_times: u32,
    pub next_execute_time: u64,
}

impl UpdateBalanceJob {
    const MAX_FAILED_TIMES: u32 = 15;
    const INIT_DELAY: u64 = 60 * 50;
    const FAILED_DELAY: u64 = 60 * 5;
    pub fn new(osmosis_account_id: String) -> Self {
        UpdateBalanceJob {
            osmosis_account_id,
            failed_times: 0,
            next_execute_time: get_chain_time_seconds() + UpdateBalanceJob::INIT_DELAY,
        }
    }

    pub fn executable(&self) -> bool {
        get_chain_time_seconds() >= self.next_execute_time
    }

    pub fn handle_execute_failed_and_continue(&mut self) -> bool {
        self.failed_times += 1;
        if self.failed_times >= UpdateBalanceJob::MAX_FAILED_TIMES {
            return false;
        }
        self.next_execute_time = get_chain_time_seconds() + UpdateBalanceJob::FAILED_DELAY;
        return true
    }

}

impl Storable for UpdateBalanceJob {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let update_balance_job = ciborium::de::from_reader(bytes.as_ref())
            .expect("failed to decode UpdateBalanceJob");
        update_balance_job
    }

    const BOUND: Bound = Bound::Unbounded;
}