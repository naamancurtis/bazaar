use async_graphql::{ErrorExtensions, FieldError};
use tracing::error;

pub fn generate_error_log(error: anyhow::Error, message: Option<&str>) {
    let mut error_chain = error.chain().collect::<Vec<_>>();
    if let Some(root_cause) = error_chain.pop() {
        error!(?root_cause, ?error_chain, "{}", message.unwrap_or_default());
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BazaarError {
    #[error("Could not find resource")]
    NotFound,

    #[error("Invalid token provided")]
    InvalidToken(String),

    #[error("Internal Server Error")]
    ServerError(String),

    #[error("Unexpected error occurred")]
    UnexpectedError,
}

impl ErrorExtensions for BazaarError {
    fn extend(&self) -> FieldError {
        self.extend_with(|err, e| match err {
            Self::NotFound => {
                e.set("status", 404);
                e.set("statusText", "NOT_FOUND");
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
