use std::ops::Deref;

use actix_web::{
    body::BoxBody,
    get,
    http::{header::ContentType, StatusCode},
    post, put, web, HttpResponse, Responder,
};
use entities::dto::AssetMintStatus;
use entities::l2::PublicKey;
use interfaces::{
    asset_service::{L1MintError, L2AssetInfo},
    l1_service::L1MintTransactionError,
    l2_storage::L2StorageError,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use util::publickey::PublicKeyExt;

use crate::rest::{auth::ApiKeyExtractor, marshalling, web_app::AppState};

const ASSET_NOT_FOUND: &str = "No asset found with given ID";
const ROYALTY_BASIS_POINTS_MAX_VALUE: u16 = 10_000;

/// Request object for creating an L2 asset
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    /// NFT asset name
    pub name: String,

    /// NFT Metadata JSON
    pub metadata_json: String,

    /// Base58 encoded public key of the asset owner
    pub owner: String,

    /// Base58 encoded public key of the asset creator
    pub creator: String,

    /// Base58 encoded public key of the asset authority
    pub authority: String,

    /// Royalty basis points of asset, should be less than 10_000
    pub royalty_basis_points: u16,

    /// Base58 encoded public key of a coolection the asset belongs to
    pub collection: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub metadata_json: Option<String>,
    pub owner: Option<String>,
    pub creator: Option<String>,
    pub authority: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    pub collection: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L2AssetInfoResponse {
    pub pubkey: String,
    pub name: String,
    pub owner: String,
    pub creator: String,
    pub collection: Option<String>,
    pub authority: String,
    pub create_timestamp: String,
    pub medata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L1MintRequest {
    /// BASE64 encoded bincode serialized solana transaction
    pub tx: String,
    pub callback: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintStatusResponse {
    pub status: AssetMintStatus,
    pub signature: Option<String>,
}

/// Creates an L2 asset.
#[post("/asset")]
pub async fn create_asset(
    _: ApiKeyExtractor,
    req: web::Json<CreateAssetRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    if req.owner.is_empty() {
        return bad_request("owner cannot be empty string");
    };
    if req.creator.is_empty() {
        return bad_request("creator cannot be empty string");
    };
    if req.authority.is_empty() {
        return bad_request("authority cannot be empty string");
    };
    let royalty_basis_points = if req.royalty_basis_points > ROYALTY_BASIS_POINTS_MAX_VALUE {
        return bad_request("royalty basis points cannot be greater than '10 000'");
    } else {
        req.royalty_basis_points
    };
    let collection = if let Some(collection_str) = &req.collection {
        let Some(collection) = PublicKey::from_bs58(collection_str) else {
            return bad_request("collection contains malformed public key");
        };
        Some(collection)
    } else {
        None
    };

    match state
        .asset_service
        .create_asset(
            &req.metadata_json,
            req.owner.deref(),
            req.creator.deref(),
            req.authority.deref(),
            &req.name,
            royalty_basis_points,
            collection,
        )
        .await
    {
        Ok(L2AssetInfo { asset, metadata }) => {
            let dto = state.asset_converter.to_response_asset_dto(&asset, metadata);
            HttpResponse::Created()
                .content_type(ContentType::json())
                .body(json!(dto).to_string())
        }
        Err(e) => internal_server_error(Some(&e.to_string())),
    }
}

#[put("/asset/{pubkey}")]
pub async fn update_asset(
    _: ApiKeyExtractor,
    asset_pubkey: web::Path<String>,
    req: web::Json<UpdateAssetRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let Some(pubkey) = PublicKey::from_bs58(&asset_pubkey) else {
        return bad_request("Invalid asset public key");
    };

    if req.owner.as_ref().map(|v| v.is_empty()).unwrap_or(false) {
        return bad_request("owner cannot be empty string");
    }
    if req.creator.as_ref().map(|v| v.is_empty()).unwrap_or(false) {
        return bad_request("creator cannot be empty string");
    }
    if req.authority.as_ref().map(|v| v.is_empty()).unwrap_or(false) {
        return bad_request("authority cannot be empty string");
    }

    // None - don't touch collection during the update
    // Some(None) - need to delete collection
    // Some(Some(x)) - need to update collection to x
    let collection = match req
        .collection
        .as_ref()
        .map(|op| op.as_ref().map(|v| PublicKey::from_bs58(v)))
    {
        Some(Some(Some(v))) => Some(Some(v)),
        Some(Some(None)) => return bad_request("collection contains malformed public key"),
        Some(None) => Some(None),
        None => None,
    };

    match state
        .asset_service
        .update_asset(
            pubkey,
            req.metadata_json.as_deref(),
            req.owner.clone(),
            req.creator.clone(),
            req.authority.clone(),
            req.name.as_deref(),
            collection,
        )
        .await
    {
        Ok(mayble_l2) => match mayble_l2 {
            Some(L2AssetInfo { asset, metadata }) => {
                let dto = state.asset_converter.to_response_asset_dto(&asset, metadata);
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(json!(dto).to_string())
            }
            None => bad_request(ASSET_NOT_FOUND),
        },
        Err(e) => internal_server_error(Some(&e.to_string())),
    }
}

#[get("/asset/{pubkey}")]
pub async fn get_asset(asset_pubkey: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let Some(pubkey) = PublicKey::from_bs58(&asset_pubkey) else {
        return bad_request("Invalid asset public key");
    };

    match state.asset_service.fetch_asset(pubkey).await {
        Ok(mayble_l2) => match mayble_l2 {
            Some(L2AssetInfo { asset, metadata }) => {
                let dto = state.asset_converter.to_response_asset_dto(&asset, metadata);
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(json!(dto).to_string())
            }
            None => bad_request(ASSET_NOT_FOUND),
        },
        Err(e) => internal_server_error(Some(&e.to_string())),
    }
}

#[get("/asset/{pubkey}/metadata.json")]
pub async fn get_metadata(asset_pubkey: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let Some(pubkey) = PublicKey::from_bs58(&asset_pubkey) else {
        return bad_request("Invalid asset public key");
    };

    match state.asset_service.fetch_metadata(pubkey).await {
        Ok(mayble_metadata) => match mayble_metadata {
            Some(metadata) => HttpResponse::Ok().content_type(ContentType::json()).body(metadata),
            None => bad_request(ASSET_NOT_FOUND),
        },
        Err(e) => internal_server_error(Some(&e.to_string())),
    }
}

/// This endpoint accepts mint CreateV1 mpl-core transaction, that is fully populated
/// and partially signed on the client side.
/// The transaction is verified, signed on by the asset keypair and sent to Solana.
#[post("/asset/mint")]
pub async fn mint_transaction(req: web::Json<L1MintRequest>, state: web::Data<AppState>) -> impl Responder {
    let Ok(tx) = marshalling::decode_transaction(&req.0.tx) else {
        return bad_request("Malformed transaction");
    };
    match state.asset_service.execute_asset_l1_mint(tx, true).await {
        Ok(()) => HttpResponse::new(StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<L1MintError>() {
                bad_request(&e.to_string())
            } else if let Some(e) = e.downcast_ref::<L2StorageError>() {
                bad_request(&e.to_string())
            } else if let Some(e) = e.downcast_ref::<L1MintTransactionError>() {
                bad_request(&e.to_string())
            } else {
                internal_server_error(Some(&e.to_string()))
            }
        }
    }
}

/// This endpoint accepts mint CreateV1 mpl-core transaction, that is fully populated
/// and partially signed on the client side.
/// The transaction is verified, signed on by the asset keypair and sent to Solana.
#[post("/asset/mint-async")]
pub async fn mint_transaction_async(req: web::Json<L1MintRequest>, state: web::Data<AppState>) -> impl Responder {
    let Ok(tx) = marshalling::decode_transaction(&req.0.tx) else {
        return bad_request("Malformed transaction");
    };
    match state.asset_service.execute_asset_l1_mint(tx, false).await {
        Ok(()) => HttpResponse::new(StatusCode::OK),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<L1MintError>() {
                bad_request(&e.to_string())
            } else if let Some(e) = e.downcast_ref::<L2StorageError>() {
                bad_request(&e.to_string())
            } else if let Some(e) = e.downcast_ref::<L1MintTransactionError>() {
                bad_request(&e.to_string())
            } else {
                internal_server_error(Some(&e.to_string()))
            }
        }
    }
}

#[get("/asset/mint/{pubkey}")]
pub async fn mint_status(asset_pubkey: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let Some(pubkey) = PublicKey::from_bs58(&asset_pubkey) else {
        return bad_request("Invalid asset public key");
    };

    match state.asset_service.get_mint_status(pubkey).await {
        Ok((status, signature)) => {
            let resp = MintStatusResponse { status, signature: signature.map(|signature| signature.to_string()) };
            HttpResponse::Ok().content_type(ContentType::json()).json(resp)
        }
        Err(e) => internal_server_error(Some(&e.to_string())),
    }
}

fn bad_request(msg: &str) -> HttpResponse {
    // TODO: need to define common error message structure
    let payload = json!({
        "error": msg,
    });

    HttpResponse::Ok()
        .status(StatusCode::BAD_REQUEST)
        .body(payload.to_string())
}

fn internal_server_error(msg: Option<&str>) -> HttpResponse {
    let mut response = HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    if let Some(m) = msg {
        let payload = json!({
            "error": m,
        });
        response = response.set_body(BoxBody::new(payload.to_string()));
    }
    response
}

impl From<L2AssetInfo> for L2AssetInfoResponse {
    fn from(L2AssetInfo { asset, metadata }: L2AssetInfo) -> Self {
        L2AssetInfoResponse {
            pubkey: bs58::encode(asset.pubkey).into_string(),
            name: asset.name.clone(),
            owner: bs58::encode(asset.owner).into_string(),
            creator: bs58::encode(asset.creator).into_string(),
            collection: asset.collection.map(|v| bs58::encode(v).into_string()),
            authority: bs58::encode(asset.authority).into_string(),
            // Postgres timestamp keep 6 digits fraction of a second
            create_timestamp: asset.create_timestamp.format("%Y-%m-%d %H:%M:%S%.6f").to_string(),
            medata_json: metadata,
        }
    }
}

fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}
