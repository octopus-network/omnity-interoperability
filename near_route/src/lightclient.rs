pub use near_client::{types::*, BasicNearLightClient, HeaderVerificationError};
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<ConsensusState>> = RefCell::new(None);
}

pub struct DummyLightClient;

impl BasicNearLightClient for DummyLightClient {
    fn latest_height(&self) -> Height {
        STATE.with_borrow(|s| s.as_ref().expect("Client not initialized").header.height())
    }

    fn get_consensus_state(&self, height: &Height) -> Option<ConsensusState> {
        STATE.with_borrow(|s| {
            let state = s.as_ref().expect("Client not initialized");
            if state.header.height() == *height {
                return Some(state.clone());
            }
            None
        })
    }
}

impl DummyLightClient {
    fn init_or_return(state: ConsensusState) {
        STATE.with_borrow_mut(|s| {
            if s.is_none() {
                *s = Some(state)
            }
        });
    }

    fn update_state(&self, header: Header) -> Result<Height, HeaderVerificationError> {
        let height = header.height();
        if self.latest_height() >= height {
            return Ok(self.latest_height());
        }
        // TODO is this behaviour expected?
        let current_bps = self
            .get_consensus_state(&self.latest_height())
            .map(|cs| cs.get_block_producers_of(&header.epoch_id()))
            .ok_or(HeaderVerificationError::MissingNextBlockProducersInHead)?;
        STATE.with_borrow_mut(|s| {
            *s = Some(ConsensusState {
                current_bps,
                header,
            })
        });
        Ok(height)
    }

    fn verify_to_height(&self, headers: Vec<Header>) -> Result<Height, HeaderVerificationError> {
        let mut height = self.latest_height();
        for header in headers.into_iter() {
            // fault tolerant since `verify_header` doesn't allow to re-input passed headers
            if header.height() <= height {
                continue;
            }
            self.verify_header(&header)?;
            height = self.update_state(header)?;
        }
        Ok(height)
    }
}
