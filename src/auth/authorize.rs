use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lazy_static::lazy_static;
use std::env;
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{ACCESS_TOKEN_DURATION, REFRESH_TOKEN_DURATION},
    models::{Claims, CustomerType, TokenType},
    BazaarError,
};

// @TODO - check these are actually okay being `lazy_static` - if the server
// is left up and running for a long time, but we wanted to cycle keys every x
// days, would this pick up on the changes? or would it store a constant value
// for the whole period of time the server is up
lazy_static! {
    static ref ACCESS_TOKEN_PRIVATE_KEY: String = {
        let key = env::var("ACCESS_TOKEN_PRIVATE_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve access token private key");
            panic!("no access token private key was provided");
        });
        key.expect("[ENV VAR] ACCESS_TOKEN_PRIVATE_KEY failed")
    };
    static ref ACCESS_TOKEN_PUBLIC_KEY: String = {
        let key = env::var("ACCESS_TOKEN_PUBLIC_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve access token public key");
            panic!("no access token public key was provided");
        });
        key.expect("[ENV VAR] ACCESS_TOKEN_PUBLIC_KEY failed")
    };
    static ref REFRESH_TOKEN_PRIVATE_KEY: String = {
        let key = env::var("REFRESH_TOKEN_PRIVATE_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve refresh token private key");
            panic!("no refresh token private key was provided");
        });
        key.expect("[ENV VAR] REFRESH_TOKEN_PRIVATE_KEY failed")
    };
    static ref REFRESH_TOKEN_PUBLIC_KEY: String = {
        let key = env::var("REFRESH_TOKEN_PUBLIC_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve refresh token public key");
            panic!("no refresh token public key was provided");
        });
        key.expect("[ENV VAR] REFRESH_TOKEN_PUBLIC_KEY failed")
    };
}

#[tracing::instrument]
pub fn encode_token(
    user_id: Option<Uuid>,
    cart_id: Uuid,
    token_type: TokenType,
) -> Result<String, BazaarError> {
    let iat = Utc::now();
    let exp = if token_type == TokenType::Access {
        iat + Duration::seconds(ACCESS_TOKEN_DURATION)
    } else {
        iat + Duration::seconds(REFRESH_TOKEN_DURATION)
    };
    let customer_type = if user_id.is_some() {
        CustomerType::Known
    } else {
        CustomerType::Anonymous
    };

    let claims = Claims {
        sub: user_id,
        customer_type,
        cart_id,
        exp: exp.timestamp() as usize,
        iat: iat.timestamp() as usize,
        id: None,
        token_type,
    };
    encode_jwt(&claims, token_type)
}

#[tracing::instrument]
pub(crate) fn encode_jwt(claims: &Claims, token_type: TokenType) -> Result<String, BazaarError> {
    let headers = Header::new(Algorithm::PS256);
    let key = if token_type == TokenType::Access {
        ACCESS_TOKEN_PRIVATE_KEY.as_bytes()
    } else {
        REFRESH_TOKEN_PRIVATE_KEY.as_bytes()
    };
    let encoding_key = EncodingKey::from_rsa_pem(key).map_err(|e| {
        error!(err = ?e, "failed to parse the jwt encoding key");
        BazaarError::UnexpectedError
    })?;
    encode(&headers, claims, &encoding_key).map_err(|e| {
        error!(err= ?e, "failed to encode json web token");
        BazaarError::UnexpectedError
    })
}

#[tracing::instrument(skip(token))]
pub fn decode_token(token: &str, token_type: TokenType) -> Result<TokenData<Claims>, BazaarError> {
    let key = if token_type == TokenType::Access {
        ACCESS_TOKEN_PUBLIC_KEY.as_bytes()
    } else {
        REFRESH_TOKEN_PUBLIC_KEY.as_bytes()
    };
    let decoding_key = DecodingKey::from_rsa_pem(key).map_err(|e| {
        error!(err= ?e, "failed to retrieve the decoding key");
        BazaarError::UnexpectedError
    })?;
    let validation = Validation::new(Algorithm::PS256);
    decode(token, &decoding_key, &validation).map_err(|e| {
        error!(err= ?e, "failed to decode json web token");
        // @TODO - Separate out errors and invalid tokens
        BazaarError::InvalidToken("Token did not match what was expected".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_valid_jwt_token;
    use claim::assert_ok;

    #[test]
    fn test_encode_jwt() {
        let iat = Utc::now();
        let exp = iat + Duration::minutes(15);
        let claims = Claims {
            sub: Some(Uuid::new_v4()),
            customer_type: CustomerType::Known,
            cart_id: Uuid::new_v4(),
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
            id: None,
            token_type: TokenType::Access,
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
        assert_eq!(decoded_token.claims.customer_type, CustomerType::Anonymous);
        let diff = decoded_token.claims.exp - decoded_token.claims.iat;
        let expected_diff = Duration::weeks(4).num_seconds() as usize;
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn decode_valid_token() {
        let (token, claims) = create_valid_jwt_token(TokenType::Access);
        let decoded_token = decode_token(&token, TokenType::Access);
        assert_ok!(&decoded_token);
        let decoded_token = decoded_token.unwrap();
        dbg!(&decoded_token.header);
        dbg!(&decoded_token.claims);
        assert_eq!(claims, decoded_token.claims);
    }
}
