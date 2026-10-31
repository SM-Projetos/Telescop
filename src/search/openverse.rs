use serde::Deserialize;

use super::ImageResult;

const API_URL: &str = "https://api.openverse.org/v1/images/";

#[derive(Debug, Deserialize)]
struct OpenverseResponse {
    #[serde(default)]
    results: Vec<OpenverseImage>,
}

#[derive(Debug, Deserialize)]
struct OpenverseImage {
    #[serde(default)]
    url: String,
    thumbnail: Option<String>,
    title: Option<String>,
    foreign_landing_url: Option<String>,
    #[serde(default = "provider_name")]
    provider: String,
    width: Option<u32>,
    height: Option<u32>,
}

fn provider_name() -> String {
    "Openverse".to_owned()
}

/// Searches Openverse and converts its response into Telescop's shared result model.
pub async fn search_images(query: &str, limit: u8) -> Result<Vec<ImageResult>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("Telescop/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let page_size = limit.clamp(1, 50).to_string();
    let response = client
        .get(API_URL)
        .query(&[("q", query), ("page_size", page_size.as_str())])
        .send()
        .await?
        .error_for_status()?
        .json::<OpenverseResponse>()
        .await?;

    Ok(response
        .results
        .into_iter()
        .filter(|image| !image.url.trim().is_empty())
        .map(|image| ImageResult {
            image_url: image.url,
            thumbnail_url: image.thumbnail,
            title: image.title,
            source_page: image.foreign_landing_url,
            provider: image.provider,
            width: image.width,
            height: image.height,
        })
        .collect())
}
