use regex::Regex;

use lazy_static::lazy_static;
lazy_static! {
    static ref URL_PASSWD_RE: Regex = Regex::new(r"\w+:\/\/\w+:(\w+)@.*").unwrap();
}

pub fn form_url(host: &str, port: u16, path: &str) -> String {
    format!("{host}:{port}/{path}")
}

pub fn mask_url_passwd(url: &str) -> String {
    let mut masked_url = url.to_string();

    if let Some(m) = URL_PASSWD_RE.captures_iter(url).next().and_then(|c| c.get(1)) {
        masked_url.replace_range(m.start()..m.end(), "****");
    };

    masked_url
}

pub fn mask_creds(s: &str) -> String {
    let mut result = s.to_owned();
    result.replace_range(2..s.len(), "*".repeat(s.len() - 2).as_str());
    result
}

#[test]
fn test_masking_password() {
    assert_eq!(
        mask_url_passwd("postgres://postgres:postgres@localhost:5432/db"),
        "postgres://postgres:****@localhost:5432/db".to_string()
    );
}
