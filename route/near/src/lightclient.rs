pub use near_client::{
    near_types::{hash::sha256, merkle::merklize},
    types::*,
    BasicNearLightClient, HeaderVerificationError, StateProofVerificationError,
};
use std::{cell::RefCell, collections::BTreeMap};
use thiserror::Error;

thread_local! {
    static STATES: RefCell<BTreeMap<Height, ConsensusState>> = RefCell::new(Default::default());
}

#[derive(Clone, Debug, Error)]
pub enum LightClientError {
    #[error("State verification error: {0:?}")]
    StateError(StateProofVerificationError),
    #[error("Header verification error: {0:?}")]
    HeaderError(HeaderVerificationError),
}

pub struct DummyLightClient;

impl BasicNearLightClient for DummyLightClient {
    fn latest_height(&self) -> Height {
        STATES.with_borrow(|s| s.keys().last().expect("Client not initialized").to_owned())
    }

    fn get_consensus_state(&self, height: &Height) -> Option<ConsensusState> {
        STATES.with_borrow(|s| s.get(height).map(|cs| cs.clone()))
    }
}

impl DummyLightClient {
    pub fn init_or_return(state: ConsensusState) {
        STATES.with_borrow_mut(|s| {
            s.entry(state.header.height()).or_insert(state);
        });
    }

    pub fn verify_proofs(
        &self,
        header: Header,
        key: &[u8],
        value: &[u8],
        proofs: Vec<Vec<u8>>,
    ) -> Result<(), LightClientError> {
        let state = self
            .try_accept_header(header)
            .map_err(|e| LightClientError::HeaderError(e))?;
        state
            .verify_membership(key, value, &proofs)
            .map_err(|e| LightClientError::StateError(e))
    }

    fn try_accept_header(&self, header: Header) -> Result<ConsensusState, HeaderVerificationError> {
        let height = header.height();
        if let Some(state) = self.get_consensus_state(&height) {
            return Ok(state);
        }
        let ancestor = self.verify_header(&header)?;
        let accepted = ConsensusState {
            current_bps: ancestor.current_bps,
            header,
        };
        STATES.with_borrow_mut(|s| {
            s.insert(height, accepted.clone());
        });
        Ok(accepted)
    }

    fn find_nearest_ancestor(&self, header: &Header) -> Option<ConsensusState> {
        let mut height = header.height();
        let min =
            STATES.with_borrow(|s| s.keys().next().expect("Client not initialized").to_owned());
        while height > min {
            if let Some(state) = self.get_consensus_state(&height) {
                return Some(state);
            }
            height -= 1;
        }
        None
    }

    fn verify_header(&self, header: &Header) -> Result<ConsensusState, HeaderVerificationError> {
        let ancestor = self.find_nearest_ancestor(header).ok_or(
            HeaderVerificationError::MissingCachedEpochBlockProducers {
                epoch_id: header.epoch_id(),
            },
        )?;
        let approval_message = header.light_client_block.approval_message();

        // Check the height of the block is higher than the height of the current head.
        if header.height() <= ancestor.header.height() {
            return Err(HeaderVerificationError::InvalidBlockHeight);
        }

        // Check the epoch of the block is equal to the epoch_id or next_epoch_id
        // known for the current head.
        if header.epoch_id() != ancestor.header.epoch_id()
            && header.epoch_id() != ancestor.header.next_epoch_id()
        {
            return Err(HeaderVerificationError::InvalidEpochId);
        }

        // If the epoch of the block is equal to the next_epoch_id of the head,
        // then next_bps is not None.
        if header.epoch_id() == ancestor.header.next_epoch_id()
            && header.light_client_block.next_bps.is_none()
        {
            return Err(HeaderVerificationError::MissingNextBlockProducersInHead);
        }

        // 1. The approvals_after_next contains valid signatures on approval_message
        // from the block producers of the corresponding epoch.
        // 2. The signatures present in approvals_after_next correspond to
        // more than 2/3 of the total stake.
        let mut total_stake = 0;
        let mut approved_stake = 0;

        let bps = ancestor.get_block_producers_of(&header.epoch_id());
        if bps.is_none() {
            return Err(HeaderVerificationError::MissingCachedEpochBlockProducers {
                epoch_id: header.epoch_id(),
            });
        }

        let epoch_block_producers = bps.expect("Should not fail based on previous checking.");
        for (maybe_signature, block_producer) in header
            .light_client_block
            .approvals_after_next
            .iter()
            .zip(epoch_block_producers.iter())
        {
            let bp_stake_view = block_producer.clone().into_validator_stake();
            let bp_stake = bp_stake_view.stake;
            total_stake += bp_stake;

            if maybe_signature.is_none() {
                continue;
            }

            approved_stake += bp_stake;

            let validator_public_key = bp_stake_view.public_key.clone();
            if !maybe_signature
                .as_ref()
                .expect("Should not fail based on previous checking.")
                .verify(&approval_message, &validator_public_key)
            {
                return Err(HeaderVerificationError::InvalidValidatorSignature {
                    signature: maybe_signature
                        .clone()
                        .expect("Should not fail based on previous checking."),
                    pubkey: validator_public_key,
                });
            }
        }

        if approved_stake * 3 <= total_stake * 2 {
            return Err(HeaderVerificationError::BlockIsNotFinal);
        }

        // If next_bps is not none, sha256(borsh(next_bps)) corresponds to
        // the next_bp_hash in inner_lite.
        if header.light_client_block.next_bps.is_some() {
            let block_view_next_bps_serialized = borsh::to_vec(
                header
                    .light_client_block
                    .next_bps
                    .as_deref()
                    .expect("Should not fail based on previous checking."),
            )
            .expect("Should not fail based on borsh serialization.");
            if sha256(&block_view_next_bps_serialized).as_slice()
                != header.light_client_block.inner_lite.next_bp_hash.as_ref()
            {
                return Err(HeaderVerificationError::InvalidNextBlockProducersHash);
            }
        }

        // Check the `prev_state_root` is the merkle root of `prev_state_root_of_chunks`.
        if header.light_client_block.inner_lite.prev_state_root
            != merklize(&header.prev_state_root_of_chunks).0
        {
            return Err(HeaderVerificationError::InvalidPrevStateRootOfChunks);
        }

        Ok(ancestor)
    }
}
