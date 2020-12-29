use async_graphql::{Context, InputObject, Object};
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
    pub cart_id: Option<Uuid>,
}

#[derive(InputObject, Debug, Deserialize)]
pub struct CustomerUpdate {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CustomerIds {
    id: Uuid,
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
    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_all<DB: CustomerRepository>(pool: &PgPool) -> Result<Vec<Self>> {
        DB::find_all(pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_by_id<DB: CustomerRepository>(id: Uuid, pool: &PgPool) -> Result<Self> {
        DB::find_by_id(id, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn find_by_email<DB: CustomerRepository>(
        email: String,
        pool: &PgPool,
    ) -> Result<Self> {
        DB::find_by_email(email, pool).await
    }

    #[tracing::instrument(
        name = "new_customer",
        skip(pool, email, password, first_name, last_name),
        fields(model = "Customer")
    )]
    pub async fn new<DB: CustomerRepository>(
        id: Uuid,
        email: String,
        password: String,
        first_name: String,
        last_name: String,
        cart_id: Uuid,
        pool: &PgPool,
    ) -> Result<CustomerIds> {
        let public_id = Uuid::new_v4();
        let password_hash = auth::hash_password(&password)?;

        let new_customer = NewCustomer {
            public_id,
            private_id: id,
            cart_id,
            email,
            password_hash,
            first_name,
            last_name,
        };

        DB::create_new_user(new_customer, Currency::GBP, pool).await?;
        Ok(CustomerIds {
            public_id,
            id,
            cart_id,
        })
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn update<DB: CustomerRepository>(
        id: Uuid,
        update: Vec<CustomerUpdate>,
        pool: &PgPool,
    ) -> Result<Self> {
        DB::update(id, update, pool).await?;
        DB::find_by_id(id, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    pub async fn add_new_cart<C: CustomerRepository, SC: ShoppingCartRepository>(
        id: Uuid,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<ShoppingCart> {
        if let Some(cart_id) = C::check_cart(id, pool).await {
            return ShoppingCart::find_by_id::<SC>(cart_id, pool).await;
        };
        let cart_id = Uuid::new_v4();
        C::add_new_cart(id, cart_id, currency, pool).await
    }
}

/// Private API
impl Customer {
    #[tracing::instrument(skip(pool), fields(model = "Customer"))]
    async fn check_cart<DB: CustomerRepository>(id: Uuid, pool: &PgPool) -> Option<Uuid> {
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

    async fn cart(&self, ctx: &Context<'_>) -> Option<ShoppingCart> {
        let pool = ctx.data::<PgPool>().ok()?;
        if let Some(cart_id) = self.cart_id {
            return ShoppingCart::find_by_id::<ShoppingCartDatabase>(cart_id, pool)
                .await
                .ok();
        }
        None
    }
}
