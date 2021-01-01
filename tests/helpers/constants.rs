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

pub const TOKEN_GRAPHQL_FIELDS: &str = "#
 issuedAt
 accessToken
 accessTokenExpiresIn
 refreshToken
 refreshTokenExpiresIn
 tokenType
 #";
