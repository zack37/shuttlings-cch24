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

fn parse_toml(body: String) -> Result<String, ManifestParseError> {
    if cargo_manifest::Manifest::from_str(&body).is_err() {
        return Err(ManifestParseError::InvalidManifest);
    };

    let package_manifest = toml::from_str::<Manifest<Table>>(&body).unwrap();
    parse_manifest::<Table>(package_manifest)
}

fn parse_yaml(body: String) -> Result<String, ManifestParseError> {
    let raw = serde_yaml::from_str::<serde_yaml::Value>(&body).unwrap();
    let toml_string = toml::to_string(&raw).map_err(|_| ManifestParseError::InvalidManifest)?;
    let manifest = serde_yaml::from_value::<Manifest<serde_yaml::Value>>(raw).unwrap();
    if cargo_manifest::Manifest::from_str(&toml_string).is_err() {
        return Err(ManifestParseError::InvalidManifest);
    };

    parse_manifest(manifest)
}

fn parse_json(body: String) -> Result<String, ManifestParseError> {
    let raw = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|_| ManifestParseError::InvalidManifest)?;
    let toml_string = toml::to_string(&raw).map_err(|_| ManifestParseError::InvalidManifest)?;
    let manifest = serde_json::from_value::<Manifest<serde_json::Value>>(raw).unwrap();
    if cargo_manifest::Manifest::from_str(&toml_string).is_err() {
        return Err(ManifestParseError::InvalidManifest);
    }

    parse_manifest(manifest)
}

fn parse_manifest<T>(manifest: Manifest<T>) -> Result<String, ManifestParseError>
where
    ValidToy: TryFrom<T>,
{
    let keywords_opt = manifest.package.keywords;
    if keywords_opt.is_none()
        || !keywords_opt
            .unwrap()
            .contains(&String::from("Christmas 2024"))
    {
        return Err(ManifestParseError::MissingMagicKeyword);
    }

    let Some(metadata) = manifest.package.metadata else {
        return Err(ManifestParseError::MissingOrders);
    };
    let Some(orders) = metadata.orders else {
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
    dbg!(&body);
    let Some(content_type) = headers.get("Content-Type") else {
        return (StatusCode::BAD_REQUEST, "Missing Content Type".to_string()).into_response();
    };
    let Ok(content_type) = content_type.to_str() else {
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
