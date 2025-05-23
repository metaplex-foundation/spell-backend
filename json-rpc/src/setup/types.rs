use service::converter::get_metadata_uri_for_key_str;

#[derive(Clone)]
pub struct MetadataUriCreator {
    base: String,
}

impl MetadataUriCreator {
    pub fn new<T: Into<String>>(base: T) -> Self {
        Self { base: base.into() }
    }

    pub fn get_metadata_uri_for_key(&self, public_key: &str) -> String {
        get_metadata_uri_for_key_str(&self.base, public_key)
    }
}

#[cfg(test)]
mod test {
    use crate::setup::types::MetadataUriCreator;
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
