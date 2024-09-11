use anyhow::{bail, Context};
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::engine::GeneralPurpose;
use base64::Engine;
use chrono::{DateTime, NaiveDateTime};
use entities::l2::PublicKey;
use std::mem::size_of;

const BASE64_ENGINE: GeneralPurpose = STANDARD_NO_PAD;

pub fn decode_timestamp_and_asset_pubkey(encoded_key: &str) -> anyhow::Result<(NaiveDateTime, PublicKey)> {
    let key = BASE64_ENGINE
        .decode(encoded_key)
        .context("Failed to decode base64 string")?;

    if key.len() < 8 {
        bail!("Invalid key: Not enough data for a timestamp (requires at least 8 bytes)");
    }

    let timestamp_millis = i64::from_be_bytes(key[0..8].try_into()?);

    let timestamp = DateTime::from_timestamp_millis(timestamp_millis)
        .context("Invalid timestamp: Could not parse into NaiveDateTime")?
        .naive_utc();

    let pubkey = PublicKey::try_from(&key[8..]).context("Failed to parse public key from remaining bytes")?;

    Ok((timestamp, pubkey))
}

pub fn encode_timestamp_and_asset_pubkey(date: NaiveDateTime, pubkey: PublicKey) -> String {
    let timestamp_size = size_of::<i64>();
    let pubkey_size = size_of::<PublicKey>();

    let mut vec = Vec::with_capacity(timestamp_size + pubkey_size);

    vec.extend_from_slice(&date.and_utc().timestamp_millis().to_be_bytes());
    vec.extend_from_slice(&pubkey);

    BASE64_ENGINE.encode(vec)
}

#[cfg(test)]
mod test {
    use crate::base64_encode_decode::{decode_timestamp_and_asset_pubkey, encode_timestamp_and_asset_pubkey};
    use crate::publickey::PublicKeyExt;
    use chrono::NaiveDateTime;
    use entities::l2::PublicKey;

    #[test]
    fn encode_and_decode_timestamp_and_asset_pubkey_test() {
        let input_date = NaiveDateTime::default();
        let input_pubkey = PublicKey::new_unique();
        let encoded = encode_timestamp_and_asset_pubkey(input_date, input_pubkey);
        let (time, pubkey) = decode_timestamp_and_asset_pubkey(&encoded).unwrap();

        assert_eq!(time, input_date);
        assert_eq!(pubkey, input_pubkey);
    }
}
