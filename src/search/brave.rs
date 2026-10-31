use serde::Deserialize;

use super::ImageResult;

const API_URL: &str = "https://api.search.brave.com/res/v1/images/search";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    results: Vec<ApiImage>,
}

#[derive(Debug, Deserialize)]
struct ApiImage {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    thumbnail: Option<Thumbnail>,
    #[serde(default)]
    properties: Option<Properties>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Properties {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

/// Search images through the official Brave Search API.
///
/// The API token is supplied by the caller and must never be committed to source control.
pub async fn search_images(
    query: &str,
    api_key: &str,
    limit: u16,
) -> Result<Vec<ImageResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Telescop/0.1.0 (https://github.com/SM-Projetos/Telescop)")
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let count = limit.clamp(1, 200).to_string();

    let response = client
        .get(API_URL)
        .header(reqwest::header::ACCEPT, "application/json")
        .header("X-Subscription-Token", api_key)
        .query(&[
            ("q", query),
            ("count", count.as_str()),
            ("country", "BR"),
            ("search_lang", "pt"),
            ("safesearch", "strict"),
            ("spellcheck", "true"),
        ])
        .send()
        .await?;

    let status = response.status();
    let body = response.bytes().await?;
    if !status.is_success() {
        let detail = String::from_utf8_lossy(&body);
        let detail = detail.chars().take(500).collect::<String>();
        return Err(format!("Brave Search API retornou HTTP {status}: {detail}").into());
    }

    let response: ApiResponse = serde_json::from_slice(&body)?;
    Ok(response
        .results
        .into_iter()
        .filter_map(|item| {
            let thumbnail = item.thumbnail?;
            let thumbnail_url = thumbnail.src?;
            let image_url = item
                .properties
                .as_ref()
                .and_then(|properties| properties.url.clone())
                .unwrap_or_else(|| thumbnail_url.clone());
            let properties = item.properties;

            Some(ImageResult {
                image_url,
                thumbnail_url: Some(
                    properties
                        .as_ref()
                        .and_then(|properties| properties.placeholder.clone())
                        .unwrap_or(thumbnail_url),
                ),
                title: item.title,
                source_page: item.url,
                provider: "Brave Search".into(),
                width: properties
                    .as_ref()
                    .and_then(|properties| properties.width)
                    .or(thumbnail.width),
                height: properties
                    .as_ref()
                    .and_then(|properties| properties.height)
                    .or(thumbnail.height),
            })
        })
        .take(limit.clamp(1, 200) as usize)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_image_results_and_dimensions() {
        let payload = r#"{
            "results": [{
                "title": "Test image",
                "url": "https://example.com/page",
                "thumbnail": {
                    "src": "https://example.com/thumb.jpg",
                    "width": 320,
                    "height": 200
                },
                "properties": {
                    "url": "https://example.com/original.jpg",
                    "width": 1920,
                    "height": 1200
                }
            }]
        }"#;

        let parsed: ApiResponse = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed.results.len(), 1);
        let item = parsed.results.into_iter().next().unwrap();
        assert_eq!(item.url.as_deref(), Some("https://example.com/page"));
        assert_eq!(
            item.properties.as_ref().and_then(|properties| properties.width),
            Some(1920)
        );
    }
}
