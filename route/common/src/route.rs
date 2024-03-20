// use candid::Principal;
// use omnity_types::Ticket;

// pub trait RouteCanister {
//     fn hub_addr(&self) -> Principal;
//     fn ticket_sequence(&self) -> u64;
//     fn ticket_query_limit(&self) -> u32;
//     fn update_ticket_sequence(&self, seq: u64);
// }

// pub trait Transport: RouteCanister {
//     fn query_tickets(&self, seq: u64, limit: u32) -> Vec<Ticket>;
//     fn send_transaction_by_ticket(&self, ticket: Ticket) -> crate::error::Result;
//     fn transport(&self) -> crate::error::Result {
//         let seq = self.ticket_sequence();
//         let limit = self.ticket_query_limit();
//         let tickets = self.query_tickets(seq, limit);

//         let mut success_ticket_count = 0;
//         for ticket in tickets {
//             match self.send_transaction_by_ticket(ticket) {
//                 Ok(_) => {
//                     success_ticket_count += 1;
//                 },
//                 Err(error) => {
//                     self.update_ticket_sequence(seq + success_ticket_count);
//                     return Err(error)
//                 },
//             }
//         }
//         self.update_ticket_sequence(seq + success_ticket_count);
//         Ok(())
//     }
// }

// pub trait Redeem: RouteCanister {
//     fn confirm_transaction(&self);
//     fn generate_ticket(&self)->Ticket;
//     fn send_ticket(&self, ticket: Ticket) -> crate::error::Result {
        
//     }
// }
