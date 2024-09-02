use crate::endpoints::types::JsonRpcError;
use jsonrpc_core::ErrorCode;
use thiserror::Error;

const STANDARD_ERROR_CODE: i64 = -32000;

#[derive(Error, Debug)]
pub enum DasApiError {
    #[error("No data found.")]
    NoDataFoundError,
    #[error("Pubkey Validation Err: {0} is invalid")]
    PubkeyValidationError(String),
    #[error("Database Error")]
    DatabaseError,
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
        }
    }
}
