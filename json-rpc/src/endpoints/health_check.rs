use crate::endpoints::types::JsonRpcResponse;
use serde_json::json;

pub async fn health() -> JsonRpcResponse {
    Ok(json!("Server is ok"))
}
