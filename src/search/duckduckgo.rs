use serde::Deserialize;
use super::ImageResult;

const PAGE_URL: &str = "https://duckduckgo.com/";
const IMAGE_URL: &str = "https://duckduckgo.com/i.js";

#[derive(Debug, Deserialize)]
struct Response { #[serde(default)] results: Vec<Item> }
#[derive(Debug, Deserialize)]
struct Item {
    #[serde(default)] image: String,
    thumbnail: Option<String>,
    title: Option<String>,
    url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

pub async fn search_images(query: &str, limit: u8) -> Result<Vec<ImageResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .build()?;
    let page = client.get(PAGE_URL).query(&[("q", query), ("iax", "images"), ("ia", "images")])
        .send().await?.error_for_status()?.text().await?;
    let token = extract_vqd(&page).ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::InvalidData, "DuckDuckGo não forneceu o token de sessão VQD"
    ))?;
    let response = client.get(IMAGE_URL)
        .query(&[("q", query), ("o", "json"), ("l", "wt-wt"), ("vqd", token.as_str()), ("f", ",,,,,")])
        .header("referer", PAGE_URL)
        .send().await?.error_for_status()?.json::<Response>().await?;
    Ok(response.results.into_iter().filter(|item| !item.image.is_empty())
        .take(limit.clamp(1, 50) as usize).map(|item| ImageResult {
            image_url: item.image,
            thumbnail_url: item.thumbnail,
            title: item.title,
            source_page: item.url,
            provider: "DuckDuckGo".into(),
            width: item.width,
            height: item.height,
            creator: None,
            license: None,
            license_url: None,
            attribution: None,
        }).collect())
}

fn extract_vqd(html: &str) -> Option<String> {
    for marker in ["vqd=\"", "vqd='", "vqd="] {
        if let Some((_, tail)) = html.split_once(marker) {
            let end = if marker.ends_with('=') { ['&', '"', '\''].iter().filter_map(|c| tail.find(*c)).min() }
                      else { tail.find(marker.chars().last()?) };
            if let Some(end) = end {
                let value = &tail[..end];
                if !value.is_empty() { return Some(value.to_owned()); }
            }
        }
    }
    None
}
