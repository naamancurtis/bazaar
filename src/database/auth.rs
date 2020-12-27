use async_trait::async_trait;
use sqlx::{query, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait AuthRepository {
    async fn map_id(id: Option<Uuid>, pool: &PgPool) -> Option<Uuid>;
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
}
