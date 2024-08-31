#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{body::MessageBody, dev::ServiceResponse, test, web, App};
    use entities::l2::PublicKey;
    use rest_server::infrastructure::{
        endpoints::l2_assets::{
            create_asset, get_asset, update_asset, CreateAssetRequest, L2AssetInfoResponse, UpdateAssetRequest,
        },
        web::app::create_app_state,
    };
    use setup::{TestEnvironment, TestEnvironmentCfg};
    use util::config::Settings;
    use util::{config::HttpServerCfg, publickey::PublicKeyExt};

    #[actix_web::test]
    async fn test_l2_asset_endpoints() {
        // Prepare test env
        let t_env = TestEnvironmentCfg::with_all().start().await;
        let cfg = make_test_cfg(&t_env).await;
        let state = Arc::new(create_app_state(cfg).await);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(create_asset)
                .service(update_asset)
                .service(get_asset),
        )
        .await;

        // given
        let metadata_json = "{}".to_string();
        let owner = PublicKey::new_unique();
        let creator = PublicKey::new_unique();
        let authority = PublicKey::new_unique();

        // create asset
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
                .set_json(req_payload)
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            extract_response(serv_resp)
        };

        assert_eq!(created_asset.name.clone(), "name1".to_string());
        assert_eq!(created_asset.medata_json.clone().unwrap(), metadata_json);
        assert_eq!(created_asset.owner, bs58::encode(owner).into_string());
        assert_eq!(created_asset.creator, bs58::encode(creator).into_string());
        assert_eq!(created_asset.authority, bs58::encode(authority).into_string());
        assert!(created_asset.collection.is_none());

        // get asset
        let fetched_asset_1 = {
            let req = test::TestRequest::get()
                .uri(format!("/asset/{}", created_asset.pubkey).as_str())
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            extract_response(serv_resp)
        };

        assert_eq!(created_asset, fetched_asset_1);

        // new values
        let new_metadata_json = "{ \"imgage\": \"http://some/img.png\"}".to_string();
        let new_owner = PublicKey::new_unique();
        let new_creator = PublicKey::new_unique();
        let new_authority = PublicKey::new_unique();
        let new_collection = PublicKey::new_unique();

        // Update asset
        let updated_asset = {
            let req_payload = UpdateAssetRequest {
                name: Some("name2".to_string()),
                metadata_json: Some(new_metadata_json.clone()),
                owner: Some(bs58::encode(new_owner).into_string()),
                creator: Some(bs58::encode(new_creator).into_string()),
                authority: Some(bs58::encode(new_authority).into_string()),
                collection: Some(Some(bs58::encode(new_collection).into_string())),
            };

            let req = test::TestRequest::put()
                .uri(format!("/asset/{}", created_asset.pubkey).as_str())
                .set_json(req_payload)
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            extract_response(serv_resp)
        };

        assert_eq!(updated_asset.name.clone(), "name2".to_string());
        assert_eq!(updated_asset.medata_json.clone().unwrap(), new_metadata_json);
        assert_eq!(updated_asset.owner, bs58::encode(new_owner).into_string());
        assert_eq!(updated_asset.creator, bs58::encode(new_creator).into_string());
        assert_eq!(updated_asset.authority, bs58::encode(new_authority).into_string());
        assert_eq!(updated_asset.clone().collection.unwrap(), bs58::encode(new_collection).into_string());

        // get asset again
        let fetched_asset_2 = {
            let req = test::TestRequest::get()
                .uri(format!("/asset/{}", created_asset.pubkey).as_str())
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            extract_response(serv_resp)
        };

        assert_eq!(updated_asset, fetched_asset_2);
    }

    async fn make_test_cfg(t_env: &TestEnvironment) -> Settings {
        Settings {
            http_server: HttpServerCfg { port: 0 },
            database: t_env.database_cfg().await,
            obj_storage: t_env.obj_storage_cfg().await,
            env: "it".to_string(),
        }
    }

    fn extract_response(serv_resp: ServiceResponse) -> L2AssetInfoResponse {
        let resp_text = String::from_utf8(serv_resp.into_body().try_into_bytes().unwrap().to_vec()).unwrap();
        serde_json::from_str(&resp_text).unwrap()
    }
}
