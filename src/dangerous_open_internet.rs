use std::str::FromStr;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use toml::{Table, Value};

#[derive(serde::Serialize, Debug)]
pub struct ValidToy {
    pub item: String,
    pub quantity: u32,
}

impl TryFrom<Table> for ValidToy {
    type Error = String;

    fn try_from(value: Table) -> Result<Self, Self::Error> {
        let quantity = match value.get("quantity") {
            Some(opt) => match opt {
                Value::Integer(quantity) => *quantity as u32,
                _ => return Err("Invalid quantity type".to_string()),
            },
            None => return Err("Missing quantity".to_string()),
        };
        let item = match value.get("item").to_owned() {
            Some(item) => match item {
                Value::String(item) => item.to_string(),
                _ => return Err("Invalid item type".to_string()),
            },
            None => return Err("Missing item".to_string()),
        };
        Ok(ValidToy { quantity, item })
    }
}

impl TryFrom<serde_yaml::Value> for ValidToy {
    type Error = String;

    fn try_from(value: serde_yaml::Value) -> Result<Self, Self::Error> {
        let item = value["item"].as_str().ok_or("Invalid item")?;
        let quantity = value["quantity"].as_i64().ok_or("Invalid quantity")?;
        Ok(Self {
            item: item.to_string(),
            quantity: quantity as u32,
        })
    }
}

impl TryFrom<serde_json::Value> for ValidToy {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let item = value["item"].as_str().ok_or("Invalid item")?;
        let quantity = value["quantity"].as_i64().ok_or("Invalid quantity")?;
        Ok(Self {
            item: item.to_string(),
            quantity: quantity as u32,
        })
    }
}

impl std::fmt::Display for ValidToy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", &self.item, self.quantity)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ManifestPackageMetadata<T> {
    orders: Option<Vec<T>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ManifestPackage<T> {
    name: String,
    keywords: Option<Vec<String>>,
    metadata: Option<ManifestPackageMetadata<T>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Manifest<T> {
    package: ManifestPackage<T>,
}

#[derive(thiserror::Error, Debug)]
enum ManifestParseError {
    #[error("")]
    InvalidContentType,
    #[error("Invalid manifest")]
    InvalidManifest,
    #[error("Magic keyword not provided")]
    MissingMagicKeyword,
    #[error("")]
    MissingOrders,
}

impl IntoResponse for ManifestParseError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ManifestParseError::InvalidContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ManifestParseError::MissingMagicKeyword => StatusCode::BAD_REQUEST,
            ManifestParseError::InvalidManifest => StatusCode::BAD_REQUEST,
            ManifestParseError::MissingOrders => StatusCode::NO_CONTENT,
        };

        (status_code, self.to_string()).into_response()
    }
}

macro_rules! parse_manifest {
    ($ns: ident, $body: expr) => {{
        let raw =
            $ns::from_str::<$ns::Value>(&$body).map_err(|_| ManifestParseError::InvalidManifest)?;
        let toml_string = toml::to_string(&raw).map_err(|_| ManifestParseError::InvalidManifest)?;
        cargo_manifest::Manifest::from_str(&toml_string)
            .map_err(|_| ManifestParseError::InvalidManifest)?;

        let manifest = $ns::from_value::<Manifest<$ns::Value>>(raw).unwrap();
        parse_manifest(manifest)
    }};
}

fn parse_toml(body: String) -> Result<String, ManifestParseError> {
    cargo_manifest::Manifest::from_str(&body).map_err(|_| ManifestParseError::InvalidManifest)?;
    let package_manifest = toml::from_str::<Manifest<Table>>(&body).unwrap();
    parse_manifest(package_manifest)
}

fn parse_yaml(body: String) -> Result<String, ManifestParseError> {
    parse_manifest!(serde_yaml, body)
}

fn parse_json(body: String) -> Result<String, ManifestParseError> {
    parse_manifest!(serde_json, body)
}

fn parse_manifest<T>(manifest: Manifest<T>) -> Result<String, ManifestParseError>
where
    ValidToy: TryFrom<T>,
{
    match manifest.package.keywords {
        None => return Err(ManifestParseError::MissingMagicKeyword),
        Some(k) if !k.contains(&String::from("Christmas 2024")) => {
            return Err(ManifestParseError::MissingMagicKeyword);
        }
        _ => {}
    };

    let Some(orders) = manifest.package.metadata.and_then(|m| m.orders) else {
        return Err(ManifestParseError::MissingOrders);
    };

    let toys = orders
        .into_iter()
        .filter_map(|o| ValidToy::try_from(o).ok())
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    if toys.is_empty() {
        Err(ManifestParseError::MissingOrders)
    } else {
        Ok(toys)
    }
}

pub async fn manifest(headers: HeaderMap, body: String) -> impl IntoResponse {
    let Some(content_type) = headers.get("Content-Type").and_then(|h| h.to_str().ok()) else {
        return (StatusCode::BAD_REQUEST, "Missing Content Type".to_string()).into_response();
    };

    match content_type {
        "application/toml" => parse_toml(body),
        "application/yaml" => parse_yaml(body),
        "application/json" => parse_json(body),
        _ => Err(ManifestParseError::InvalidContentType),
    }
    .into_response()
}
