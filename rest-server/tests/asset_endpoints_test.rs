#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    use actix_web::{body::MessageBody, dev::ServiceResponse, http::StatusCode, test, web, App};
    use entities::{l2::PublicKey, rpc_asset_models::Asset};
    use rest_server::{
        endpoints::l2_assets::{
            create_asset, get_asset, get_metadata, update_asset, CreateAssetRequest, UpdateAssetRequest,
        },
        web::app::create_app_state,
    };
    use setup::{TestEnvironment, TestEnvironmentCfg};
    use util::config::{EnvProfile, JsonRpc, Settings};
    use util::{config::RestServerCfg, publickey::PublicKeyExt};

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
                .service(get_asset)
                .service(get_metadata),
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
            assert_eq!(serv_resp.status(), StatusCode::CREATED);
            extract_response(serv_resp)
        };

        assert_eq!(
            created_asset
                .content
                .as_ref()
                .unwrap()
                .metadata
                .get_item("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "name1"
        );
        assert_eq!(created_asset.ownership.owner, bs58::encode(owner).into_string());
        assert_eq!(created_asset.creators.as_ref().unwrap()[0].address, bs58::encode(creator).into_string());
        assert_eq!(created_asset.authorities.as_ref().unwrap()[0].address, bs58::encode(authority).into_string());
        assert!(created_asset.grouping.is_none());

        // get asset
        let fetched_asset_1 = {
            let req = test::TestRequest::get()
                .uri(format!("/asset/{}", created_asset.id).as_str())
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            assert_eq!(serv_resp.status(), StatusCode::OK);
            extract_response(serv_resp)
        };

        assert_eq!(created_asset, fetched_asset_1);

        // new values
        let new_metadata_json = r#"
            {
                "name": "name2",
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
                .uri(format!("/asset/{}", created_asset.id).as_str())
                .set_json(req_payload)
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            assert_eq!(serv_resp.status(), StatusCode::OK);
            extract_response(serv_resp)
        };

        assert_eq!(
            updated_asset
                .content
                .as_ref()
                .unwrap()
                .metadata
                .get_item("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "name2"
        );
        assert_eq!(updated_asset.ownership.owner, bs58::encode(new_owner).into_string());
        assert_eq!(updated_asset.creators.as_ref().unwrap()[0].address, bs58::encode(new_creator).into_string());
        assert_eq!(updated_asset.authorities.as_ref().unwrap()[0].address, bs58::encode(new_authority).into_string());
        assert_eq!(
            updated_asset.grouping.as_ref().unwrap()[0].clone().group_value.unwrap(),
            bs58::encode(new_collection).into_string()
        );
        assert_eq!(
            updated_asset
                .content
                .as_ref()
                .unwrap()
                .metadata
                .get_item("description")
                .unwrap(),
            "test description"
        );
        assert_eq!(
            updated_asset
                .content
                .as_ref()
                .unwrap()
                .links
                .as_ref()
                .unwrap()
                .get("image")
                .unwrap(),
            "http://host/image.png"
        );
        assert_eq!(
            updated_asset.content.as_ref().unwrap().files.as_ref().unwrap()[0]
                .uri
                .as_ref()
                .unwrap(),
            "http://host/image.png"
        );

        // get asset again
        let fetched_asset_2 = {
            let req = test::TestRequest::get()
                .uri(format!("/asset/{}", created_asset.id).as_str())
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            assert_eq!(serv_resp.status(), StatusCode::OK);
            extract_response(serv_resp)
        };

        assert_eq!(updated_asset, fetched_asset_2);

        // get metadata json separately
        let fetched_metadata = {
            let req = test::TestRequest::get()
                .uri(format!("/asset/{}/metadata.json", created_asset.id).as_str())
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            assert_eq!(serv_resp.status(), StatusCode::OK);
            String::from_utf8(serv_resp.into_body().try_into_bytes().unwrap().to_vec()).unwrap()
        };

        assert_eq!(fetched_metadata, new_metadata_json);
    }

    async fn make_test_cfg(t_env: &TestEnvironment) -> Settings {
        Settings {
            rest_server: RestServerCfg {
                port: 8080,
                host: Ipv4Addr::LOCALHOST,
                log_level: "DEBUG".to_string(),
                base_url: "http://localhost".to_string(),
            },
            database: t_env.database_cfg().await,
            obj_storage: t_env.obj_storage_cfg().await,
            env: EnvProfile::Local,
            json_rpc_server: JsonRpc { port: 8081, host: Ipv4Addr::LOCALHOST, log_level: "DEBUG".to_string() },
        }
    }

    fn extract_response(serv_resp: ServiceResponse) -> Asset {
        let resp_text = String::from_utf8(serv_resp.into_body().try_into_bytes().unwrap().to_vec()).unwrap();
        serde_json::from_str(&resp_text).unwrap()
    }
}
