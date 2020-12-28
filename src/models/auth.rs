use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{database::AuthRepository, BazaarError};

#[derive(Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct AuthCustomer {
    pub public_id: Uuid,
    pub id: Uuid,
    pub password_hash: String,
}

impl AuthCustomer {
    pub async fn find_by_email<DB: AuthRepository>(
        email: &str,
        pool: &PgPool,
    ) -> Result<Self, BazaarError> {
        DB::get_auth_customer(email, pool).await
    }
}
