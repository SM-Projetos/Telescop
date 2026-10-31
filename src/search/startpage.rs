use super::ImageResult;
use scraper::{Html, Selector};

const SEARCH_URL: &str = "https://www.startpage.com/";

/// Startpage image results are parsed from the image-search HTML page.
/// The page structure is not a stable public API and may change without notice.
pub async fn search_images(query: &str, limit: u8) -> Result<Vec<ImageResult>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .build()?;
    let html = client.get(SEARCH_URL)
        .query(&[("query", query), ("cat", "images"), ("pl", "opensearch")])
        .send().await?.error_for_status()?.text().await?;
    Ok(parse_results(&html, limit))
}

fn parse_results(html: &str, limit: u8) -> Vec<ImageResult> {
    let document = Html::parse_document(html);
    let Ok(images) = Selector::parse("img") else { return Vec::new() };
    let Ok(anchors) = Selector::parse("a") else { return Vec::new() };
    let mut results = Vec::new();

    for image in document.select(&images) {
        let thumbnail = image.value().attr("data-src")
            .or_else(|| image.value().attr("src"))
            .or_else(|| image.value().attr("srcset").and_then(|s| s.split_whitespace().next()));
        let Some(thumbnail) = thumbnail.filter(|url| url.starts_with("http")) else { continue };
        if thumbnail.contains("logo") || thumbnail.contains("icon") { continue; }

        let parent_link = image.ancestors().filter_map(scraper::ElementRef::wrap)
            .find(|node| node.value().name() == "a");
        let source = parent_link.and_then(|node| node.value().attr("href"))
            .filter(|url| url.starts_with("http"));
        let title = image.value().attr("alt").filter(|text| !text.trim().is_empty());
        results.push(ImageResult {
            // Startpage proxies thumbnails. Until its result markup exposes the original
            // image URL reliably, use the returned image URL and preserve the source page.
            image_url: thumbnail.to_owned(),
            thumbnail_url: Some(thumbnail.to_owned()),
            title: title.map(str::to_owned),
            source_page: source.map(str::to_owned),
            provider: "Startpage".into(),
            width: None,
            height: None,
        });
        if results.len() >= limit.clamp(1, 50) as usize { break; }
    }
    results
}
