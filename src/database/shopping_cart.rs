use async_trait::async_trait;
use serde_json::Value;
use sqlx::{query, query_as, types::Json, PgPool};
use uuid::Uuid;

use crate::{
    models::{
        cart_item::InternalCartItem,
        shopping_cart::{CartType, SqlxShoppingCart},
        Currency, ShoppingCart,
    },
    Result,
};

#[async_trait]
pub trait ShoppingCartRepository {
    async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<ShoppingCart>;
    async fn find_by_customer_id(id: Uuid, pool: &PgPool) -> Result<ShoppingCart>;
    async fn find_cart_id_by_customer_id(id: Uuid, pool: &PgPool) -> Result<Uuid>;
    async fn create_new_cart(
        id: Uuid,
        customer_id: Option<Uuid>,
        cart_type: CartType,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<ShoppingCart>;
    async fn update_cart(
        cart: &ShoppingCart,
        items_array: serde_json::Value,
        pool: &PgPool,
    ) -> Result<ShoppingCart>;
    async fn update_cart_type(id: Uuid, cart_type: CartType, pool: &PgPool) -> Result<Uuid>;
}

pub struct ShoppingCartDatabase;

#[async_trait]
impl ShoppingCartRepository for ShoppingCartDatabase {
    #[tracing::instrument(skip(pool), fields(repository = "shopping_cart"))]
    async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<ShoppingCart> {
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

    #[tracing::instrument(skip(pool), fields(repository = "shopping_cart"))]
    async fn find_by_customer_id(id: Uuid, pool: &PgPool) -> Result<ShoppingCart> {
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
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(cart.into())
    }

    #[tracing::instrument(skip(pool), fields(repository = "shopping_cart"))]
    async fn find_cart_id_by_customer_id(id: Uuid, pool: &PgPool) -> Result<Uuid> {
        let cart_id = query!(
            r#"
            SELECT id FROM shopping_carts WHERE customer_id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(cart_id.id)
    }

    #[tracing::instrument(skip(pool), fields(repository = "shopping_cart"))]
    async fn create_new_cart(
        id: Uuid,
        customer_id: Option<Uuid>,
        cart_type: CartType,
        currency: Currency,
        pool: &PgPool,
    ) -> Result<ShoppingCart> {
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

    #[tracing::instrument(skip(pool), fields(repository = "shopping_cart"))]
    async fn update_cart(
        cart: &ShoppingCart,
        items_array: Value,
        pool: &PgPool,
    ) -> Result<ShoppingCart> {
        let cart = query_as!(
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
            cart.price_before_discounts,
            cart.price_after_discounts,
            items_array,
            cart.id
        )
        .fetch_one(pool)
        .await?;
        Ok(cart.into())
    }

    #[tracing::instrument(skip(pool), fields(repository = "shopping_cart"))]
    async fn update_cart_type(id: Uuid, cart_type: CartType, pool: &PgPool) -> Result<Uuid> {
        let cart = query!(
            r#"
            UPDATE shopping_carts
            SET cart_type = $1
            WHERE id = $2
            RETURNING id
            "#,
            cart_type as CartType,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(cart.id)
    }
}
