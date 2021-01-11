pub mod auth;
mod build_app;
pub mod configuration;
mod constants;
pub mod database;
mod error;
mod graphql;
pub mod models;
pub mod routes;
pub mod telemetry;

pub use build_app::{build_app, generate_schema};
pub use configuration::get_configuration;
pub use constants::*;
pub use error::BazaarError;
pub use graphql::{BazaarSchema, MutationRoot, QueryRoot};

pub type Result<T> = std::result::Result<T, BazaarError>;

#[cfg(test)]
pub mod test_helpers;
