mod mutation;
mod query;
mod validators;

use async_graphql::{EmptySubscription, Schema};

pub use mutation::MutationRoot;
pub use query::QueryRoot;
pub type BazaarSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
