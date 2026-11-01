use scraper::Html;
use serde::Deserialize;
use std::collections::HashMap;

use super::ImageResult;

const API_URL: &str = "https://commons.wikimedia.org/w/api.php";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    query: Query,
    error: Option<ApiError>,
}

#[derive(Debug, Default, Deserialize)]
struct Query {
    // MediaWiki returns query.pages as an object keyed by page ID, not an array.
    #[serde(default)]
    pages: HashMap<String, Page>,
}

#[derive(Debug, Deserialize)]
struct Page {
    title: String,
    #[serde(default)]
    imageinfo: Vec<ImageInfo>,
}

#[derive(Debug, Deserialize)]
struct ImageInfo {
    url: String,
    #[serde(default)]
    thumburl: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    descriptionurl: Option<String>,
    #[serde(default)]
    extmetadata: HashMap<String, MetadataValue>,
}

#[derive(Debug, Deserialize)]
struct MetadataValue {
    #[serde(default)]
    value: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    info: String,
}

/// Search Wikimedia Commons through its documented MediaWiki API.
pub async fn search_images(
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Telescop/0.1.0 (https://github.com/SM-Projetos/Telescop)")
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let limit = limit.clamp(1, 50).to_string();

    let response = client
        .get(API_URL)
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("generator", "search"),
            ("gsrnamespace", "6"),
            ("gsrsearch", query),
            ("gsrlimit", limit.as_str()),
            ("prop", "imageinfo"),
            ("iiprop", "url|size|extmetadata"),
            ("iiurlwidth", "640"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse>()
        .await?;

    if let Some(error) = response.error {
        return Err(std::io::Error::other(format!(
            "Wikimedia Commons API ({}): {}",
            error.code, error.info
        )).into());
    }

    let mut pages = response.query.pages.into_values().collect::<Vec<_>>();
    pages.sort_by(|a, b| a.title.cmp(&b.title));

    Ok(pages
        .into_iter()
        .filter_map(|page| {
            let info = page.imageinfo.into_iter().next()?;
            let creator = metadata_value(&info.extmetadata, "Artist");
            let license = metadata_value(&info.extmetadata, "LicenseShortName");
            let license_url = metadata_value(&info.extmetadata, "LicenseUrl");
            let attribution = metadata_value(&info.extmetadata, "Attribution");

            Some(ImageResult {
                image_url: info.url,
                thumbnail_url: info.thumburl,
                title: Some(page.title.trim_start_matches("File:").to_owned()),
                source_page: info.descriptionurl,
                provider: "Wikimedia Commons".into(),
                width: info.width,
                height: info.height,
                creator,
                license,
                license_url,
                attribution,
            })
        })
        .collect())
}

fn metadata_value(
    metadata: &HashMap<String, MetadataValue>,
    key: &str,
) -> Option<String> {
    let value = metadata.get(key)?.value.trim();
    let plain_text = Html::parse_fragment(value)
        .root_element()
        .text()
        .collect::<String>();
    let plain_text = plain_text.trim();

    if plain_text.is_empty() {
        None
    } else {
        Some(plain_text.to_owned())
    }
}
