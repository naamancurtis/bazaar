pub mod auth;
pub mod configuration;
pub mod database;
mod error;
mod graphql;
pub mod models;
pub mod routes;
mod startup;
pub mod telemetry;

pub use configuration::get_configuration;
pub use error::BazaarError;
pub use graphql::{BazaarSchema, MutationRoot, QueryRoot};
pub use startup::{build_app, generate_schema};

pub type Result<T> = std::result::Result<T, BazaarError>;

#[cfg(test)]
pub mod test_helpers;
