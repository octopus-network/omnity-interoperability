use crate::state::mutate_state;

#[must_use]
pub struct ProcessDirectiveMsgGuard(());

impl ProcessDirectiveMsgGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_process_directive_msg {
                return None;
            }
            s.is_process_directive_msg = true;
            Some(ProcessDirectiveMsgGuard(()))
        })
    }
}

impl Drop for ProcessDirectiveMsgGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_process_directive_msg = false;
        });
    }
}

#[must_use]
pub struct ProcessTicketMsgGuard(());

impl ProcessTicketMsgGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_process_ticket_msg {
                return None;
            }
            s.is_process_ticket_msg = true;
            Some(ProcessTicketMsgGuard(()))
        })
    }
}

impl Drop for ProcessTicketMsgGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_process_ticket_msg = false;
        });
    }
}
