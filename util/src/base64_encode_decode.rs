use crate::publickey::PublicKeyExt;
use anyhow::bail;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use chrono::NaiveDateTime;
use entities::l2::{pubkey_to_string, PublicKey};

/// For pagination of assets in the utility chain, we use a string encoded in `base64` format in the following structure:
/// `slot+asset_pubkey`
/// (`asset_pubkey` is a string encoded in `base54` format).
///
/// In our backend implementation, the slot number is absent because the assets have not been minted on-chain.
/// As a result, instead of the slot argument, we use a `timestamp` combined with the `asset_pubkey`,
///  separated by the `=` symbol.
/// Thus, the valid pagination string format would be:
///     `2015-09-18T23:56:04=CqToY3qWMRKK3H8UpmXLUQoduUFL8U9JizjN2oCevnFV`
/// The timestamp format adheres to the following specification: https://docs.rs/chrono/0.4.38/chrono/naive/struct.NaiveDateTime.html#impl-FromStr-for-NaiveDateTime.
pub fn decode_timestamp_and_asset_pubkey(encoded_key: &str) -> anyhow::Result<(NaiveDateTime, PublicKey)> {
    let Ok(decoded_key) = STANDARD_NO_PAD.decode(encoded_key) else { bail!("Base64 decoding error!") };

    let decoded_key = String::from_utf8(decoded_key)?;
    let Some((date, pubkey)) = decoded_key.split_once('=') else {
        bail!("No encoded separator provided!")
    };

    let creation_timestamp = date.parse::<NaiveDateTime>()?;

    let Some(pubkey) = PublicKey::from_bs58(pubkey) else { bail!("Cannot create public key!") };

    Ok((creation_timestamp, pubkey))
}

/// We also need to return a cursor for pagination, which must be encoded in the same format:
/// `'timestamp' + '=' + 'asset_pubkey'` encoded in base64,
pub fn encode_timestamp_and_asset_pubkey(date: NaiveDateTime, pubkey: PublicKey) -> String {
    STANDARD_NO_PAD.encode(format!("{}={}", date.format("%Y-%m-%dT%H:%M:%S%.f").to_string(), pubkey_to_string(pubkey)))
}

#[cfg(test)]
mod test {
    use crate::base64_encode_decode::{decode_timestamp_and_asset_pubkey, encode_timestamp_and_asset_pubkey};
    use crate::publickey::PublicKeyExt;
    use chrono::NaiveDateTime;
    use entities::l2::PublicKey;

    #[test]
    fn decode_creation_timestamp_and_asset_pubkey_test() {
        // base64 encoding of:
        //  2015-09-18T23:56:04=CqToY3qWMRKK3H8UpmXLUQoduUFL8U9JizjN2oCevnFV
        // with no padding and UTF-8 destination character set
        let input = "MjAxNS0wOS0xOFQyMzo1NjowND1DcVRvWTNxV01SS0szSDhVcG1YTFVRb2R1VUZMOFU5Sml6ak4yb0Nldm5GVg";

        let expected_date = "2015-09-18T23:56:04".parse::<NaiveDateTime>().unwrap();
        let expected_pubkey = PublicKey::from_bs58("CqToY3qWMRKK3H8UpmXLUQoduUFL8U9JizjN2oCevnFV").unwrap();
        let (actual_date, actual_pubkey) = decode_timestamp_and_asset_pubkey(input).unwrap();

        assert_eq!(actual_date, expected_date);
        assert_eq!(actual_pubkey, expected_pubkey)
    }

    #[test]
    fn encode_creation_timestamp_and_asset_pubkey_test() {
        // base64 encoding of:
        //  2015-09-18T23:56:04=CqToY3qWMRKK3H8UpmXLUQoduUFL8U9JizjN2oCevnFV
        // with no padding and UTF-8 destination character set
        let input = "MjAxNS0wOS0xOFQyMzo1NjowND1DcVRvWTNxV01SS0szSDhVcG1YTFVRb2R1VUZMOFU5Sml6ak4yb0Nldm5GVg";

        let input_date = "2015-09-18T23:56:04".parse::<NaiveDateTime>().unwrap();
        let input_pubkey = PublicKey::from_bs58("CqToY3qWMRKK3H8UpmXLUQoduUFL8U9JizjN2oCevnFV").unwrap();
        let res = encode_timestamp_and_asset_pubkey(input_date, input_pubkey);

        assert_eq!(&res, "MjAxNS0wOS0xOFQyMzo1NjowND1DcVRvWTNxV01SS0szSDhVcG1YTFVRb2R1VUZMOFU5Sml6ak4yb0Nldm5GVg");
    }
}
