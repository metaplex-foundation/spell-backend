
use chrono::NaiveDateTime;
use setup::TestEnvironment;
use storage::l2_storage::{L2Asset, L2Storage};
use storage_pg::l2_storage_pg::L2StoragePg;

#[tokio::test]
async fn test_save_fetch() {
    let test_env = TestEnvironment::start().await;

    let url = test_env.pg_url().await;

    let storage = L2StoragePg::new_from_url(&url, 1, 1)
        .await.unwrap();

    let asset = L2Asset {
        pubkey: Some([1u8; 32]),
        name: "name".to_string(),
        owner: [1u8; 32],
        creator: [1u8; 32],
        collection: None,
        authority: [1u8; 32],
        metadata_url: "url".to_string(),
        create_timestamp: NaiveDateTime::default()
    };

    storage.save(&asset).await.unwrap();

    let fetched = storage.find(&[1u8; 32]).await.unwrap();

    assert_eq!(fetched.unwrap(), asset);
}