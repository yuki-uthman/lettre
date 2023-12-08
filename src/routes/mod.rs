//! src/routes/mod.rs
mod health_check;
pub use health_check::*;

mod subscriptions;
pub use subscriptions::*;

mod subscriptions_confirm;
pub use subscriptions_confirm::*;

pub mod newsletters;

mod home;
pub use home::*;

mod login;
pub use login::*;

mod admin;
pub use admin::admin_dashboard;
pub use admin::change_password;
pub use admin::change_password_form;

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
