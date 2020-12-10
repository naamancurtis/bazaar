use anyhow::Result;
use assert_json_diff::assert_json_include;
use chrono::DateTime;
use serde_json::json;
use uuid::Uuid;

use bazaar::{
    database::{CustomerDatabase, ShoppingCartDatabase},
    models::{cart_item::InternalCartItem, Customer, ShoppingCart},
};

pub mod helpers;
mod mutation;
mod query;

use helpers::*;

pub const CUSTOMER_GRAPHQL_FIELDS: &str = "#
id,
firstName,
lastName,
email,
createdAt,
lastModified
#";

pub const SHOPPING_CART_GRAPHQL_FIELDS: &str = "#
id
cartType
items {
   sku 
   quantity
   pricePerUnit
   name
   tags
}
priceBeforeDiscounts
discounts
priceAfterDiscounts
currency
lastModified
createdAt
#";


