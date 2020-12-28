use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{types::Json, PgPool};
use std::collections::HashSet;
use std::iter::FromIterator;
use tracing::debug;
use uuid::Uuid;

use crate::{
    database::{CartItemDatabase, CartItemRepository, ShoppingCartRepository},
    models::{cart_item::InternalCartItem, CartItem, Currency},
    Result,
};

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

pub(crate) struct SqlxShoppingCart {
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
    pub async fn find_by_id<DB: ShoppingCartRepository>(id: Uuid, pool: &PgPool) -> Result<Self> {
        DB::find_by_id(id, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn find_by_customer_id<DB: ShoppingCartRepository>(
        customer_id: Uuid,
        pool: &PgPool,
    ) -> Result<Self> {
        DB::find_by_customer_id(customer_id, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn find_cart_id_by_customer_id<DB: ShoppingCartRepository>(
        customer_id: Uuid,
        pool: &PgPool,
    ) -> Result<Uuid> {
        DB::find_cart_id_by_customer_id(customer_id, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn new_anonymous<DB: ShoppingCartRepository>(
        currency: Currency,
        pool: &PgPool,
    ) -> Result<Self> {
        ShoppingCart::new::<DB>(Uuid::new_v4(), None, CartType::Anonymous, currency, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn new_known<DB: ShoppingCartRepository>(
        id: Uuid,
        customer_id: Uuid,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<Self> {
        ShoppingCart::new::<DB>(id, Some(customer_id), CartType::Known, currency, pool).await
    }

    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    pub async fn edit_cart_items<DB: ShoppingCartRepository, CI: CartItemRepository>(
        cart_id: Uuid,
        items: Vec<InternalCartItem>,
        pool: &PgPool,
    ) -> Result<Self> {
        let mut cart = Self::find_by_id::<DB>(cart_id, pool).await?;
        cart.update_items_in_cart(items);
        cart.update_cart::<DB, CI>(pool).await
    }
}

/// Private API
impl ShoppingCart {
    #[tracing::instrument(skip(pool), fields(model = "ShoppingCart"))]
    async fn new<DB: ShoppingCartRepository>(
        id: Uuid,
        customer_id: Option<Uuid>,
        cart_type: CartType,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<Self> {
        DB::create_new_cart(id, customer_id, cart_type, currency, pool).await
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
    async fn update_cart<SC: ShoppingCartRepository, CI: CartItemRepository>(
        &mut self,
        pool: &PgPool,
    ) -> Result<Self> {
        let cart_items = CartItem::find_multiple::<CI>(&self.items, pool).await?;
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
        SC::update_cart(&self, items_array, pool).await
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
            let items = CartItem::find_multiple::<CartItemDatabase>(&self.items, pool)
                .await
                .expect("error occurred while trying to find cart items");
            return items;
        }
        Vec::new()
    }
}
