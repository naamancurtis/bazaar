use actix_web::{error::ResponseError, HttpResponse};
use async_graphql::ErrorExtensions;
use serde::Serialize;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error, PartialEq)]
pub enum BazaarError {
    #[error("Could not find resource")]
    NotFound,

    #[error("User is not authorized")]
    Unauthorized,

    #[error("Not authorized to request the specified resource")]
    Forbidden,

    #[error("Incorrect credentials provided")]
    IncorrectCredentials,

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Invalid token provided")]
    InvalidToken(String),

    #[error("A server error occurred")]
    DatabaseError,

    #[error("Internal Server Error")]
    ServerError(String),

    #[error("Unexpected error occurred")]
    UnexpectedError,

    #[error("Provided data was malformed")]
    MalformedData,

    #[error("Unexpected error occurred")]
    CryptoError(#[from] argon2::Error),
}

impl ErrorExtensions for BazaarError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|err, e| {
            warn!(?err, ?e, "from errors.rs looking at async");
            match self {
                Self::BadRequest(error) => {
                    e.set("status", 400);
                    e.set("statusText", "BAD_REQUEST");
                    e.set("details", error.to_string());
                }
                Self::Unauthorized | Self::IncorrectCredentials => {
                    e.set("status", 401);
                    e.set("statusText", "UNAUTHORIZED");
                }
                Self::InvalidToken(error) => {
                    e.set("status", 401);
                    e.set("statusText", "INVALID_TOKEN");
                    e.set("details", error.to_string());
                }
                Self::Forbidden => {
                    e.set("status", 403);
                    e.set("statusText", "FORBIDDEN");
                }
                Self::NotFound => {
                    e.set("status", 404);
                    e.set("statusText", "NOT_FOUND");
                }
                Self::ServerError(error) => {
                    e.set("status", 500);
                    e.set("statusText", "SERVER_ERROR");
                    e.set("context", error.to_string());
                }
                Self::UnexpectedError => {
                    e.set("status", 500);
                    e.set("statusText", "SERVER_ERROR");
                }
                _ => {}
            }
        })
    }
}

#[derive(Debug, Serialize)]
struct Messages(Vec<String>);

impl From<Vec<&String>> for Messages {
    fn from(s: Vec<&String>) -> Self {
        Self(s.iter().map(|s| s.to_string()).collect::<Vec<String>>())
    }
}

impl ResponseError for BazaarError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::NotFound => HttpResponse::NotFound().finish(),
            Self::Unauthorized | Self::IncorrectCredentials => {
                HttpResponse::Unauthorized().finish()
            }
            Self::Forbidden => HttpResponse::Forbidden().finish(),
            Self::InvalidToken(error) => {
                HttpResponse::Unauthorized().json::<Messages>(vec![error].into())
            }
            Self::ServerError(error) => {
                HttpResponse::InternalServerError().json::<Messages>(vec![error].into())
            }
            Self::UnexpectedError => HttpResponse::InternalServerError().finish(),
            // Catch all, as most of the time we should be using GraphQL errors
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl From<sqlx::Error> for BazaarError {
    fn from(e: sqlx::Error) -> BazaarError {
        use sqlx::Error::*;

        match e {
            RowNotFound => BazaarError::NotFound,
            _ => {
                error!(err = ?e, "SQLx error occurred");
                BazaarError::DatabaseError
            }
        }
    }
}

impl From<serde_json::Error> for BazaarError {
    fn from(e: serde_json::Error) -> BazaarError {
        use serde_json::error::Category::*;
        error!(err = ?e, "JSON Serde error occurred");

        match e.classify() {
            Syntax | Data => BazaarError::MalformedData,
            _ => BazaarError::UnexpectedError,
        }
    }
}

impl From<tokio::task::JoinError> for BazaarError {
    fn from(e: tokio::task::JoinError) -> BazaarError {
        error!(
            err = ?e,
            was_cancelled = e.is_cancelled(),
            did_panic = e.is_panic(),
            "Tokio task join error occurred"
        );
        BazaarError::UnexpectedError
    }
}
