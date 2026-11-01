use reqwest::{header, Client, Url};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::{error::Error, io};

use super::ImageResult;

const USER_AGENT: &str =
    "Telescop/0.1.0 (https://github.com/SM-Projetos/Telescop)";
const REQUEST_TIMEOUT_SECONDS: u64 = 25;
const MAX_RESULTS: usize = 100;

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    img_src: Option<String>,
    thumbnail_src: Option<String>,
    title: Option<String>,
    url: Option<String>,
    resolution: Option<String>,
    author: Option<String>,
    source: Option<String>,
}

/// Searches the image category of a user-configured SearXNG instance.
///
/// The instance URL is its base URL (for example, https://example.org), not
/// the /search endpoint. JSON is preferred; HTML is used when JSON is disabled.
pub async fn search_images(
    instance_url: &str,
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn Error + Send + Sync>> {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let base_url = parse_instance_url(instance_url)?;
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .build()?;

    let json_url = build_search_url(&base_url, query, true)?;
    let response = client
        .get(json_url)
        .header(header::ACCEPT, "application/json")
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return search_html(&client, &base_url, query, limit).await;
    }

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("A instância SearXNG respondeu com HTTP {}.", response.status()),
        )
        .into());
    }

    let body = response.bytes().await?;
    match serde_json::from_slice::<SearchResponse>(&body) {
        Ok(payload) => Ok(normalize_items(payload.results, limit)),
        Err(json_error) => {
            search_html(&client, &base_url, query, limit)
                .await
                .map_err(|html_error| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "A resposta JSON do SearXNG é inválida ({json_error}); \
                             a alternativa HTML também falhou: {html_error}"
                        ),
                    )
                    .into()
                })
        }
    }
}

fn parse_instance_url(value: &str) -> Result<Url, Box<dyn Error + Send + Sync>> {
    let value = value.trim();
    if value.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Configure SEARXNG_INSTANCE_URL com o endereço-base de uma instância SearXNG.",
        )
        .into());
    }

    let mut url = Url::parse(value)?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SEARXNG_INSTANCE_URL deve ser uma URL HTTP(S) base, sem credenciais, consulta ou fragmento.",
        )
        .into());
    }

    let path = url.path().trim_end_matches('/');
    url.set_path(&format!("{path}/search"));
    Ok(url)
}

fn build_search_url(
    base_url: &Url,
    query: &str,
    json: bool,
) -> Result<Url, Box<dyn Error + Send + Sync>> {
    let mut url = base_url.clone();
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("categories", "images")
        .append_pair("language", "pt-BR")
        .append_pair("safesearch", "1");

    if json {
        url.query_pairs_mut().append_pair("format", "json");
    }

    Ok(url)
}

async fn search_html(
    client: &Client,
    base_url: &Url,
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn Error + Send + Sync>> {
    let url = build_search_url(base_url, query, false)?;
    let response = client
        .get(url)
        .header(
            header::ACCEPT,
            "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8",
        )
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "A alternativa HTML do SearXNG respondeu com HTTP {}. \
                 Verifique a URL e as regras da instância.",
                response.status()
            ),
        )
        .into());
    }

    let html = response.text().await?;
    let results = parse_html_results(&html, limit)?;
    if results.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "A instância SearXNG não apresentou resultados de imagem em HTML. \
             Ela pode bloquear esse formato ou exigir uma configuração diferente.",
        )
        .into());
    }

    Ok(results)
}

fn normalize_items(items: Vec<SearchItem>, limit: u8) -> Vec<ImageResult> {
    items
        .into_iter()
        .filter_map(|item| {
            let image_url = valid_web_url(item.img_src.as_deref()?)?;
            let thumbnail_url = item
                .thumbnail_src
                .as_deref()
                .and_then(valid_web_url);
            let (width, height) = parse_resolution(item.resolution.as_deref());

            Some(ImageResult {
                image_url,
                thumbnail_url,
                title: item.title.filter(|value| !value.trim().is_empty()),
                source_page: item.url.as_deref().and_then(valid_web_url),
                provider: "SearXNG".into(),
                width,
                height,
                creator: item.author.filter(|value| !value.trim().is_empty()),
                // SearXNG's general image results do not establish reuse rights.
                license: None,
                license_url: None,
                attribution: item.source.filter(|value| !value.trim().is_empty()),
            })
        })
        .take(usize::from(limit).min(MAX_RESULTS))
        .collect()
}

fn parse_html_results(
    html: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn Error + Send + Sync>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let document = Html::parse_document(html);
    let article_selector = Selector::parse("article.result-images")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let image_selector = Selector::parse("img.image_thumbnail")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let title_selector = Selector::parse("span.title")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let source_selector = Selector::parse("span.source")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let resolution_selector = Selector::parse("span.image_resolution")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let source_page_selector = Selector::parse(".result-url a")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let anchor_selector = Selector::parse("a")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    let mut results = Vec::new();
    for article in document.select(&article_selector) {
        let Some(image) = article.select(&image_selector).next() else {
            continue;
        };

        let image_url = article
            .select(&anchor_selector)
            .find_map(|anchor| anchor.value().attr("href").and_then(valid_web_url))
            .or_else(|| {
                image
                    .value()
                    .attr("data-src")
                    .or_else(|| image.value().attr("src"))
                    .and_then(valid_web_url)
            });
        let Some(image_url) = image_url else {
            continue;
        };

        let thumbnail_url = image
            .value()
            .attr("data-src")
            .or_else(|| image.value().attr("src"))
            .and_then(valid_web_url);
        let title = article
            .select(&title_selector)
            .next()
            .map(|node| node.text().collect::<String>().trim().to_owned())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                image
                    .value()
                    .attr("alt")
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
            });
        let attribution = article
            .select(&source_selector)
            .next()
            .map(|node| node.text().collect::<String>().trim().to_owned())
            .filter(|value| !value.is_empty());
        let resolution = article
            .select(&resolution_selector)
            .next()
            .map(|node| node.text().collect::<String>());
        let (width, height) = parse_resolution(resolution.as_deref());
        let source_page = article
            .select(&source_page_selector)
            .next()
            .and_then(|node| node.value().attr("href"))
            .and_then(valid_web_url);

        results.push(ImageResult {
            image_url,
            thumbnail_url,
            title,
            source_page,
            provider: "SearXNG".into(),
            width,
            height,
            creator: None,
            license: None,
            license_url: None,
            attribution,
        });

        if results.len() >= usize::from(limit).min(MAX_RESULTS) {
            break;
        }
    }

    Ok(results)
}

fn valid_web_url(value: &str) -> Option<String> {
    let value = value.trim();
    let url = Url::parse(value).ok()?;
    matches!(url.scheme(), "http" | "https").then(|| value.to_owned())
}

fn parse_resolution(value: Option<&str>) -> (Option<u32>, Option<u32>) {
    let Some(value) = value else {
        return (None, None);
    };
    let normalized = value.replace('×', "x");
    let mut dimensions = normalized.split('x');
    let width = dimensions.next().and_then(parse_dimension);
    let height = dimensions.next().and_then(parse_dimension);
    (width, height)
}

fn parse_dimension(value: &str) -> Option<u32> {
    let digits = value.trim().chars().take_while(char::is_ascii_digit).collect::<String>();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_json_image_search_url_with_current_query() {
        let base = parse_instance_url("https://search.example.org/").unwrap();
        let url = build_search_url(&base, "carro de corrida", true).unwrap();

        assert_eq!(url.path(), "/search");
        let params = url.query_pairs().collect::<std::collections::HashMap<_, _>>();
        assert_eq!(params.get("q").map(|value| value.as_ref()), Some("carro de corrida"));
        assert_eq!(params.get("categories").map(|value| value.as_ref()), Some("images"));
        assert_eq!(params.get("format").map(|value| value.as_ref()), Some("json"));
        assert_eq!(params.get("safesearch").map(|value| value.as_ref()), Some("1"));
    }

    #[test]
    fn preserves_reverse_proxy_base_path() {
        let base = parse_instance_url("https://search.example.org/metasearch/").unwrap();
        let url = build_search_url(&base, "árvore", false).unwrap();

        assert_eq!(url.path(), "/metasearch/search");
        assert!(!url.query().unwrap().contains("format=json"));
    }

    #[test]
    fn rejects_instance_urls_with_credentials_or_non_http_scheme() {
        assert!(parse_instance_url("ftp://search.example.org").is_err());
        assert!(parse_instance_url("https://user:pass@search.example.org").is_err());
        assert!(parse_instance_url("https://search.example.org/?q=other").is_err());
    }

    #[test]
    fn parses_resolution_with_common_separators() {
        assert_eq!(parse_resolution(Some("1920 x 1080")), (Some(1920), Some(1080)));
        assert_eq!(parse_resolution(Some("800×600")), (Some(800), Some(600)));
        assert_eq!(parse_resolution(Some("sem dimensão")), (None, None));
    }

    #[test]
    fn normalizes_json_results_and_does_not_claim_reuse_rights() {
        let json = r#"{
            "results": [{
                "img_src": "https://images.example.org/car.jpg",
                "thumbnail_src": "https://images.example.org/thumb.jpg",
                "title": "Carro",
                "url": "https://gallery.example.org/item/1",
                "resolution": "1280x720",
                "author": "Autor",
                "source": "Galeria"
            }]
        }"#;
        let response: SearchResponse = serde_json::from_str(json).unwrap();
        let results = normalize_items(response.results, 30);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "SearXNG");
        assert_eq!(results[0].width, Some(1280));
        assert_eq!(results[0].height, Some(720));
        assert_eq!(results[0].license, None);
        assert_eq!(results[0].creator.as_deref(), Some("Autor"));
    }

    #[test]
    fn skips_results_without_valid_image_urls() {
        let json = r#"{
            "results": [
                {"img_src": "javascript:alert(1)", "title": "Inválida"},
                {"img_src": "https://images.example.org/valid.jpg", "title": "Válida"}
            ]
        }"#;
        let response: SearchResponse = serde_json::from_str(json).unwrap();
        let results = normalize_items(response.results, 30);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.as_deref(), Some("Válida"));
    }

    #[test]
    fn parses_documented_searxng_image_html() {
        let html = r#"
            <article class="result result-images">
                <a href="https://images.example.org/full.jpg">
                    <img class="image_thumbnail" src="https://images.example.org/thumb.jpg"
                         alt="Paisagem" />
                    <span class="image_resolution">1920 x 1080</span>
                    <span class="title">Paisagem</span>
                    <span class="source">example.org</span>
                </a>
                <div class="detail">
                    <p class="result-url">
                        <a href="https://example.org/page">Origem</a>
                    </p>
                </div>
            </article>
        "#;
        let results = parse_html_results(html, 30).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].image_url, "https://images.example.org/full.jpg");
        assert_eq!(
            results[0].thumbnail_url.as_deref(),
            Some("https://images.example.org/thumb.jpg")
        );
        assert_eq!(results[0].source_page.as_deref(), Some("https://example.org/page"));
        assert_eq!(results[0].width, Some(1920));
        assert_eq!(results[0].height, Some(1080));
        assert_eq!(results[0].license, None);
    }
}
