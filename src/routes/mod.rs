//! src/routes/mod.rs
mod health_check;
mod subscriptions;
mod subscriptions_confirm;

pub mod newsletters;

pub use health_check::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
