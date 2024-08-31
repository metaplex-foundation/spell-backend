use std::sync::Arc;

use interfaces::asset_storage::AssetMetadataStorage;
use setup::{data_gen::rand_pubkey, TestEnvironment};
use storage::asset_storage_s3::S3Storage;

#[tokio::test]
async fn test_save_fetch() {
    let test_env = TestEnvironment::start().await;

    let s3_client = test_env.metadata_storage_s3_client().await;

    let pubkey = rand_pubkey();
    let initial_json = r#"{ "some": "json" }"#;

    let metadata_storage = S3Storage::new(setup::s3::BUCKET, setup::s3::BUCKET, Arc::new(s3_client)).await;
    metadata_storage.put_json(&pubkey, initial_json).await.unwrap();

    // S3 is fully consistent, so it is safe to read right after write
    let fetched_json = metadata_storage.get_json(&pubkey).await.unwrap();

    assert_eq!(initial_json, fetched_json.unwrap());
}
