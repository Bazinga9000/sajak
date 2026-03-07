// src/main.rs
use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::derive::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum SajakError {
    #[display("An internal error has occurred.")]
    InternalError,

    #[display("The sajak corpus file is either nonexistent or malformed. Please contact your administrator.")]
    BadCorpus,

    #[display("{field} must be positive.")]
    MustBePositive { field: String },

    #[display("Query parse error at {input}.")]
    ParseError { input: String },

    #[display("Computation limit reached.1")]
    Timeout,
}

impl error::ResponseError for SajakError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            SajakError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            SajakError::BadCorpus => StatusCode::INTERNAL_SERVER_ERROR,
            SajakError::MustBePositive { .. } => StatusCode::BAD_REQUEST,
            SajakError::ParseError { .. } => StatusCode::BAD_REQUEST,
            SajakError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}
