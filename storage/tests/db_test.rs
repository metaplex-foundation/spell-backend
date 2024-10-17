use chrono::NaiveDateTime;
use entities::l2::L2Asset;
use interfaces::l2_storage::Bip44DerivationSequence;
use interfaces::l2_storage::DerivationValues;
use interfaces::l2_storage::L2Storage;
use setup::data_gen::rand_pubkey_str;
use setup::{data_gen::rand_pubkey, TestEnvironment};
use storage::l2_storage_pg::L2StoragePg;

#[tokio::test]
async fn test_save_fetch() {
    let test_env = TestEnvironment::builder().with_pg().start().await;

    let url = test_env.l2_storage_pg_url().await;

    let storage = L2StoragePg::new_from_url(&url, 1, 1).await.unwrap();

    let asset = L2Asset {
        pubkey: rand_pubkey(),
        name: "name".to_string(),
        owner: rand_pubkey_str(),
        creator: rand_pubkey_str(),
        collection: None,
        authority: rand_pubkey_str(),
        royalty_basis_points: 0,
        create_timestamp: NaiveDateTime::default(),
        update_timestamp: NaiveDateTime::default(),
        bip44_account_num: 1,
        bip44_address_num: 1,
    };

    storage.save(&asset).await.unwrap();

    let fetched = storage.find(&asset.pubkey).await.unwrap();

    assert_eq!(fetched.unwrap(), asset);
}

#[tokio::test]
async fn test_bip44_sequences() {
    let test_env = TestEnvironment::builder().with_pg().start().await;

    let url = test_env.l2_storage_pg_url().await;

    let sut: &dyn Bip44DerivationSequence = &L2StoragePg::new_from_url(&url, 1, 1).await.unwrap();

    assert_eq!(sut.next_account_and_address().await.unwrap(), DerivationValues { account: 0, address: 1 });
    assert_eq!(sut.next_account_and_address().await.unwrap(), DerivationValues { account: 0, address: 2 });
    assert_eq!(sut.next_account_and_address().await.unwrap(), DerivationValues { account: 0, address: 3 });
}
