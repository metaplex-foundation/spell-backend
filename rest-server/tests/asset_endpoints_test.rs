mod test_app_util;

#[cfg(test)]
mod tests {
    use actix_web::{body::MessageBody, http::StatusCode, test};
    use entities::l2::PublicKey;
    use rest_server::rest::endpoints::l2_assets::{CreateAssetRequest, UpdateAssetRequest};
    use setup::TestEnvironmentCfg;
    use util::publickey::PublicKeyExt;

    use crate::test_app_util::{self, extract_asset_from_response};

    #[actix_web::test]
    async fn test_l2_asset_endpoints() {
        // Prepare test env
        let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
        let app = test_app_util::init_web_app(&t_env).await;

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
                royalty_basis_points: 1345,
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
            extract_asset_from_response(serv_resp)
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
                .append_header(("x-api-key", "111"))
                .set_json(req_payload)
                .to_request();

            let serv_resp = test::call_service(&app, req).await;
            assert_eq!(serv_resp.status(), StatusCode::OK);
            extract_asset_from_response(serv_resp)
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
            extract_asset_from_response(serv_resp)
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
}
