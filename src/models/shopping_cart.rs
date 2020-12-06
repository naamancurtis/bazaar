use async_graphql::{Context, Object, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query_as, types::Json, PgPool};
use std::collections::HashSet;
use std::iter::FromIterator;
use tracing::{debug, error};
use uuid::Uuid;

use crate::models::{cart_item::InternalCartItem, CartItem, Currency};

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
    pub customer_id: Option<Uuid>,
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
    pub customer_id: Option<Uuid>,
    pub cart_type: CartType,
    pub items: Json<Vec<InternalCartItem>>,
    pub discounts: Option<Vec<Uuid>>,
    pub price_before_discounts: f64,
    pub price_after_discounts: f64,
    pub currency: Currency,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

impl ShoppingCart {
    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Self> {
        let cart = query_as!(
            SqlxShoppingCart,
            r#"
            SELECT
                id, customer_id,
                cart_type as "cart_type!: CartType", 
                items as "items!: Json<Vec<InternalCartItem>>",
                currency as "currency!: Currency",
                discounts, price_before_discounts, price_after_discounts,
                created_at, last_modified
            FROM shopping_carts WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(cart.into())
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn find_by_customer_id(customer_id: Uuid, pool: &PgPool) -> Result<Self> {
        let cart = query_as!(
            SqlxShoppingCart,
            r#"
            SELECT
                id, customer_id,
                cart_type as "cart_type!: CartType", 
                items as "items!: Json<Vec<InternalCartItem>>",
                currency as "currency!: Currency",
                discounts, price_before_discounts, price_after_discounts,
                created_at, last_modified
            FROM shopping_carts WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_one(pool)
        .await?;

        Ok(cart.into())
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn new_anonymous(currency: Currency, pool: &PgPool) -> Result<Self> {
        ShoppingCart::new(Uuid::new_v4(), None, CartType::Anonymous, currency, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn new_known(
        id: Uuid,
        customer_id: Uuid,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<Self> {
        ShoppingCart::new(id, Some(customer_id), CartType::Known, currency, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn edit_cart_items(
        cart_id: Uuid,
        items: Vec<InternalCartItem>,
        pool: &PgPool,
    ) -> Result<Self> {
        let mut cart = Self::find_by_id(cart_id, pool).await?;
        cart.update_items_in_cart(items);
        cart.update_cart(pool).await
    }
}

/// Private API
impl ShoppingCart {
    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    async fn new(
        id: Uuid,
        customer_id: Option<Uuid>,
        cart_type: CartType,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<Self> {
        let cart = query_as!(
            SqlxShoppingCart,
            r#"
            INSERT INTO shopping_carts (id, customer_id, cart_type, currency)
            VALUES ( $1, $2, $3, $4)
            RETURNING
                id, customer_id, 
                cart_type as "cart_type!: CartType", 
                items as "items!: Json<Vec<InternalCartItem>>",
                currency as "currency!: Currency",
                discounts, price_before_discounts, price_after_discounts,
                created_at, last_modified
            "#,
            id,
            customer_id,
            cart_type as CartType,
            currency as Currency
        )
        .fetch_one(pool)
        .await?;

        Ok(cart.into())
    }

    #[tracing::instrument(fields(model = "ShoppingCart"))]
    fn update_items_in_cart(&mut self, items: Vec<InternalCartItem>) {
        let mut current_cart_items = Vec::new();
        std::mem::swap(&mut self.items, &mut current_cart_items);
        let mut item_set: HashSet<InternalCartItem> = HashSet::from_iter(current_cart_items);
        for item in items {
            let updated_item = match item_set.take(&item) {
                Some(old_item) => old_item + item,
                None => item,
            };
            if updated_item.quantity > 0 {
                item_set.insert(updated_item);
            }
        }
        self.items = item_set.into_iter().collect::<Vec<InternalCartItem>>();
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    async fn update_cart(&mut self, pool: &PgPool) -> Result<Self> {
        let cart_items = CartItem::find_multiple(&self.items, pool).await?;
        self.price_before_discounts = cart_items.iter().fold(0f64, |mut acc, item| {
            acc += item.price_per_unit * item.quantity as f64;
            acc
        });
        // @TODO - Add in discounts stuff
        self.price_after_discounts = self.price_before_discounts;

        // Work around until SQLx supports an Array of Custom Types (their goal
        // is for 0.5 release)
        let items_array = serde_json::to_value(&self.items)?;
        debug!(?items_array, "json stringified the items to update");
        let cart = match query_as!(
            SqlxShoppingCart,
            r#"
            UPDATE shopping_carts
            SET price_before_discounts = $1, price_after_discounts = $2, items = $3::jsonb
            WHERE id = $4
            RETURNING 
                id, customer_id, 
                cart_type as "cart_type!: CartType", 
                items as "items!: Json<Vec<InternalCartItem>>",
                currency as "currency!: Currency",
                discounts, price_before_discounts, price_after_discounts,
                created_at, last_modified
            "#,
            self.price_before_discounts,
            self.price_after_discounts,
            items_array,
            self.id
        )
        .fetch_one(pool)
        .await
        {
            Ok(cart) => cart,
            Err(e) => {
                error!(?e);
                return Err(e.into());
            }
        };

        Ok(cart.into())
    }
}

impl From<SqlxShoppingCart> for ShoppingCart {
    fn from(cart: SqlxShoppingCart) -> Self {
        Self {
            id: cart.id,
            customer_id: cart.customer_id,
            items: cart.items.to_vec(),
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

#[Object]
impl ShoppingCart {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn customer_id(&self) -> Option<Uuid> {
        self.customer_id
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

    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    async fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }

    // @TODO - Implement proper error handling for this - theres quite a few layers that could
    // potentially go wrong
    async fn items(&self, ctx: &Context<'_>) -> Vec<CartItem> {
        if self.items.is_empty() {
            return Vec::new();
        }
        if let Ok(pool) = ctx.data::<PgPool>() {
            let items = CartItem::find_multiple(&self.items, pool)
                .await
                .expect("error occurred while trying to find cart items");
            return items;
        }
        Vec::new()
    }
}
