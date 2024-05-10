use actix_web::ResponseError;
use http::StatusCode;
use megamind::{
    models::{ErrorMeta, ErrorResponse, Response},
    ClientError,
};
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("JSON (de)serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Join async task error: {0}")]
    TaskJoin(#[from] JoinError),
    #[error("Server resources exhausted")]
    ResourcesExhausted,
    #[error("Genius client error: {0}")]
    GeniusClient(#[from] ClientError),
    #[error("Genius response error: {meta:?} - {response:?}")]
    GeniusResponse {
        meta: ErrorMeta,
        response: Option<ErrorResponse>,
    },
    #[error("Other genius error: {error} - {error_description}")]
    GeniusOther {
        error: String,
        error_description: String,
    },
}

impl Error {
    pub fn from_genius_response<T>(response: Response<T>) -> Result<T, Self> {
        match response {
            Response::Success { meta: _, response } => Ok(response),
            Response::Error { meta, response } => Err(Self::GeniusResponse { meta, response }),
            Response::Other {
                error,
                error_description,
            } => Err(Self::GeniusOther {
                error,
                error_description,
            }),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ResourcesExhausted => StatusCode::SERVICE_UNAVAILABLE,
            Self::GeniusClient(ClientError::RateLimited) => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
