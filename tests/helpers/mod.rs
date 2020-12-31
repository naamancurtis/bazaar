#![allow(dead_code)]
mod app;
mod constants;
mod database;
mod graphql;
mod math;
mod reqwest;
mod tokens;
mod types;

pub use self::reqwest::*;
pub use app::{spawn_app, IdHolder, TestApp};
pub use constants::*;
pub use database::*;
pub use graphql::parse_graphql_response;
pub use math::assert_on_decimal;
pub use tokens::*;
pub use types::*;

use lazy_static::lazy_static;
use serde_json::json;
use uuid::Uuid;

use bazaar::telemetry::{generate_subscriber, init_subscriber};

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
            ""
        };
        let subscriber = generate_subscriber("test".to_string(), filter.into());
        init_subscriber(subscriber);
    };

    pub static ref DEFAULT_CUSTOMER: serde_json::Value = {
        json!({
            "email": format!("{}@test.com", Uuid::nil()),
            "firstName": Uuid::nil(),
            "lastName": Uuid::nil()
        })
    };
}
