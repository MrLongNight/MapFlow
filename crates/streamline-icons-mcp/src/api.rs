//! Streamline Icons API Client
//!
//! HTTP client for interacting with the Streamline Icons API.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

const API_BASE_URL: &str = "https://public-api.streamlinehq.com";

/// Streamline API Client
pub struct StreamlineClient {
    client: reqwest::Client,
    api_key: String,
}

impl StreamlineClient {
    /// Create a new client using the ICONS_API_KEY environment variable
    pub fn from_env() -> Result<Self> {
        let api_key =
            env::var("ICONS_API_KEY").context("ICONS_API_KEY environment variable not set")?;
        Ok(Self::new(api_key))
    }

    /// Create a new client with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    /// Search for icons, illustrations, emojis, or elements
    pub async fn search(&self, params: SearchParams) -> Result<SearchResponse> {
        let url = format!("{}/v1/search/global", API_BASE_URL);

        let mut query = vec![
            ("productType", params.product_type.to_string()),
            ("query", params.query),
        ];

        if let Some(offset) = params.offset {
            query.push(("offset", offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(tier) = params.product_tier {
            query.push(("productTier", tier));
        }
        if let Some(style) = params.style {
            query.push(("style", style));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&query)
            .send()
            .await
            .context("Failed to send search request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        response
            .json()
            .await
            .context("Failed to parse search response")
    }

    /// Search within a specific family
    pub async fn family_search(&self, params: FamilySearchParams) -> Result<SearchResponse> {
        let url = format!("{}/v1/search/family/{}", API_BASE_URL, params.family_slug);

        let mut query = vec![("query", params.query)];

        if let Some(offset) = params.offset {
            query.push(("offset", offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&query)
            .send()
            .await
            .context("Failed to send family search request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        response
            .json()
            .await
            .context("Failed to parse family search response")
    }

    /// Get detailed information about a specific icon
    pub async fn get_icon_by_hash(&self, icon_hash: &str) -> Result<IconDetails> {
        let url = format!("{}/v1/icons/{}", API_BASE_URL, icon_hash);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to send get icon request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        response
            .json()
            .await
            .context("Failed to parse icon details response")
    }

    /// Download an icon as SVG
    pub async fn download_svg(&self, params: DownloadSvgParams) -> Result<String> {
        let url = format!("{}/v1/icons/{}/svg", API_BASE_URL, params.icon_hash);

        let mut query: Vec<(&str, String)> = vec![("size", params.size.to_string())];

        if let Some(colors) = params.colors {
            for color in colors {
                query.push(("colors", color));
            }
        }
        if let Some(bg) = params.background_color {
            query.push(("backgroundColor", bg));
        }
        if let Some(responsive) = params.responsive {
            query.push(("responsive", responsive.to_string()));
        }
        if let Some(stroke_width) = params.stroke_width {
            query.push(("strokeWidth", stroke_width.to_string()));
        }
        if let Some(stroke_to_fill) = params.stroke_to_fill {
            query.push(("strokeToFill", stroke_to_fill.to_string()));
        }
        if let Some(base64) = params.base64 {
            query.push(("base64", base64.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&query)
            .send()
            .await
            .context("Failed to send download SVG request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        response.text().await.context("Failed to get SVG content")
    }

    /// Download an icon as PNG (returns base64-encoded data)
    pub async fn download_png(&self, params: DownloadPngParams) -> Result<Vec<u8>> {
        let url = format!("{}/v1/icons/{}/png", API_BASE_URL, params.icon_hash);

        let mut query: Vec<(&str, String)> = vec![("size", params.size.to_string())];

        if let Some(colors) = params.colors {
            for color in colors {
                query.push(("colors", color));
            }
        }
        if let Some(bg) = params.background_color {
            query.push(("backgroundColor", bg));
        }
        if let Some(stroke_width) = params.stroke_width {
            query.push(("strokeWidth", stroke_width.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&query)
            .send()
            .await
            .context("Failed to send download PNG request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .context("Failed to get PNG content")
    }
}

// === Request/Response Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductType {
    #[serde(rename = "icons")]
    Icons,
    #[serde(rename = "illustrations")]
    Illustrations,
    #[serde(rename = "emojis")]
    Emojis,
    #[serde(rename = "elements")]
    Elements,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductType::Icons => write!(f, "icons"),
            ProductType::Illustrations => write!(f, "illustrations"),
            ProductType::Emojis => write!(f, "emojis"),
            ProductType::Elements => write!(f, "elements"),
        }
    }
}

impl std::str::FromStr for ProductType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "icons" => Ok(ProductType::Icons),
            "illustrations" => Ok(ProductType::Illustrations),
            "emojis" => Ok(ProductType::Emojis),
            "elements" => Ok(ProductType::Elements),
            _ => anyhow::bail!(
                "Invalid product type: {}. Must be one of: icons, illustrations, emojis, elements",
                s
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub product_type: ProductType,
    pub query: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub product_tier: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FamilySearchParams {
    pub family_slug: String,
    pub query: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DownloadSvgParams {
    pub icon_hash: String,
    pub size: u32,
    pub colors: Option<Vec<String>>,
    pub background_color: Option<String>,
    pub responsive: Option<bool>,
    pub stroke_width: Option<f32>,
    pub stroke_to_fill: Option<bool>,
    pub base64: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DownloadPngParams {
    pub icon_hash: String,
    pub size: u32,
    pub colors: Option<Vec<String>>,
    pub background_color: Option<String>,
    pub stroke_width: Option<f32>,
}

// === API Response Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub results: Vec<IconResult>,
    #[serde(default)]
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub next_skip: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconResult {
    pub hash: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub image_preview_url: String,
    #[serde(default)]
    pub is_free: bool,
    #[serde(default)]
    pub family_slug: String,
    #[serde(default)]
    pub family_name: String,
    #[serde(default)]
    pub category_slug: String,
    #[serde(default)]
    pub category_name: String,
    #[serde(default)]
    pub subcategory_slug: String,
    #[serde(default)]
    pub subcategory_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconDetails {
    pub hash: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub family_name: String,
    #[serde(default)]
    pub family_slug: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub is_free: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}
