use serde::Deserialize;
use std::{error::Error, io, time::Duration};

use super::ImageResult;

const PAGE_URL: &str = "https://duckduckgo.com/";
const IMAGE_URL: &str = "https://duckduckgo.com/i.js";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";
const MAX_QUERY_CHARS: usize = 499;

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(default)]
    results: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    #[serde(default)]
    image: String,
    thumbnail: Option<String>,
    title: Option<String>,
    url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

pub async fn search_images(
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn Error + Send + Sync>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(provider_error(format!(
            "a pesquisa excede o limite de {MAX_QUERY_CHARS} caracteres aceito pelo DuckDuckGo"
        )));
    }

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(25))
        .build()?;

    // This reproduces the image-search URL supplied for Telescop:
    // https://duckduckgo.com/?q=<query>&iar=images&t=h_
    let page_url = build_page_url(query);
    let page_response = client
        .get(page_url)
        .header(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .header(
            reqwest::header::ACCEPT_LANGUAGE,
            "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7",
        )
        .header("sec-fetch-dest", "document")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-site", "none")
        .header("sec-fetch-user", "?1")
        .header("upgrade-insecure-requests", "1")
        .send()
        .await
        .map_err(|error| {
            provider_error(format!("falha ao acessar a página de imagens: {error}"))
        })?;

    let page_status = page_response.status();
    if !page_status.is_success() {
        return Err(http_status_error("abrir a página de imagens", page_status));
    }

    let html = page_response
        .text()
        .await
        .map_err(|error| provider_error(format!("falha ao ler a página de imagens: {error}")))?;

    let token = extract_vqd(&html).ok_or_else(|| {
        provider_error(
            "o DuckDuckGo não forneceu o token VQD; a página pode ter mudado ou bloqueado a requisição"
                .to_owned(),
        )
    })?;

    // The HTML page is not itself a machine-readable list of images.
    // DuckDuckGo's page requests image records from this JSON endpoint.
    let image_response = client
        .get(IMAGE_URL)
        .query(&[
            ("q", query),
            ("o", "json"),
            ("l", "br-pt"),
            ("vqd", token.as_str()),
        ])
        .header(
            reqwest::header::ACCEPT,
            "application/json, text/javascript, */*; q=0.01",
        )
        .header(reqwest::header::ACCEPT_LANGUAGE, "pt-BR,pt;q=0.9,en;q=0.8")
        .header(reqwest::header::REFERER, PAGE_URL)
        .header("x-requested-with", "XMLHttpRequest")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .send()
        .await
        .map_err(|error| {
            provider_error(format!("falha ao solicitar os resultados de imagens: {error}"))
        })?;

    let image_status = image_response.status();
    if !image_status.is_success() {
        return Err(http_status_error(
            "receber os resultados de imagens",
            image_status,
        ));
    }

    let response = image_response.json::<Response>().await.map_err(|error| {
        provider_error(format!(
            "o DuckDuckGo retornou uma resposta de imagens inválida: {error}"
        ))
    })?;

    Ok(response
        .results
        .into_iter()
        .filter(|item| !item.image.trim().is_empty())
        .take(limit.clamp(1, 50) as usize)
        .map(|item| ImageResult {
            image_url: item.image,
            thumbnail_url: item.thumbnail,
            title: item.title,
            source_page: item.url,
            provider: "DuckDuckGo".into(),
            width: item.width,
            height: item.height,
            creator: None,
            // DuckDuckGo's image response does not provide reliable license
            // metadata; the UI must label these results as unverified.
            license: None,
            license_url: None,
            attribution: None,
        })
        .collect())
}

fn build_page_url(query: &str) -> reqwest::Url {
    let mut url = reqwest::Url::parse(PAGE_URL).expect("PAGE_URL deve ser uma URL válida");
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("iar", "images")
        .append_pair("t", "h_");
    url
}

fn provider_error(message: String) -> Box<dyn Error + Send + Sync> {
    Box::new(io::Error::new(io::ErrorKind::Other, message))
}

fn http_status_error(
    stage: &str,
    status: reqwest::StatusCode,
) -> Box<dyn Error + Send + Sync> {
    let detail = if status == reqwest::StatusCode::FORBIDDEN {
        "o acesso foi bloqueado pelo DuckDuckGo"
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "o DuckDuckGo limitou temporariamente as requisições"
    } else {
        "resposta HTTP inesperada"
    };
    provider_error(format!(
        "DuckDuckGo: {stage}: HTTP {status}; {detail}"
    ))
}

fn extract_vqd(html: &str) -> Option<String> {
    for marker in ["vqd=\"", "vqd='", "vqd="] {
        if let Some((_, tail)) = html.split_once(marker) {
            let end = if marker == "vqd=\"" {
                tail.find('"')
            } else if marker == "vqd='" {
                tail.find('\'')
            } else {
                tail.find(|character: char| {
                    matches!(character, '&' | '"' | '\'' | '<' | ';' | ' ')
                })
            };

            if let Some(end) = end {
                let value = &tail[..end];
                if !value.is_empty() {
                    return Some(value.to_owned());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{build_page_url, extract_vqd};

    #[test]
    fn builds_the_requested_duckduckgo_images_url() {
        let url = build_page_url("carro de corrida");
        let parameters = url.query_pairs().collect::<Vec<_>>();

        assert_eq!(parameters[0].0, "q");
        assert_eq!(parameters[0].1, "carro de corrida");
        assert_eq!(parameters[1].0, "iar");
        assert_eq!(parameters[1].1, "images");
        assert_eq!(parameters[2].0, "t");
        assert_eq!(parameters[2].1, "h_");
    }

    #[test]
    fn extracts_single_quoted_vqd_token() {
        assert_eq!(
            extract_vqd("<input value='x' data-vqd='token-123'>"),
            Some("token-123".to_owned())
        );
    }

    #[test]
    fn extracts_double_quoted_vqd_token() {
        assert_eq!(
            extract_vqd(r#"<script>vqd="token-456";</script>"#),
            Some("token-456".to_owned())
        );
    }

    #[test]
    fn extracts_unquoted_vqd_token() {
        assert_eq!(
            extract_vqd("vqd=token-789&other=value"),
            Some("token-789".to_owned())
        );
    }

    #[test]
    fn returns_none_when_token_is_missing_or_empty() {
        assert_eq!(extract_vqd("<html>sem token</html>"), None);
        assert_eq!(extract_vqd("vqd=&other=value"), None);
    }
}
