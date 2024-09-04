#[derive(Clone)]
pub struct MetadataUriCreator {
    base: String,
}

impl MetadataUriCreator {
    pub fn new(base: String) -> Self {
        Self { base }
    }

    pub fn get_metadata_uri_for_key(&self, public_key: &str) -> String {
        format!("{}/asset/{}/metadata.json", self.base, public_key)
    }
}

#[cfg(test)]
mod test {
    use crate::config::types::MetadataUriCreator;
    use std::net::{Ipv4Addr, SocketAddr};

    #[test]
    fn test() {
        let base = SocketAddr::from((Ipv4Addr::LOCALHOST, 8080)).to_string();
        assert_eq!("127.0.0.1:8080", base);
        let metadata_uri = MetadataUriCreator::new(base);
        let res = metadata_uri.get_metadata_uri_for_key("some_pubkey_112233");
        assert_eq!("127.0.0.1:8080/asset/some_pubkey_112233/metadata.json", res);
    }
}
