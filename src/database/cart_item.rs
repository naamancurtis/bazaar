use async_trait::async_trait;
use sqlx::{query, PgPool};

use crate::{models::CartItem, Result};

#[async_trait]
pub trait CartItemRepository {
    async fn find_multiple(items: &[String], pool: &PgPool) -> Result<Vec<CartItem>>;
}

pub struct CartItemDatabase;

#[async_trait]
impl CartItemRepository for CartItemDatabase {
    async fn find_multiple(items: &[String], pool: &PgPool) -> Result<Vec<CartItem>> {
        let items = query!(
            "SELECT * FROM items WHERE sku = ANY ($1) ORDER BY sku ASC",
            items
        )
        .fetch_all(pool)
        .await?;

        Ok(items
            .into_iter()
            .map(|item| CartItem {
                sku: item.sku,
                quantity: 0,
                price_per_unit: item.price,
                name: item.name,
                description: item.description,
                img_src: item.img_src,
                tags: item.tags,
            })
            .collect())
    }
}
