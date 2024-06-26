pub use near_client::{
    near_types::{hash::sha256, merkle::merklize, ValidatorStakeView},
    types::*,
    BasicNearLightClient, HeaderVerificationError, StateProofVerificationError,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use thiserror::Error;

const GENESIS_HEIGHT: Height = 9820210;
const EPOCH_DURATION: Height = 43200;

thread_local! {
    static STATES: RefCell<BTreeMap<Height, ConsensusState>> = RefCell::new(Default::default());
    static RPC: RefCell<Option<Rc<String>>> = RefCell::new(None);
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

fn epoch_of_height(height: Height) -> Height {
    if height <= GENESIS_HEIGHT {
        return 0;
    }
    (height - GENESIS_HEIGHT) / EPOCH_DURATION
}

impl DummyLightClient {
    pub fn init_or_return(rpc: impl ToString, state: ConsensusState) {
        STATES.with_borrow_mut(|s| {
            s.entry(state.header.height()).or_insert(state);
        });
        RPC.with_borrow_mut(|r| r.replace(Rc::new(rpc.to_string())));
    }

    pub fn rpc(&self) -> Rc<String> {
        RPC.with_borrow(|r| r.as_ref().expect("Client already initialized.").clone())
    }

    pub async fn verify_proofs(
        &self,
        target: Header,
        key: &[u8],
        value: &[u8],
        proofs: Vec<Vec<u8>>,
    ) -> Result<(), LightClientError> {
        self.ensure_continuous_epoches(&target).await?;
        let state = self
            .verify_header(&target)
            .map_err(|e| LightClientError::HeaderError(e))?;
        // TODO change to verify transactions
        state
            .verify_membership(key, value, &proofs)
            .map_err(|e| LightClientError::StateError(e))
    }

    async fn ensure_continuous_epoches(&self, target: &Header) -> Result<(), LightClientError> {
        let highest_epoch = epoch_of_height(self.latest_height());
        let requested_epoch = epoch_of_height(target.height());
        if highest_epoch >= requested_epoch {
            return Ok(());
        } else if requested_epoch == highest_epoch + 1 {
            let epoch_height =
                EPOCH_DURATION - (self.latest_height() - GENESIS_HEIGHT) % EPOCH_DURATION + 1;
            if let Ok(header) = crate::rpc::fetch_header(self.rpc().as_str(), epoch_height).await {
                let cs = STATES.with_borrow(|s| {
                    s.get(&highest_epoch)
                        .map(|cs| cs.header.light_client_block.next_bps.clone())
                        .flatten()
                        .expect("Should not fail based on previous checking.")
                });
                self.try_accept_new_epoch(header, cs)
                    .map_err(|e| LightClientError::HeaderError(e))?;
            }
            return Ok(());
        } else {
            return Err(LightClientError::HeaderError(
                HeaderVerificationError::InvalidBlockHeight,
            ));
        }
    }

    fn try_accept_new_epoch(
        &self,
        header: Header,
        bps: Vec<ValidatorStakeView>,
    ) -> Result<(), HeaderVerificationError> {
        let height = header.height();
        if let Some(_s) = self.get_consensus_state(&height) {
            return Ok(());
        }
        self.verify_header(&header)?;
        let accepted = ConsensusState {
            current_bps: Some(bps),
            header,
        };
        STATES.with_borrow_mut(|s| {
            s.insert(height, accepted.clone());
        });
        Ok(())
    }

    fn find_nearest_ancestor(&self, header: &Header) -> Option<ConsensusState> {
        let height = header.height();
        STATES.with_borrow(|states| {
            for s in states.keys().rev() {
                if *s < height {
                    return states.get(s).map(|cs| cs.clone());
                }
            }
            return None;
        })
    }

    fn verify_header(&self, header: &Header) -> Result<ConsensusState, HeaderVerificationError> {
        let ancestor = self.find_nearest_ancestor(header).ok_or(
            HeaderVerificationError::MissingCachedEpochBlockProducers {
                epoch_id: header.epoch_id(),
            },
        )?;
        let approval_message = header.light_client_block.approval_message();

        // Check the height of the block is higher than the height of the current head.
        if header.height() < ancestor.header.height() {
            return Err(HeaderVerificationError::InvalidBlockHeight);
        }

        if header.height() == ancestor.header.height() {
            return Ok(ancestor);
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
