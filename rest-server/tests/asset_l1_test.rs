mod test_app_util;

#[cfg(not(any(skip_solana_tests)))]
#[allow(clippy::all)]
mod test {
    use std::{str::FromStr, time::Duration};

    use actix_http::StatusCode;
    use actix_web::test;
    use entities::l2::PublicKey;
    use mpl_core::instructions::CreateV1Builder;
    use publickey::PublicKeyExt;
    use rest_server::endpoints::l2_assets::CreateAssetRequest;
    use setup::TestEnvironmentCfg;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
    use test_app_util::extract_asset_from_response;
    use util::publickey;

    use crate::test_app_util;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_l1() {
        // Prepare environment
        let t_env = TestEnvironmentCfg::default()
            .with_solana()
            .with_pg()
            .with_s3()
            .start()
            .await;
        let test_cfg = t_env.make_test_cfg().await;

        let app = test_app_util::init_web_app(&t_env).await;

        let solana_client = RpcClient::new_with_timeout(t_env.solana_url(), Duration::from_secs(1));

        // Waiting for server to start
        await_for(10, Duration::from_secs(1), || solana_client.get_health())
            .await
            .unwrap();

        let client_keypair = Keypair::new();
        {
            let master_keypair = solana_sdk::signature::keypair_from_seed(&test_cfg.master_key_seed()).unwrap();
            let airdrop_sig_1 = solana_client
                .request_airdrop(&master_keypair.pubkey(), 20000000 * 10000)
                .await
                .unwrap();
            let airdrop_sig_2 = solana_client
                .request_airdrop(&client_keypair.pubkey(), 20000000 * 10000)
                .await
                .unwrap();

            while !(solana_client.confirm_transaction(&airdrop_sig_1).await.unwrap()
                && solana_client.confirm_transaction(&airdrop_sig_2).await.unwrap())
            {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        // preparing L2 asset
        let metadata_json = r#"
                {
                    "name": "name1",
                    "description": "test description",
                    "image": "http://host/image.png",
                    "properties": {
                        "files": [
                            {
                                "uri": "http://host/image.png",
                                "type": "image/png"
                            }
                        ],
                        "category": "image"
                    }
                }
            "#
        .to_string();
        let owner = PublicKey::new_unique();
        let creator = PublicKey::new_unique();
        let authority = PublicKey::new_unique();

        let created_asset = {
            let req_payload = CreateAssetRequest {
                name: "name1".to_string(),
                metadata_json: metadata_json.clone(),
                owner: bs58::encode(owner).into_string(),
                creator: bs58::encode(creator).into_string(),
                authority: bs58::encode(authority).into_string(),
                collection: None,
            };

            let req = test::TestRequest::post()
                .uri("/asset")
                .append_header(("x-api-key", "111"))
                .set_json(req_payload)
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            assert_eq!(serv_resp.status(), StatusCode::CREATED);
            extract_asset_from_response(serv_resp)
        };

        // Transaction created on client side
        let create_asset_ix = CreateV1Builder::new()
            .asset(Pubkey::from_str(&created_asset.id).unwrap())
            .payer(client_keypair.pubkey())
            .name(
                created_asset
                    .content
                    .as_ref()
                    .unwrap()
                    .metadata
                    .get_item("name")
                    .unwrap()
                    .to_string(),
            )
            // TODO: replace with util call
            .uri(format!("https://l2-backend/asset/{}/metadata.json", &created_asset.id))
            .instruction();

        let signers = vec![&client_keypair];

        let last_blockhash = solana_client.get_latest_blockhash().await.unwrap();

        // Transaction is partially signed by client
        let mut create_asset_tx = Transaction::new_with_payer(&[create_asset_ix], Some(&client_keypair.pubkey()));
        create_asset_tx.partial_sign(&signers, last_blockhash);

        // Calling L1 mint endpoint
        let req = test::TestRequest::post()
            .uri("/asset/mint")
            .append_header(("x-api-key", "111"))
            .set_json(create_asset_tx)
            .to_request();

        let serv_resp = test::call_service(&app, req).await;
        assert_eq!(serv_resp.status(), StatusCode::OK);
    }

    /// Helps to wait for an async functionality to startup.
    async fn await_for<T, E, F, Fut>(attempts: u32, interval: Duration, f: F) -> std::result::Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
    {
        for _attempts in 1..attempts {
            let r = f().await;
            if r.is_ok() {
                return r;
            }
            tokio::time::sleep(interval).await;
        }
        f().await
    }
}
