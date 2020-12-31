use async_graphql::Enum;
use serde::Deserialize;
use sqlx::Type;
use strum::{EnumString, ToString};

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq, Deserialize, EnumString, ToString, Type)]
#[sqlx(rename = "currency_type", rename_all = "UPPERCASE")]
pub enum Currency {
    GBP,
    USD,
}
