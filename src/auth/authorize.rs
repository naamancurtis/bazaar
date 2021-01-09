use chrono::Utc;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lazy_static::lazy_static;
use sqlx::PgPool;
use std::env;
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{ACCESS_TOKEN_DURATION, REFRESH_TOKEN_DURATION},
    database::AuthRepository,
    models::{BazaarToken, Claims, CustomerType, TokenType},
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

#[tracing::instrument(skip(token, pool))]
pub async fn verify_and_deserialize_token<R: AuthRepository>(
    token: String,
    token_type: TokenType,
    pool: &PgPool,
) -> Result<BazaarToken, BazaarError> {
    if token.is_empty() {
        return Err(BazaarError::InvalidToken("No token was found".to_owned()));
    }
    let mut token_data = decode_token(&token, token_type)?;
    let id = R::map_id(token_data.claims.sub, pool).await?;
    token_data.claims.id = id;
    Ok(BazaarToken::from(token_data))
}

#[tracing::instrument]
pub fn encode_token(
    user_id: Option<Uuid>,
    cart_id: Uuid,
    token_type: TokenType,
) -> Result<String, BazaarError> {
    let iat = Utc::now();
    let (exp, count) = if let TokenType::Refresh(count) = token_type {
        let exp = iat + *REFRESH_TOKEN_DURATION;
        (exp, Some(count))
    } else {
        let exp = iat + *ACCESS_TOKEN_DURATION;
        (exp, None)
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
        count,
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
    use async_trait::async_trait;
    use chrono::Duration;
    use claim::{assert_err, assert_ok, assert_some};

    use crate::{
        models::auth::AuthCustomer,
        test_helpers::{create_valid_jwt_token, set_token_env_vars_for_tests},
        Result,
    };

    #[test]
    fn test_encode_jwt() {
        set_token_env_vars_for_tests();
        let iat = Utc::now();
        let exp = iat + Duration::minutes(15);
        let claims = Claims {
            sub: Some(Uuid::new_v4()),
            customer_type: CustomerType::Known,
            cart_id: Uuid::new_v4(),
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
            count: None,
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
        set_token_env_vars_for_tests();
        let user_id = None;
        let cart_id = Uuid::new_v4();
        let token = encode_token(user_id, cart_id, TokenType::Refresh(1)).unwrap();
        let decoding_key = DecodingKey::from_rsa_pem(REFRESH_TOKEN_PUBLIC_KEY.as_bytes()).unwrap();
        let decoded_token =
            decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::PS256)).unwrap();
        dbg!(&decoded_token.header);
        dbg!(&decoded_token.claims);
        assert_eq!(decoded_token.claims.sub, user_id);
        assert_eq!(decoded_token.claims.cart_id, cart_id);
        assert_eq!(decoded_token.claims.customer_type, CustomerType::Anonymous);
        assert_eq!(decoded_token.claims.count, Some(1));
        let diff = decoded_token.claims.exp - decoded_token.claims.iat;
        let expected_diff = Duration::weeks(4).num_seconds() as usize;
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn decode_valid_token() {
        set_token_env_vars_for_tests();
        let (token, claims) = create_valid_jwt_token(TokenType::Access);
        let decoded_token = decode_token(&token, TokenType::Access);
        assert_ok!(&decoded_token);
        let decoded_token = decoded_token.unwrap();
        dbg!(&decoded_token.header);
        dbg!(&decoded_token.claims);
        assert_eq!(claims, decoded_token.claims);
    }

    struct MockAuthRepo;

    #[async_trait]
    impl AuthRepository for MockAuthRepo {
        async fn map_id(_: Option<Uuid>, _: &PgPool) -> Result<Option<Uuid>> {
            Ok(Some(Uuid::new_v4()))
        }

        async fn get_auth_customer(_: &str, _: &PgPool) -> Result<AuthCustomer> {
            unimplemented!("Not used for these tests");
        }
    }

    #[tokio::test]
    async fn correctly_parses_valid_token() {
        set_token_env_vars_for_tests();
        let (token, claims) = create_valid_jwt_token(TokenType::Access);
        let config = crate::get_configuration().expect("failed to read config");
        let pool = PgPool::connect_lazy(&config.database.raw_pg_url())
            .expect("failed to create fake connection");
        let result = verify_and_deserialize_token::<MockAuthRepo>(token, TokenType::Access, &pool)
            .await
            .expect("should successfully parse a valid token");
        assert_some!(result.id);
        assert_eq!(claims.iat, result.iat);
        assert_eq!(claims.exp, result.exp);
    }

    #[tokio::test]
    async fn rejects_when_no_token_is_provided() {
        set_token_env_vars_for_tests();
        let token = String::default();
        let config = crate::get_configuration().expect("failed to read config");
        let pool = PgPool::connect_lazy(&config.database.raw_pg_url())
            .expect("failed to create fake connection");
        let result =
            verify_and_deserialize_token::<MockAuthRepo>(token, TokenType::Access, &pool).await;

        assert_err!(&result);
        let err = result.unwrap_err();

        assert_eq!(
            err,
            BazaarError::InvalidToken("No token was found".to_string())
        );
    }

    #[tokio::test]
    async fn rejects_an_invalid_token_token() {
        set_token_env_vars_for_tests();
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string();
        let config = crate::get_configuration().expect("failed to read config");
        let pool = PgPool::connect_lazy(&config.database.raw_pg_url())
            .expect("failed to create fake connection");
        let result =
            verify_and_deserialize_token::<MockAuthRepo>(token, TokenType::Access, &pool).await;
        assert_err!(&result);
        let err = result.unwrap_err();

        assert_eq!(
            err,
            BazaarError::InvalidToken("Token did not match what was expected".to_string())
        );
    }
}
