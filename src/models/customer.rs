use async_graphql::{Context, ErrorExtensions, InputObject, Object};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth,
    database::{CustomerRepository, ShoppingCartDatabase, ShoppingCartRepository},
    models::{Currency, ShoppingCart},
    Result,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub cart_id: Uuid,
    pub refresh_token_count: i32,
}

#[derive(InputObject, Debug, Deserialize)]
pub struct CustomerUpdate {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CustomerIds {
    pub(crate) id: Uuid,
    pub public_id: Uuid,
    pub cart_id: Uuid,
}

#[derive(Debug)]
pub struct NewCustomer {
    pub public_id: Uuid,
    pub private_id: Uuid,
    pub cart_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
}

impl Customer {
    #[tracing::instrument(skip(pool))]
    pub async fn find_all<DB: CustomerRepository>(pool: &PgPool) -> Result<Vec<Self>> {
        DB::find_all(pool).await
    }

    #[tracing::instrument(skip(pool))]
    pub async fn find_by_id<DB: CustomerRepository>(id: Uuid, pool: &PgPool) -> Result<Self> {
        DB::find_by_id(id, pool).await
    }

    #[tracing::instrument(skip(pool, email))]
    pub async fn find_by_email<DB: CustomerRepository>(
        email: String,
        pool: &PgPool,
    ) -> Result<Self> {
        DB::find_by_email(email, pool).await
    }

    #[tracing::instrument(
        name = "new_customer",
        skip(pool, email, password, first_name, last_name)
    )]
    pub async fn new<DB: CustomerRepository>(
        id: Uuid,
        email: String,
        password: String,
        first_name: String,
        last_name: String,
        cart_id: Option<Uuid>,
        pool: &PgPool,
    ) -> Result<CustomerIds> {
        let public_id = Uuid::new_v4();
        let password_hash = auth::hash_password(&password)?;
        let shopping_cart_id = if let Some(id) = cart_id {
            id
        } else {
            Uuid::new_v4()
        };

        let new_customer = NewCustomer {
            public_id,
            private_id: id,
            cart_id: shopping_cart_id,
            email,
            password_hash,
            first_name,
            last_name,
        };

        DB::create_new_user(new_customer, cart_id.is_none(), Currency::GBP, pool).await?;
        Ok(CustomerIds {
            public_id,
            id,
            cart_id: shopping_cart_id,
        })
    }

    #[tracing::instrument(skip(pool, update))]
    pub async fn update<DB: CustomerRepository>(
        id: Uuid,
        update: Vec<CustomerUpdate>,
        pool: &PgPool,
    ) -> Result<Self> {
        DB::update(id, update, pool).await?;
        DB::find_by_id(id, pool).await
    }

    #[tracing::instrument(skip(pool))]
    pub async fn add_new_cart<C: CustomerRepository, SC: ShoppingCartRepository>(
        id: Uuid,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<ShoppingCart> {
        if let Ok(cart_id) = C::check_cart(id, pool).await {
            return ShoppingCart::find_by_id::<SC>(cart_id, pool).await;
        };
        let cart_id = Uuid::new_v4();
        C::add_new_cart(id, cart_id, currency, pool).await
    }

    #[tracing::instrument(skip(pool))]
    pub async fn increment_refresh_token_counter<DB: CustomerRepository>(
        id: Uuid,
        pool: &PgPool,
    ) -> Result<i32> {
        DB::increment_refresh_token_counter(id, pool).await
    }

    #[tracing::instrument(skip(pool))]
    pub async fn fetch_refresh_token_counter<DB: CustomerRepository>(
        id: Uuid,
        pool: &PgPool,
    ) -> Result<i32> {
        DB::fetch_refresh_token_counter(id, pool).await
    }
}

/// Private API
impl Customer {
    #[tracing::instrument(skip(pool))]
    async fn check_cart<DB: CustomerRepository>(id: Uuid, pool: &PgPool) -> Result<Uuid> {
        DB::check_cart(id, pool).await
    }
}

/// Graphql Resolver
#[Object]
impl Customer {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn email(&self) -> String {
        self.email.clone()
    }

    async fn first_name(&self) -> String {
        self.first_name.clone()
    }

    async fn last_name(&self) -> String {
        self.last_name.clone()
    }
    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    async fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }

    async fn cart(&self, ctx: &Context<'_>) -> async_graphql::Result<ShoppingCart> {
        let pool = ctx.data::<PgPool>()?;
        ShoppingCart::find_by_id::<ShoppingCartDatabase>(self.cart_id, pool)
            .await
            .map_err(|e| e.extend())
    }
}

impl CustomerIds {
    /// This should never be used in the actual application
    /// This is purely for testing
    ///
    /// Unfortunately there doesn't seem to be a way to put it behind
    /// a `cfg(test)` flag if we want it accessible in integration tests
    pub fn get_private_id(&self) -> Uuid {
        self.id
    }
}
