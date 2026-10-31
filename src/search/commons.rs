use serde::Deserialize;

use super::ImageResult;

const API_URL: &str = "https://commons.wikimedia.org/w/api.php";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    query: Query,
}

#[derive(Debug, Default, Deserialize)]
struct Query {
    #[serde(default)]
    pages: Vec<Page>,
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
}

/// Search Wikimedia Commons through its documented MediaWiki API.
pub async fn search_images(
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("Telescop/0.1.0 (desktop image search)")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client
        .get(API_URL)
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("generator", "search"),
            ("gsrnamespace", "6"),
            ("gsrsearch", query),
            ("gsrlimit", &limit.clamp(1, 50).to_string()),
            ("prop", "imageinfo"),
            ("iiprop", "url|size"),
            ("iiurlwidth", "640"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse>()
        .await?;

    Ok(response.query.pages.into_iter().filter_map(|page| {
        let info = page.imageinfo.into_iter().next()?;
        Some(ImageResult {
            image_url: info.url,
            thumbnail_url: info.thumburl,
            title: Some(page.title.trim_start_matches("File:").to_owned()),
            source_page: info.descriptionurl,
            provider: "Wikimedia Commons".into(),
            width: info.width,
            height: info.height,
        })
    }).collect())
}
