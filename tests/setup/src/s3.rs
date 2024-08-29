use aws_config::{BehaviorVersion, Region};
use testcontainers::ContainerAsync;
use testcontainers_modules::localstack::LocalStack;
use testcontainers::{runners::AsyncRunner, ImageExt};

pub struct S3Container {
    node: ContainerAsync<LocalStack>,
}

impl S3Container {
    pub async fn run() -> anyhow::Result<S3Container> {
        let container_cfg = testcontainers_modules::localstack::LocalStack::default()
            .with_env_var("SERVICES", "s3");

        let node = container_cfg.start().await?;

        Ok(S3Container {
            node
        })
    }

    pub async fn s3_client(&self) -> anyhow::Result<aws_sdk_s3::Client> {
        let host_ip = self.node.get_host().await?;
        let host_port = self.node.get_host_port_ipv4(4566).await?;

        let endpoint_url = format!("http://{host_ip}:{host_port}");
        let creds = aws_sdk_s3::config::Credentials::new("fake", "fake", None, None, "test");

        let config = aws_sdk_s3::config::Builder::default()
            .behavior_version(BehaviorVersion::v2024_03_28())
            .region(Region::new("us-east-1"))
            .credentials_provider(creds)
            .endpoint_url(endpoint_url)
            .force_path_style(true)
            .build();

        let s3_client = aws_sdk_s3::Client::from_conf(config);

        Ok(s3_client)
    }

}
