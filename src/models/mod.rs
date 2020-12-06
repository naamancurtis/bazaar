use serde::Deserialize;

pub mod cart_item;
pub mod customer;
pub mod shopping_cart;

pub use cart_item::CartItem;
pub use customer::{Customer, CustomerUpdate};
pub use shopping_cart::ShoppingCart;

#[derive(
    Debug,
    async_graphql::Enum,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Deserialize,
    strum::EnumString,
    strum::ToString,
    sqlx::Type,
)]
#[sqlx(rename = "currency_type", rename_all = "UPPERCASE")]
pub enum Currency {
    GBP,
    USD,
}
