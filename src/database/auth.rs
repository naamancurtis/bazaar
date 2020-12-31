use async_trait::async_trait;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

use crate::{models::auth::AuthCustomer, Result};

#[async_trait]
pub trait AuthRepository {
    async fn map_id(id: Option<Uuid>, pool: &PgPool) -> Option<Uuid>;
    async fn get_auth_customer(email: &str, pool: &PgPool) -> Result<AuthCustomer>;
}

pub struct AuthDatabase;

#[async_trait]
impl AuthRepository for AuthDatabase {
    #[tracing::instrument(skip(pool, id), fields(repository = "auth"))]
    async fn map_id(id: Option<Uuid>, pool: &PgPool) -> Option<Uuid> {
        if id.is_none() {
            return id;
        }
        let private_id = query!(
            r#"
            SELECT id FROM auth WHERE public_id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|s| s.id)?;
        Some(private_id)
    }

    #[tracing::instrument(skip(pool, email), fields(repository = "auth"))]
    async fn get_auth_customer(email: &str, pool: &PgPool) -> Result<AuthCustomer> {
        let customer = query_as!(
            AuthCustomer,
            r#"
            SELECT public_id, id, password_hash FROM auth WHERE email = $1
            "#,
            email
        )
        .fetch_one(pool)
        .await?;
        Ok(customer)
    }
}
