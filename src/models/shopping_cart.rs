use async_graphql::{Result, SimpleObject};
use serde::Deserialize;
use sqlx::{query, PgPool};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, async_graphql::Enum, Copy, Clone, Eq, PartialEq, Deserialize, strum::EnumString)]
pub enum Currency {
    GBP,
    USD,
}

#[derive(Debug, async_graphql::Enum, Copy, Clone, Eq, PartialEq, Deserialize)]
pub enum CartType {
    Anonymous,
    Known,
}

impl From<i16> for CartType {
    fn from(num: i16) -> Self {
        match num {
            0 => Self::Anonymous,
            1 => Self::Known,
            _ => panic!("invalid cart type"),
        }
    }
}

// @TODO - Add in discounts struct
// pub struct Discount {
//     id: Uuid,
//     category: DiscountCategory,
//     description:
// }

#[derive(Debug, SimpleObject, Deserialize)]
pub struct CartItem {
    pub sku: String,
    pub quantity: u32,
    pub price_per_unit: f64,
    pub name: String,
    pub description: String,
}

impl Hash for CartItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sku.hash(state);
    }
}

impl PartialEq for CartItem {
    fn eq(&self, other: &Self) -> bool {
        self.sku == other.sku
    }
}

impl Eq for CartItem {}

#[derive(Debug, SimpleObject, Deserialize)]
pub struct ShoppingCart {
    pub id: Uuid,
    pub cart_type: CartType,
    pub items: Vec<String>,
    pub price_before_discounts: f64,
    pub discounts: Option<Vec<Uuid>>, // See @TODO above
    pub price_after_discounts: f64,
    pub currency: Currency,
}

impl ShoppingCart {
    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Option<Self>> {
        let cart = query!(
            r#"
                SELECT 
                id, 
                items, 
                cart_type, 
                price_before_discounts, 
                price_after_discounts,
                discounts,
                currency
                FROM shopping_carts WHERE id = $1
                "#,
            id
        )
        .fetch_one(pool)
        .await?;

        let cart = Self {
            id: cart.id,
            items: cart.items.unwrap_or_else(Vec::default),
            cart_type: CartType::from(cart.cart_type),
            price_before_discounts: cart.price_before_discounts,
            discounts: cart.discounts,
            price_after_discounts: cart.price_after_discounts,
            currency: Currency::from_str(&cart.currency)?,
        };

        Ok(Some(cart))
    }
}
