use async_graphql::connection::{query, Connection, Edge, EmptyFields};
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Enum, Interface, Object, Result, Schema,
};
use uuid::Uuid;

pub type BazarSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn customer(&self, ctx: &Context<'_>, id: Uuid) -> Customer {
        todo!();
    }
}

pub struct Customer(Uuid);

#[Object]
impl Customer {
    async fn first_name(&self, ctx: &Context<'_>) -> &str {
        todo!()
    }
}
