//! Module dealing with the lifecycle methods of the Bitcoin Customs.
pub mod init;
pub use init::init;

pub mod upgrade;
pub use upgrade::post_upgrade;
