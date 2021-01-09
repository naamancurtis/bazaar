use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use tracing::warn;
use uuid::Uuid;

use crate::models::CustomerType;

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
    pub count: Option<i32>,
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
                count = ?claims.count,
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
            count: claims.count,
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

pub(crate) fn utc_from_timestamp(timestamp: usize) -> DateTime<Utc> {
    let duration = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    DateTime::from_utc(duration, Utc)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Access,
    Refresh(i32),
}

impl TokenType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Access => "ACCESS",
            Self::Refresh(_) => "REFRESH",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Claims {
    pub sub: Option<Uuid>,
    pub customer_type: CustomerType,
    pub cart_id: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    #[serde(skip)]
    pub id: Option<Uuid>,
}
