mod test_app_util;

#[cfg(not(any(skip_solana_tests)))]
#[allow(clippy::all)]
mod test {
    use crate::test_app_util;
    use mpl_core::instructions::CreateV1Builder;
    use reqwest::Client as ReqWestClient;
    use reqwest::StatusCode;
    use rest_server::rest::endpoints::l2_assets::CreateAssetRequest;
    use setup::TestEnvironmentCfg;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
    use std::{str::FromStr, time::Duration};
    use test_app_util::{extract_asset_from_reqwest_response, extract_string_from_reqwest_response};
    use util::str_util::form_url;

    #[actix_web::test]
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

        test_app_util::init_app_as_web_server(&t_env).await;

        let solana_client = RpcClient::new_with_timeout(t_env.solana_url(), Duration::from_secs(1));
        let reqwest_client = ReqWestClient::new();
        await_solana_to_startup(&solana_client).await;

        let client_kp = Keypair::new();
        let master_kp = solana_sdk::signature::keypair_from_seed(&test_cfg.master_key_seed()).unwrap();
        airdop(&solana_client, &[client_kp.pubkey(), master_kp.pubkey()]).await;

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

        let creator_kp = Keypair::new();
        let authority_kp = Keypair::new();

        let created_asset = {
            let req_payload = CreateAssetRequest {
                name: "name1".to_string(),
                metadata_json: metadata_json.clone(),
                owner: bs58::encode(client_kp.pubkey()).into_string(),
                creator: bs58::encode(creator_kp.pubkey()).into_string(),
                authority: bs58::encode(authority_kp.pubkey()).into_string(),
                royalty_basis_points: 500,
                collection: None,
            };

            let url = form_url(&test_cfg.rest_server.base_url, test_cfg.rest_server.port, "asset");
            let serv_resp = reqwest_client
                .post(url)
                .header("x-api-key", "111")
                .json(&req_payload)
                .send()
                .await
                .unwrap_or_else(|e| panic!("Failed to sent request: {e}"));
            assert!(serv_resp.status().eq(&StatusCode::CREATED));
            extract_asset_from_reqwest_response(serv_resp).await
        };

        // Transaction created on client side
        let asset_name = created_asset
            .content
            .as_ref()
            .unwrap()
            .metadata
            .get_item("name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let authority = Pubkey::from_str(&created_asset.authorities.as_ref().unwrap()[0].address).unwrap();

        let create_asset_ix = CreateV1Builder::new()
            .asset(Pubkey::from_str(&created_asset.id).unwrap())
            .payer(client_kp.pubkey())
            .name(asset_name)
            .uri(format!("{}/asset/{}/metadata.json", test_cfg.rest_server.base_url, &created_asset.id))
            .authority(Some(authority))
            .owner(Some(client_kp.pubkey()))
            .instruction();

        let signers = vec![&client_kp, &authority_kp];

        let last_blockhash = solana_client.get_latest_blockhash().await.unwrap();

        // Transaction is partially signed by client
        let mut create_asset_tx = Transaction::new_with_payer(&[create_asset_ix], Some(&client_kp.pubkey()));
        create_asset_tx.partial_sign(&signers, last_blockhash);

        let url = form_url(&test_cfg.rest_server.base_url, test_cfg.rest_server.port, "asset/mint");
        let serv_resp = reqwest_client
            .post(url)
            .header("x-api-key", "111")
            .json(&create_asset_tx)
            .send()
            .await
            .unwrap_or_else(|e| panic!("Failed to sent request: {e}"));
        assert!(serv_resp.status().eq(&StatusCode::OK));
        let resp_text = extract_string_from_reqwest_response(serv_resp).await;
        println!("RESP: {resp_text}");
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

    async fn await_solana_to_startup(solana_client: &RpcClient) {
        await_for(10, Duration::from_secs(1), || solana_client.get_health())
            .await
            .unwrap();
    }

    async fn airdop(solana_client: &RpcClient, pubkeys: &[Pubkey]) {
        let mut signatures = Vec::new();

        for pubkey in pubkeys {
            let signature = solana_client.request_airdrop(&pubkey, 20000000 * 10000).await.unwrap();
            signatures.push(signature);
        }

        for signature in signatures {
            while !(solana_client.confirm_transaction(&signature).await.unwrap()) {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
