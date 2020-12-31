use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::Deref;
use tracing::warn;
use uuid::Uuid;

use crate::{auth::decode_token, database::AuthRepository, models::CustomerType, BazaarError};

/// This token is intentionally immutable and unconstructable unless you have
/// the raw `TokenData`. This is because the public ID should not really be
/// leaked into the business layer of the application.
///
/// At the point where this token is constructed (once the token has been parsed
/// from the request) this sanitised token should be the thing carried through
/// the application and used.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BazaarToken {
    pub id: Option<Uuid>,
    pub iat: usize,
    pub exp: usize,
    pub customer_type: CustomerType,
    pub cart_id: Uuid,
    pub token_type: TokenType,
    sub: Option<Uuid>,
    /// This is to ensure this token isn't constructable outside of this module
    /// ie. the only viable way to construct a token is with `Trait: From<TokenData<Claims>>`
    _marker: PhantomData<()>,
}

impl From<TokenData<Claims>> for BazaarToken {
    fn from(t: TokenData<Claims>) -> Self {
        let claims = t.claims;
        if claims.customer_type == CustomerType::Known && claims.id.is_none() {
            warn!(
                pub_id = ?claims.sub,
                cart_id = ?claims.cart_id,
                customer_type = ?claims.customer_type,
                "expected to find private ID but nothing was found"
            );
        }
        Self {
            id: claims.id,
            iat: claims.iat,
            exp: claims.exp,
            customer_type: claims.customer_type,
            cart_id: claims.cart_id,
            token_type: claims.token_type,
            sub: claims.sub,
            _marker: PhantomData,
        }
    }
}

impl BazaarToken {
    pub fn time_till_expiry(&self) -> Duration {
        self.expires_at() - Utc::now()
    }

    pub fn issued_at(&self) -> DateTime<Utc> {
        utc_from_timestamp(self.iat)
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        utc_from_timestamp(self.exp)
    }

    /// This method should only be called in the GraphQL Resolver in order to ensure
    /// that the private ID is not leaked out publically (ie. to overwrite it)
    ///
    /// The public ID shouldn't be used anywhere else within the application
    pub fn public_id(&self) -> Uuid {
        if let Some(id) = self.sub {
            return id;
        }
        warn!(public_id = ?self.sub, id = ?self.id, "expected to have valid ID mappings");
        Uuid::nil()
    }
}

fn utc_from_timestamp(timestamp: usize) -> DateTime<Utc> {
    let duration = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    DateTime::from_utc(duration, Utc)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Claims {
    pub sub: Option<Uuid>,
    pub customer_type: CustomerType,
    pub cart_id: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub token_type: TokenType,
    #[serde(skip)]
    pub id: Option<Uuid>,
}

/// At the point in time where we have a `BearerToken` we aren't guarenteed that
/// the token is valid. _(for that to happen we need to use the
/// `parse_and_deserialize_token` function). However we are guarenteed that we
/// were sent a token by the user, and it followed the specified format ie:
/// `Bearer {token}`
///
/// The String contained within the unit struct is just the token, the `Bearer `
/// prefix has been stripped from it (see `TryFrom` impl for details)
#[derive(Debug, Clone)]
pub struct BearerToken(String);

impl Deref for BearerToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for BearerToken {
    type Error = BazaarError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut iter = s.split_whitespace();
        if let Some(prefix) = iter.next() {
            if prefix != "Bearer" {
                return Err(BazaarError::InvalidToken(
                    "Invalid token format, expected `Bearer {token}`".to_string(),
                ));
            }
        }
        if let Some(token) = iter.next() {
            return Ok(Self(token.to_owned()));
        }
        Err(BazaarError::InvalidToken("No token was found".to_string()))
    }
}

#[tracing::instrument(skip(token))]
pub async fn parse_and_deserialize_token<R: AuthRepository>(
    token: BearerToken,
    token_type: TokenType,
    pool: &PgPool,
) -> Result<BazaarToken, BazaarError> {
    let mut token_data = decode_token(&token, token_type)?;
    let id = R::map_id(token_data.claims.sub, pool).await;
    token_data.claims.id = id;
    Ok(BazaarToken::from(token_data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use claim::{assert_err, assert_some};

    use crate::{
        models::auth::AuthCustomer,
        test_helpers::{create_valid_jwt_token, set_token_env_vars_for_tests},
    };

    struct MockAuthRepo;

    #[async_trait]
    impl AuthRepository for MockAuthRepo {
        async fn map_id(_: Option<Uuid>, _: &PgPool) -> Option<Uuid> {
            Some(Uuid::new_v4())
        }

        async fn get_auth_customer(_: &str, _: &PgPool) -> Result<AuthCustomer, BazaarError> {
            unimplemented!("Not used for these tests");
        }
    }

    #[tokio::test]
    async fn correctly_parses_valid_token() {
        set_token_env_vars_for_tests();
        let (token, claims) = create_valid_jwt_token(TokenType::Access);
        let jwt = format!("Bearer {}", token);
        let jwt = BearerToken::try_from(jwt).expect("should provide a valid token");
        let config = crate::get_configuration().expect("failed to read config");
        let pool = PgPool::connect_lazy(&config.database.raw_pg_url())
            .expect("failed to create fake connection");
        let result = parse_and_deserialize_token::<MockAuthRepo>(jwt, TokenType::Access, &pool)
            .await
            .expect("should successfully parse a valid token");
        assert_some!(result.id);
        assert_eq!(claims.iat, result.iat);
        assert_eq!(claims.exp, result.exp);
    }

    #[test]
    fn rejects_a_malformed_bearer_token() {
        set_token_env_vars_for_tests();
        let (token, _) = create_valid_jwt_token(TokenType::Access);
        let jwt = format!("Berer {}", token);
        let result = BearerToken::try_from(jwt);
        assert_err!(&result);
        let err = result.unwrap_err();

        assert_eq!(
            err,
            BazaarError::InvalidToken(
                "Invalid token format, expected `Bearer {token}`".to_string()
            )
        );
    }

    #[test]
    fn rejects_when_no_token_is_provided() {
        set_token_env_vars_for_tests();
        let jwt = format!("Bearer {}", "".to_string());
        let result = BearerToken::try_from(jwt);
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
        let jwt = format!(
            "Bearer {}",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string()
        );
        let token = BearerToken::try_from(jwt).expect("should give valid token");
        let config = crate::get_configuration().expect("failed to read config");
        let pool = PgPool::connect_lazy(&config.database.raw_pg_url())
            .expect("failed to create fake connection");
        let result =
            parse_and_deserialize_token::<MockAuthRepo>(token, TokenType::Access, &pool).await;
        assert_err!(&result);
        let err = result.unwrap_err();

        assert_eq!(
            err,
            BazaarError::InvalidToken("Token did not match what was expected".to_string())
        );
    }
}
