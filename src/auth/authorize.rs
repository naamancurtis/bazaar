use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;
use uuid::Uuid;

lazy_static! {
    static ref ACCESS_TOKEN_PRIVATE_KEY: String = {
        let key = env::var("ACCESS_TOKEN_PRIVATE_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve access token private key");
            panic!("no access token private key was provided");
        });
        key.unwrap()
    };
    static ref ACCESS_TOKEN_PUBLIC_KEY: String = {
        let key = env::var("ACCESS_TOKEN_PUBLIC_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve access token public key");
            panic!("no access token public key was provided");
        });
        key.unwrap()
    };
    static ref REFRESH_TOKEN_PRIVATE_KEY: String = {
        let key = env::var("REFRESH_TOKEN_PRIVATE_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve refresh token private key");
            panic!("no refresh token private key was provided");
        });
        key.unwrap()
    };
    static ref REFRESH_TOKEN_PUBLIC_KEY: String = {
        let key = env::var("REFRESH_TOKEN_PUBLIC_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve refresh token public key");
            panic!("no refresh token public key was provided");
        });
        key.unwrap()
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum UserType {
    Known,
    Anonymous,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Claims {
    pub sub: Option<Uuid>,
    pub user_type: UserType,
    pub cart_id: Uuid,
    pub exp: usize,
    pub iat: usize,
}

pub fn encode_token(user_id: Option<Uuid>, cart_id: Uuid, token_type: TokenType) -> Result<String> {
    let iat = Utc::now();
    let exp = if token_type == TokenType::Access {
        iat + Duration::minutes(15)
    } else {
        iat + Duration::weeks(4)
    };
    let user_type = if user_id.is_some() {
        UserType::Known
    } else {
        UserType::Anonymous
    };

    let claims = Claims {
        sub: user_id,
        user_type,
        cart_id,
        exp: exp.timestamp() as usize,
        iat: iat.timestamp() as usize,
    };
    encode_jwt(&claims, token_type)
}

fn encode_jwt(claims: &Claims, token_type: TokenType) -> Result<String> {
    let headers = Header::new(Algorithm::PS256);
    let key = if token_type == TokenType::Access {
        ACCESS_TOKEN_PRIVATE_KEY.as_bytes()
    } else {
        REFRESH_TOKEN_PRIVATE_KEY.as_bytes()
    };
    let encoding_key = EncodingKey::from_rsa_pem(key)?;
    encode(&headers, claims, &encoding_key).map_err(|e| {
        error!(err= ?e, "failed to encode json web token");
        anyhow!("Unexpected error occurred while attempting to create jwt")
    })
}

fn decode_jwt(token: &str, token_type: TokenType) -> Result<TokenData<Claims>> {
    let key = if token_type == TokenType::Access {
        ACCESS_TOKEN_PUBLIC_KEY.as_bytes()
    } else {
        REFRESH_TOKEN_PUBLIC_KEY.as_bytes()
    };
    let decoding_key = DecodingKey::from_rsa_pem(key)?;
    let validation = Validation::new(Algorithm::PS256);
    decode(token, &decoding_key, &validation).map_err(|e| {
        error!(err= ?e, "failed to decode json web token");
        // @TODO - Separate out errors and invalid tokens
        anyhow!("invalid token was provided")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_jwt() {
        let iat = Utc::now();
        let exp = iat + Duration::minutes(15);
        let claims = Claims {
            sub: Some(Uuid::new_v4()),
            user_type: UserType::Known,
            cart_id: Uuid::new_v4(),
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
        };
        let token = encode_jwt(&claims, TokenType::Access).unwrap();
        let decoding_key = DecodingKey::from_rsa_pem(ACCESS_TOKEN_PUBLIC_KEY.as_bytes()).unwrap();
        let decoded_token =
            decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::PS256)).unwrap();
        dbg!(&decoded_token.header);
        dbg!(&decoded_token.claims);
        assert_eq!(decoded_token.claims, claims);
    }

    #[test]
    fn test_encode_token() {
        let user_id = None;
        let cart_id = Uuid::new_v4();
        let token = encode_token(user_id, cart_id, TokenType::Refresh).unwrap();
        let decoding_key = DecodingKey::from_rsa_pem(REFRESH_TOKEN_PUBLIC_KEY.as_bytes()).unwrap();
        let decoded_token =
            decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::PS256)).unwrap();
        dbg!(&decoded_token.header);
        dbg!(&decoded_token.claims);
        assert_eq!(decoded_token.claims.sub, user_id);
        assert_eq!(decoded_token.claims.cart_id, cart_id);
        assert_eq!(decoded_token.claims.user_type, UserType::Anonymous);
        let diff = decoded_token.claims.exp - decoded_token.claims.iat;
        let expected_diff = Duration::weeks(4).num_seconds() as usize;
        assert_eq!(diff, expected_diff);
    }
}
