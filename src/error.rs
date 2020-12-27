use async_graphql::{ErrorExtensions, FieldError};
use thiserror::Error;
use tracing::error;

pub fn generate_error_log(error: anyhow::Error, message: Option<&str>) {
    let mut error_chain = error.chain().collect::<Vec<_>>();
    if let Some(root_cause) = error_chain.pop() {
        error!(?root_cause, ?error_chain, "{}", message.unwrap_or_default());
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum BazaarError {
    #[error("Could not find resource")]
    NotFound,

    #[error("User is not authorized")]
    Unauthorized,

    #[error("Not authorized to request the specified resource")]
    Forbidden,

    #[error("Invalid token provided")]
    InvalidToken(String),

    #[error("Internal Server Error")]
    ServerError(String),

    #[error("Unexpected error occurred")]
    UnexpectedError,
}

impl ErrorExtensions for BazaarError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|err, e| match self {
            Self::NotFound => {
                e.set("status", 404);
                e.set("statusText", "NOT_FOUND");
            }
            Self::Unauthorized => {
                e.set("status", 401);
                e.set("statusText", "UNAUTHORIZED");
            }
            Self::Forbidden => {
                e.set("status", 403);
                e.set("statusText", "FORBIDDEN");
            }
            Self::InvalidToken(error) => {
                e.set("status", 401);
                e.set("statusText", error.to_string());
            }
            Self::ServerError(error) => {
                e.set("status", 500);
                e.set("statusText", error.to_string());
            }
            Self::UnexpectedError => {
                e.set("status", 500);
            }
        })
    }
}
