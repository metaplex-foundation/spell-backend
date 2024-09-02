use std::{cmp::Ordering, collections::BTreeMap, path::Path};

use entities::l2::{self, AssetExtended, L2Asset};
use jsonpath_lib::JsonPathError;
use mime_guess::Mime;
use schemars::JsonSchema;
use serde_json::Value;
use tracing::warn;
use url::Url;
use {
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

pub const COLLECTION_GROUP_KEY: &str = "collection";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Quality {
    #[serde(rename = "$$schema")]
    pub schema: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum Context {
    #[serde(rename = "wallet-default")]
    WalletDefault,
    #[serde(rename = "web-desktop")]
    WebDesktop,
    #[serde(rename = "web-mobile")]
    WebMobile,
    #[serde(rename = "app-mobile")]
    AppMobile,
    #[serde(rename = "app-desktop")]
    AppDesktop,
    #[serde(rename = "app")]
    App,
    #[serde(rename = "vr")]
    Vr,
}

pub type Contexts = Vec<Context>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct File {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<Quality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Contexts>,
}

pub type Files = Vec<File>;

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct MetadataMap(BTreeMap<String, serde_json::Value>);

impl MetadataMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn inner(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.0
    }

    pub fn set_item(&mut self, key: &str, value: serde_json::Value) -> &mut Self {
        self.0.insert(key.to_string(), value);
        self
    }

    pub fn get_item(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }
}

// TODO sub schema support
pub type Links = HashMap<String, serde_json::Value>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Content {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub json_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Files>,
    pub metadata: MetadataMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Scope {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "royalty")]
    Royalty,
    #[serde(rename = "metadata")]
    Metadata,
    #[serde(rename = "extension")]
    Extension,
}

impl From<String> for Scope {
    fn from(s: String) -> Self {
        match &*s {
            "royalty" => Scope::Royalty,
            "metadata" => Scope::Metadata,
            "extension" => Scope::Extension,
            _ => Scope::Full,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Authority {
    pub address: String,
    pub scopes: Vec<Scope>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Compression {
    pub eligible: bool,
    pub compressed: bool,
    pub data_hash: String,
    pub creator_hash: String,
    pub asset_hash: String,
    pub tree: String,
    pub seq: i64,
    pub leaf_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Group {
    pub group_key: String,
    pub group_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_metadata: Option<MetadataMap>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum RoyaltyModel {
    #[serde(rename = "creators")]
    Creators,
    #[serde(rename = "fanout")]
    Fanout,
    #[serde(rename = "single")]
    Single,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Royalty {
    pub royalty_model: RoyaltyModel,
    pub target: Option<String>,
    pub percent: f64,
    pub basis_points: u32,
    pub primary_sale_happened: bool,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Creator {
    pub address: String,
    pub share: i32,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum OwnershipModel {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "token")]
    Token,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ownership {
    pub frozen: bool,
    pub delegated: bool,
    pub delegate: Option<String>,
    pub ownership_model: OwnershipModel,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}
impl From<String> for UseMethod {
    fn from(s: String) -> Self {
        match &*s {
            "Burn" => UseMethod::Burn,
            "Single" => UseMethod::Single,
            "Multiple" => UseMethod::Multiple,
            _ => UseMethod::Single,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Supply {
    pub print_max_supply: Option<u64>, // None value mean that NFT is printable and has an unlimited supply (https://developers.metaplex.com/token-metadata/print)
    pub print_current_supply: u64,
    pub edition_nonce: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition_number: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MplCoreInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_minted: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_size: Option<u32>,
    pub plugins_json_version: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub interface: String, // "MplCoreAsset"
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<Authority>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<Compression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<Vec<Group>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub royalty: Option<Royalty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<Creator>>,
    pub ownership: Ownership,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<Uses>,
    pub supply: Option<Supply>,
    pub mutable: bool,
    pub burnt: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rent_epoch: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<PluginSchemaV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_plugins: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mpl_core_info: Option<MplCoreInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_plugins: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_external_plugins: Option<Value>,
    // it's only missing the inscription field, that's not used anyway
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spl20: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PluginAuthority {
    None,
    Owner,
    UpdateAuthority,
    Address {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Royalties {
    pub basis_points: u16,
    pub creators: Vec<Creator>,
    pub rule_set: RuleSet,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum RuleSet {
    None,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    pub data: Royalties,
}

pub fn safe_select<'a>(
    selector: &mut impl FnMut(&str) -> Result<Vec<&'a Value>, JsonPathError>,
    expr: &str,
) -> Option<&'a Value> {
    selector(expr)
        .ok()
        .filter(|d| !Vec::is_empty(d))
        .as_mut()
        .and_then(|v| v.pop())
}
pub fn parse_files_from_selector<'a>(
    selector: &mut impl FnMut(&str) -> Result<Vec<&'a Value>, JsonPathError>,
) -> (HashMap<String, Value>, Vec<File>) {
    let mut links = HashMap::new();
    let link_fields = vec!["image", "animation_url", "external_url"];
    for f in link_fields {
        let l = safe_select(selector, format!("$.{}", f).as_str());
        if let Some(l) = l {
            links.insert(f.to_string(), l.to_owned());
        }
    }

    let mut actual_files: HashMap<String, File> = HashMap::new();
    if let Some(files) = selector("$.properties.files[*]")
        .ok()
        .filter(|d| !d.is_empty())
    {
        for v in files.iter() {
            if v.is_object() {
                // Some assets don't follow the standard and specifiy 'url' instead of 'uri'
                let mut uri = v.get("uri");
                if uri.is_none() {
                    uri = v.get("url");
                }
                let mime_type = v.get("type");

                match (uri, mime_type) {
                    (Some(u), Some(m)) => {
                        if let Some(str_uri) = u.as_str() {
                            let file = if let Some(str_mime) = m.as_str() {
                                File {
                                    uri: Some(str_uri.to_string()),
                                    cdn_uri: None,
                                    mime: Some(str_mime.to_string()),
                                    quality: None,
                                    contexts: None,
                                }
                            } else {
                                warn!("Mime is not string: {:?}", m);
                                file_from_str(str_uri.to_string())
                            };
                            actual_files.insert(str_uri.to_string(), file);
                        } else {
                            warn!("URI is not string: {:?}", u);
                        }
                    }
                    (Some(u), None) => {
                        let str_uri = serde_json::to_string(u).unwrap_or_default();
                        actual_files.insert(str_uri.clone(), file_from_str(str_uri));
                    }
                    _ => {}
                }
            } else if v.is_string() {
                let str_uri = v.as_str().unwrap().to_string();
                actual_files.insert(str_uri.clone(), file_from_str(str_uri));
            }
        }
    }

    track_top_level_file(&mut actual_files, links.get("image"));
    track_top_level_file(&mut actual_files, links.get("animation_url"));

    let mut files: Vec<File> = actual_files.into_values().collect();
    // List the defined image file before the other files (if one exists).
    files.sort_by(|a, _: &File| match (a.uri.as_ref(), links.get("image")) {
        (Some(x), Some(y)) => {
            if x == y {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
        _ => Ordering::Equal,
    });

    (links, files)
}

pub fn track_top_level_file(
    file_map: &mut HashMap<String, File>,
    top_level_file: Option<&serde_json::Value>,
) {
    if top_level_file.is_some() {
        let img = top_level_file.and_then(|x| x.as_str());
        if let Some(img) = img {
            let entry = file_map.get(img);
            if entry.is_none() {
                file_map.insert(img.to_string(), file_from_str(img.to_string()));
            }
        }
    }
}
pub fn file_from_str(str: String) -> File {
    let mime = get_mime_type_from_uri(str.clone());
    File {
        uri: Some(str),
        cdn_uri: None,
        mime: Some(mime),
        quality: None,
        contexts: None,
    }
}

pub fn to_uri(uri: String) -> Option<Url> {
    Url::parse(uri.as_str()).ok()
}

pub fn get_mime(url: Url) -> Option<Mime> {
    mime_guess::from_path(Path::new(url.path())).first()
}

pub fn get_mime_type_from_uri(uri: String) -> String {
    let default_mime_type = "image/png".to_string();
    to_uri(uri)
        .and_then(get_mime)
        .map_or(default_mime_type, |m| m.to_string())
}
impl From<(AssetExtended, Value)> for Asset {
    fn from(value: (AssetExtended, Value)) -> Self {
        let (asset, metadata) = value;
        let l2_asset = asset.asset;
        let mut meta: MetadataMap = MetadataMap::new();

        meta.set_item("name", Value::String(l2_asset.name));
        meta.set_item("symbol", Value::String("".to_string())); // for core assets symbol is empty

        let mut selector_fn = jsonpath_lib::selector(&metadata);
        let selector = &mut selector_fn;

        let desc = safe_select(selector, "$.description");
        if let Some(desc) = desc {
            meta.set_item("description", desc.clone());
        }

        let attributes = safe_select(selector, "$.attributes");
        if let Some(attributes) = attributes {
            meta.set_item("attributes", attributes.clone());
        }

        let (links, files) = parse_files_from_selector(selector);
        let creators = vec![Creator {
            address: entities::l2::pubkey_to_string(l2_asset.creator),
            share: 100,
            verified: true, // todo: is it?
        }];
        Asset {
            interface: "MplCoreAsset".to_string(),
            id: entities::l2::pubkey_to_string(l2_asset.pubkey),
            content: Some(Content {
                schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                json_uri: asset.metadata_uri,
                files: Some(files),
                metadata: meta,
                links: Some(links),
            }),
            authorities: Some(vec![Authority {
                address: entities::l2::pubkey_to_string(l2_asset.authority),
                scopes: vec![Scope::Full],
            }]),
            compression: Some(Compression::default()),
            grouping: l2_asset.collection.map(|c| {
                vec![Group {
                    group_key: COLLECTION_GROUP_KEY.to_string(),
                    group_value: Some(entities::l2::pubkey_to_string(c)),
                    verified: Some(true), // todo: is it?
                    collection_metadata: None, //todo: figure out the collection flow with Spell
                }]
            }),
            royalty: Some(Royalty {
                royalty_model: RoyaltyModel::Creators,
                target: None,
                percent: (asset.royalty_basis_points as f64) * 0.0001,
                basis_points: asset.royalty_basis_points as u32,
                primary_sale_happened: false, // for core assets
                locked: false,
            }),
            creators: Some(creators.clone()),
            ownership: Ownership {
                frozen: false,
                delegated: false,
                delegate: None, // for core assets can come from transfer delegate plugin
                ownership_model: OwnershipModel::Single,
                owner: entities::l2::pubkey_to_string(l2_asset.owner),
            },
            uses: None, // for core assets
            supply: None,
            mutable: true,
            burnt: false,
            lamports: None,
            executable: None,
            metadata_owner: None,
            rent_epoch: None,
            plugins: Some(PluginSchemaV1{
                index: 0,
                offset: 0, //todo - this is purely onchain, change to constant, when we mint some of these assets
                authority: PluginAuthority::UpdateAuthority,
                data: Royalties {
                    basis_points: asset.royalty_basis_points,
                    creators,
                    rule_set: RuleSet::None,
                },
            }),
            unknown_plugins: None,
            mpl_core_info: None,
            external_plugins: None,
            unknown_external_plugins: None,
            spl20: None,
        }
    }
}
