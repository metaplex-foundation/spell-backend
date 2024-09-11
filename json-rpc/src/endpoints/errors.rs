use crate::endpoints::types::JsonRpcError;
use jsonrpc_core::ErrorCode;
use thiserror::Error;

const STANDARD_ERROR_CODE: i64 = -32000;

#[derive(Error, Debug, PartialOrd, PartialEq)]
pub enum DasApiError {
    #[error("No data found.")]
    NoDataFoundError,
    #[error("Pubkey Validation Err: {0} is invalid")]
    PubkeyValidationError(String),
    #[error("Database Error")]
    DatabaseError,
    #[error("Failed to parse Json metadata.")]
    JsonMetadataParsing,
    #[error("Requested limit number is too big. Up to '{0}' limit is supported.")]
    LimitTooBig(u32),
    #[error("Page number is too big. Up to '{0}' pages are supported with this kind of pagination. Please use a different pagination(before/after/cursor).")]
    PageTooBig(u32),
}

impl From<DasApiError> for JsonRpcError {
    fn from(value: DasApiError) -> Self {
        match value {
            DasApiError::NoDataFoundError => Self {
                code: ErrorCode::ServerError(STANDARD_ERROR_CODE),
                message: "Database Error: RecordNotFound Error: Asset Not Found".to_string(),
                data: None,
            },
            DasApiError::PubkeyValidationError(key) => Self {
                code: ErrorCode::ServerError(STANDARD_ERROR_CODE),
                message: format!("Pubkey Validation Error: {key} is invalid"),
                data: None,
            },
            DasApiError::DatabaseError => Self::internal_error(),
            DasApiError::JsonMetadataParsing => Self {
                code: ErrorCode::ParseError,
                message: "Failed to parse Json metadata.".to_string(),
                data: None,
            },
            DasApiError::LimitTooBig(max_limit) => Self {
                code: ErrorCode::ServerError(STANDARD_ERROR_CODE),
                message: format!("Requested limit number is too big. Up to '{max_limit}' limit is supported."),
                data: None,
            },
            DasApiError::PageTooBig(max_limit) => Self {
                code: ErrorCode::ServerError(STANDARD_ERROR_CODE),
                message: format!(
                    "
                    Page number is too big. Up to '{max_limit}' pages are supported with this kind of pagination.
                    Please use a different pagination(before/after/cursor).
                "
                ),
                data: None,
            },
        }
    }
}
