use anyhow::Result;
use sqlx::{query, PgPool};
use uuid::Uuid;

use crate::auth::authenticate::verify_password;

pub async fn verify_user_password(
    public_id: Uuid,
    password: String,
    pool: &PgPool,
) -> Result<bool> {
    let result = query!(
        r#"
            SELECT password_hash FROM auth WHERE public_id = $1
        "#,
        public_id
    )
    .fetch_one(pool)
    .await?;
    verify_password(&password, &result.password_hash)
}
