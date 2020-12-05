use async_graphql::{InputObject, Result, SimpleObject};
use serde::{Serialize, Deserialize};
use sqlx::{query, PgPool};
use std::hash::{Hash, Hasher};

#[derive(Debug, SimpleObject, Deserialize, Clone)]
pub struct CartItem {
    pub sku: String,
    pub quantity: i32,
    pub price_per_unit: f64,
    pub name: String,
    pub description: String,
    pub img_src: String,
    pub tags: Vec<String>,
}

#[derive(Debug, InputObject, Deserialize, Clone)]
pub struct AddCartItem {
    pub sku: String,
    pub quantity: u32,
}

impl CartItem {
    #[tracing::instrument(skip(pool), fields(model = "CartItem"))]
    pub async fn find_multiple(
        internal_items: &[InternalCartItem],
        pool: &PgPool,
    ) -> Result<Vec<CartItem>> {
        let items = query!(
            "SELECT * FROM items WHERE sku IN ($1) ORDER BY sku ASC",
            internal_items
                .iter()
                .map(|i| i.sku.clone())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .fetch_all(pool)
        .await?;

        let mut internal_items = internal_items.to_vec();
        internal_items.sort_by(|a, b| a.sku.cmp(&b.sku));

        let result = items
            .into_iter()
            .zip(internal_items.into_iter())
            .map(|(item, mapper)| {
                assert_eq!(item.sku, mapper.sku);
                Self {
                    sku: item.sku,
                    quantity: mapper.quantity,
                    price_per_unit: item.price,
                    name: item.name,
                    description: item.description,
                    img_src: item.img_src,
                    tags: item.tags,
                }
            })
            .collect();

        Ok(result)
    }
}

// @TODO - Add in discounts struct
// pub struct Discount {
//     id: Uuid,
//     category: DiscountCategory,
//     description:
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternalCartItem {
    pub sku: String,
    pub quantity: i32,
}

impl Hash for InternalCartItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sku.hash(state);
    }
}

impl PartialEq for InternalCartItem {
    fn eq(&self, other: &Self) -> bool {
        self.sku == other.sku
    }
}

impl Eq for InternalCartItem {}

impl From<(String, i32)> for InternalCartItem {
    fn from((sku, quantity): (String, i32)) -> Self {
        Self { sku, quantity }
    }
}

impl From<AddCartItem> for InternalCartItem {
    fn from(item: AddCartItem) -> Self {
        Self {
            sku: item.sku,
            quantity: item.quantity as i32,
        }
    }
}

impl std::ops::Add for InternalCartItem {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            sku: self.sku,
            quantity: self.quantity + other.quantity,
        }
    }
}

impl std::ops::Sub for InternalCartItem {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            sku: self.sku,
            quantity: self.quantity - other.quantity,
        }
    }
}
