pub mod directive;
pub mod ticket;

pub fn schedule_jobs() {
    ic_cdk::spawn(async {
        let _guard = match crate::guard::TimerLogicGuard::new() {
            Some(guard) => guard,
            None => return,
        };

        directive::handle_directives().await;
        ticket::handle_tickets().await;
        //TODO: fetch tx signature and  status
    });
}
