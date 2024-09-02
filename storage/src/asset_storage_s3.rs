use aws_sdk_s3::{error::SdkError, primitives::ByteStream};
use entities::l2::PublicKey;
use interfaces::asset_storage::{AssetMetadataStorage, BlobStorage};
use std::sync::Arc;

const MIME_JSON: &str = "application/json";

#[derive(Clone, Debug)]
pub struct S3Storage {
    s3_client: Arc<aws_sdk_s3::Client>,
    metadata_bucket: String,
    asset_bucket: String,
}

impl S3Storage {
    pub async fn new(metadata_bucket: &str, asset_bucket: &str, s3_client: Arc<aws_sdk_s3::Client>) -> S3Storage {
        S3Storage {
            s3_client: s3_client,
            metadata_bucket: metadata_bucket.to_string(),
            asset_bucket: asset_bucket.to_string(),
        }
    }

    pub async fn mocked() -> S3Storage {
        let creds = aws_sdk_s3::config::Credentials::new("fake", "fake", None, None, "test");
        let s3_config = aws_sdk_s3::config::Builder::default()
            .behavior_version(BehaviorVersion::v2024_03_28())
            .region(Region::new("us-east-1"))
            .credentials_provider(creds)
            .endpoint_url("http://127.0.0.1:3030")
            .force_path_style(true)
            .build();

        S3Storage {
            s3_client: Arc::new(aws_sdk_s3::Client::from_conf(s3_config)),
            bucket_name: "mocked s3 storage".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AssetMetadataStorage for S3Storage {
    async fn put_json(&self, pubkey: &PublicKey, json_metadata: &str) -> anyhow::Result<()> {
        let key = make_metadata_key(pubkey);

        let byte_stream: ByteStream = json_metadata.as_bytes().to_vec().into();

        let _resp = self
            .s3_client
            .put_object()
            .bucket(&self.metadata_bucket)
            .key(key)
            .content_type(MIME_JSON)
            .body(byte_stream)
            .send()
            .await?;
        Ok(())
    }

    async fn get_json(&self, pubkey: &PublicKey) -> anyhow::Result<Option<String>> {
        let key = make_metadata_key(pubkey);
        let resp = self
            .s3_client
            .get_object()
            .bucket(&self.metadata_bucket)
            .key(key)
            .send()
            .await;

        match resp {
            Ok(ok_resp) => {
                let bytes = ok_resp.body.collect().await?.into_bytes().to_vec();
                let text = String::from_utf8(bytes)?;
                Ok(Some(text))
            }
            Err(SdkError::ServiceError(service_error)) if service_error.err().is_no_such_key() => Ok(None),
            Err(e) => anyhow::bail!(e),
        }
    }
}

#[async_trait::async_trait]
impl BlobStorage for S3Storage {
    async fn put_binary(&self, pubkey: &PublicKey, bytes: Vec<u8>, mime: &str) -> anyhow::Result<()> {
        let key = make_binary_key(pubkey);

        let _resp = self
            .s3_client
            .put_object()
            .bucket(&self.asset_bucket)
            .key(key)
            .content_type(mime)
            .body(bytes.into())
            .send()
            .await?;
        Ok(())
    }

    async fn get_binary(&self, pubkey: &PublicKey) -> anyhow::Result<(Vec<u8>, String)> {
        let key = make_binary_key(pubkey);
        let resp = self
            .s3_client
            .get_object()
            .bucket(&self.asset_bucket)
            .key(key)
            .send()
            .await?;

        let bytes = resp.body.collect().await?.into_bytes().to_vec();

        let mime = resp.content_type.unwrap_or("application/octet-stream".to_string());

        Ok((bytes, mime))
    }
}

pub fn make_metadata_key(pubkey: &PublicKey) -> String {
    let asset_id = bs58::encode(pubkey).into_string();
    format!("asset-metadata/{}", asset_id)
}

pub fn make_binary_key(pubkey: &PublicKey) -> String {
    let asset_id = bs58::encode(pubkey).into_string();
    format!("asset-binary/{}", asset_id)
}
