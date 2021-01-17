#![allow(dead_code)]
mod app;
mod constants;
mod database;
mod env_vars;
mod graphql;
mod math;
mod reqwest;
mod types;

pub use self::reqwest::*;
pub use app::{spawn_app, IdHolder, TestApp};
pub use constants::*;
pub use database::*;
pub use env_vars::set_env_vars_for_tests;
pub use graphql::parse_graphql_response;
pub use math::assert_on_decimal;
pub use types::*;

use lazy_static::lazy_static;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

lazy_static! {
    /// To ensure logs are only outputted in tests when required, by default
    /// tests run with no logs being captured
    ///
    /// In order to set logs to be captured during tests run them with:
    /// `TEST_LOG=true cargo test | bunyan`
    pub static ref TRACING: () = {
        let filter = if std::env::var("TEST_LOG").is_ok() {
            "debug"
        } else {
            "100"
        };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));
    LogTracer::init().expect("failed to attach logs to tracing");
    let registry = Registry::default()
        .with(env_filter);
        set_global_default(registry).unwrap();
    };
}
