use async_graphql::{Object, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query, query_as, PgPool, Type};
use uuid::Uuid;

use crate::models::{CartItem, Currency};

#[derive(Debug, async_graphql::Enum, Copy, Clone, Eq, PartialEq, Deserialize, sqlx::Type)]
#[sqlx(rename = "user_cart_type", rename_all = "UPPERCASE")]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum CartType {
    Anonymous,
    Known,
}

#[derive(Debug, Deserialize, sqlx::FromRow)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct ShoppingCart {
    pub id: Uuid,
    pub cart_type: CartType,
    pub items: Vec<InternalCartItem>,
    pub discounts: Option<Vec<Uuid>>,
    pub price_before_discounts: f64,
    pub price_after_discounts: f64,
    pub currency: Currency,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

struct SqlxShoppingCart {
    pub id: Uuid,
    pub cart_type: CartType,
    pub items: Vec<(String, i32)>,
    pub discounts: Option<Vec<Uuid>>,
    pub price_before_discounts: f64,
    pub price_after_discounts: f64,
    pub currency: Currency,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

impl ShoppingCart {
    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Option<Self>> {
        let cart = query_as!(
            SqlxShoppingCart,
            r#"
            SELECT
                id, 
                cart_type as "cart_type!: CartType", 
                items as "items!: Vec<(String, i32)>",
                currency as "currency!: Currency",
                discounts, price_before_discounts, price_after_discounts,
                created_at, last_modified
            FROM shopping_carts WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .map(|cart| cart.into());

        Ok(cart)
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn new_anonymous(currency: Currency, pool: &PgPool) -> Result<Self> {
        ShoppingCart::new(Uuid::new_v4(), CartType::Anonymous, currency, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn new_known(id: Uuid, currency: Currency, pool: &PgPool) -> Result<Self> {
        ShoppingCart::new(id, CartType::Known, currency, pool).await
    }
}

/// Private API
impl ShoppingCart {
    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    async fn new(id: Uuid, cart_type: CartType, currency: Currency, pool: &PgPool) -> Result<Self> {
        let cart = query_as!(
            SqlxShoppingCart,
            r#"
            INSERT INTO shopping_carts (id, cart_type, currency)
            VALUES ( $1, $2, $3 )
            RETURNING
                id, 
                cart_type as "cart_type!: CartType", 
                items as "items!: Vec<(String, i32)>",
                currency as "currency!: Currency",
                discounts, price_before_discounts, price_after_discounts,
                created_at, last_modified
            "#,
            id,
            cart_type as CartType,
            currency as Currency
        )
        .fetch_one(pool)
        .await?;

        Ok(cart.into())
    }
}

impl From<SqlxShoppingCart> for ShoppingCart {
    fn from(cart: SqlxShoppingCart) -> Self {
        Self {
            id: cart.id,
            items: cart.items.into_iter().map(|v| v.into()).collect(),
            cart_type: cart.cart_type,
            price_before_discounts: cart.price_before_discounts,
            discounts: cart.discounts,
            price_after_discounts: cart.price_after_discounts,
            currency: cart.currency,
            created_at: cart.created_at,
            last_modified: cart.last_modified,
        }
    }
}

#[derive(Debug, Deserialize, sqlx::Type)]
#[sqlx(rename = "internal_cart_item")]
pub struct InternalCartItem {
    pub sku: String,
    pub quantity: i32,
}

impl PartialEq for InternalCartItem {
    fn eq(&self, other: &Self) -> bool {
        self.sku == other.sku
    }
}

impl Eq for InternalCartItem {}

impl From<(String, i32)> for InternalCartItem {
    fn from((sku, quantity): (String, i32)) -> Self {
        Self { sku, quantity }
    }
}

impl std::ops::Add for InternalCartItem {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            sku: self.sku,
            quantity: self.quantity + other.quantity,
        }
    }
}

impl std::ops::Sub for InternalCartItem {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            sku: self.sku,
            quantity: self.quantity - other.quantity,
        }
    }
}

#[Object]
impl ShoppingCart {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn cart_type(&self) -> CartType {
        self.cart_type
    }

    async fn discounts(&self) -> Option<Vec<Uuid>> {
        None
    }

    async fn price_before_discounts(&self) -> f64 {
        self.price_before_discounts
    }

    async fn price_after_discounts(&self) -> f64 {
        self.price_after_discounts
    }

    async fn currency(&self) -> Currency {
        self.currency
    }

    async fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }
}
