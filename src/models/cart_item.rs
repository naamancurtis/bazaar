use async_graphql::{Result, SimpleObject};
use serde::Deserialize;
use sqlx::{query, PgPool};
use std::hash::{Hash, Hasher};

#[derive(Debug, SimpleObject, Deserialize)]
pub struct CartItem {
    pub sku: String,
    pub quantity: i32,
    pub price_per_unit: f64,
    pub name: String,
    pub description: String,
    pub image_src: String,
    pub tags: Vec<String>,
}

// @TODO - Add in discounts struct
// pub struct Discount {
//     id: Uuid,
//     category: DiscountCategory,
//     description:
// }

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
