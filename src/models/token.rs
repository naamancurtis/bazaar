use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::decode_token, models::CustomerType, BazaarError};

pub type BazaarToken = TokenData<Claims>;

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
}

#[tracing::instrument(skip(s))]
pub fn parse_token(s: String, token_type: TokenType) -> Result<BazaarToken, BazaarError> {
    let mut iter = s.split_whitespace();
    if let Some(prefix) = iter.next() {
        if prefix != "Bearer" {
            return Err(BazaarError::InvalidToken(
                "Invalid token format, expected `Bearer {token}`".to_string(),
            ));
        }
    }
    if let Some(token) = iter.next() {
        return decode_token(&token, token_type);
    }
    Err(BazaarError::InvalidToken("No token was found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{create_valid_jwt_token, set_token_env_vars_for_tests};
    use claim::assert_err;

    #[test]
    fn correctly_parses_valid_token() {
        set_token_env_vars_for_tests();
        let (token, claims) = create_valid_jwt_token(TokenType::Access);
        let jwt = format!("Bearer {}", token);
        let result =
            parse_token(jwt, TokenType::Access).expect("should successfully parse a valid token");
        assert_eq!(claims, result.claims);
    }

    #[test]
    fn rejects_a_malformed_bearer_token() {
        set_token_env_vars_for_tests();
        let (token, _) = create_valid_jwt_token(TokenType::Access);
        let jwt = format!("Berer {}", token);
        let result = parse_token(jwt, TokenType::Access);
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
        let result = parse_token(jwt, TokenType::Access);
        assert_err!(&result);
        let err = result.unwrap_err();

        assert_eq!(
            err,
            BazaarError::InvalidToken("No token was found".to_string())
        );
    }

    #[test]
    fn rejects_random_stuff_for_a_token() {
        set_token_env_vars_for_tests();
        let jwt = format!(
            "Bearer {}",
            "eyaoisdfjhowiej1278431u2hiounc.ey98y1982houhbndlkiusnbaf9.a8932u498jqaj389fgt132"
                .to_string()
        );
        let result = parse_token(jwt, TokenType::Access);
        assert_err!(&result);
        let err = result.unwrap_err();

        assert_eq!(
            err,
            BazaarError::InvalidToken("Token did not match what was expected".to_string())
        );
    }
}
