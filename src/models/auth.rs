use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{database::AuthRepository, Result};

#[derive(Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct AuthCustomer {
    pub public_id: Uuid,
    pub(crate) id: Uuid,
    pub hashed_password: String,
}

impl AuthCustomer {
    pub async fn map_id<DB: AuthRepository>(
        public_id: Option<Uuid>,
        pool: &PgPool,
    ) -> Result<Option<Uuid>> {
        DB::map_id(public_id, pool).await
    }

    pub async fn find_by_email<DB: AuthRepository>(email: &str, pool: &PgPool) -> Result<Self> {
        DB::get_auth_customer(email, pool).await
    }
}
