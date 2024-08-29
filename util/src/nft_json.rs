use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonMetadataError {
    #[error("Metadata contains no URIs")]
    NoUri,
}

/// Validates JSON metadata document
pub fn validate_metadata_contains_uris(
    _json_metadata: &str,
) -> std::result::Result<(), JsonMetadataError> {
    // TODO: not forget to implement
    Ok(())
}
