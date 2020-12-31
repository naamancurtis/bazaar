use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    auth::authorize::encode_jwt,
    models::{Claims, CustomerType, TokenType},
};

pub fn create_valid_jwt_token(token_type: TokenType) -> (String, Claims) {
    let iat = Utc::now();
    let exp = iat + Duration::minutes(15);
    let claims = Claims {
        sub: Some(Uuid::new_v4()),
        customer_type: CustomerType::Known,
        cart_id: Uuid::new_v4(),
        exp: exp.timestamp() as usize,
        iat: iat.timestamp() as usize,
        id: None,
        token_type,
    };
    let token = encode_jwt(&claims, token_type).unwrap();
    (token, claims)
}
