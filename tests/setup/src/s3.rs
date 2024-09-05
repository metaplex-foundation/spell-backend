use aws_config::{BehaviorVersion, Region};
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::localstack::LocalStack;
use util::config::ObjStorageCfg;

pub struct S3Container {
    node: ContainerAsync<LocalStack>,
}

pub const BUCKET: &str = "test-bucket";

impl S3Container {
    pub async fn run() -> anyhow::Result<S3Container> {
        let container_cfg = testcontainers_modules::localstack::LocalStack::default().with_env_var("SERVICES", "s3");

        let node = container_cfg.start().await?;

        {
            let s3_client = s3_client(&node).await;
            s3_client.create_bucket().bucket(BUCKET).send().await.unwrap();
        }

        Ok(S3Container { node })
    }

    pub async fn s3_client(&self) -> aws_sdk_s3::Client {
        s3_client(&self.node).await
    }

    pub async fn obj_storage_cfg(&self) -> ObjStorageCfg {
        obj_storage_cfg(&self.node).await
    }
}

async fn endpoint_url(node: &ContainerAsync<LocalStack>) -> String {
    let host_ip = node.get_host().await.unwrap();
    let host_port = node.get_host_port_ipv4(4566).await.unwrap();

    format!("http://{host_ip}:{host_port}")
}

pub async fn s3_client(node: &ContainerAsync<LocalStack>) -> aws_sdk_s3::Client {
    let endpoint_url = endpoint_url(node).await;
    let creds = aws_sdk_s3::config::Credentials::new("fake", "fake", None, None, "test");

    let config = aws_sdk_s3::config::Builder::default()
        .behavior_version(BehaviorVersion::v2024_03_28())
        .region(Region::new("us-east-1"))
        .credentials_provider(creds)
        .endpoint_url(endpoint_url)
        .force_path_style(true)
        .build();

    aws_sdk_s3::Client::from_conf(config)
}

pub async fn obj_storage_cfg(node: &ContainerAsync<LocalStack>) -> ObjStorageCfg {
    ObjStorageCfg {
        region: Some("us-east-1".to_string()),
        endpoint: Some(endpoint_url(node).await),
        access_key_id: Some("fake".to_string()),
        secret_access_key: Some("fake".to_string()),
        session_token: None,
        bucket_for_json_metadata: BUCKET.to_string(),
        bucket_for_binary_assets: BUCKET.to_string(),
    }
}
