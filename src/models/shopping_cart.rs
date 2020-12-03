use async_graphql::{
    validators::{Email, StringMinLength},
    InputObject, Result, SimpleObject,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query_as, PgPool};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, async_graphql::Enum, Copy, Clone, Eq, PartialEq, Deserialize)]
pub enum Currency {
    GBP,
    USD,
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
    pub items: HashSet<CartItem>,
    pub price_before_discounts: f64,
    pub discounts_applied: Vec<f64>, // See @TODO above
    pub price_after_discounts: f64,
    pub currency: Currency,
}
