use serde::Deserialize;
use std::{error::Error, io, time::Duration};

use super::ImageResult;

const PAGE_URL: &str = "https://duckduckgo.com/";
const IMAGE_URL: &str = "https://duckduckgo.com/i.js";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";
const REGION: &str = "br-pt";
const CONTENT_LANGUAGE: &str = "PT";
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

    // Keep the same User-Agent for both requests: DuckDuckGo binds VQD
    // validation tokens to the search query and the HTTP client identity.
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(25))
        .build()?;

    // This URL always uses the current text from the Telescop search field:
    // https://duckduckgo.com/?q=<query>&iar=images&t=h_
    let page_response = client
        .get(build_page_url(query))
        .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,*/*;q=0.8")
        .header(reqwest::header::ACCEPT_LANGUAGE, "pt-BR,pt;q=0.9,en;q=0.8")
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

    // Match the VQD field used by DuckDuckGo's current image-search page.
    // Do not accept an arbitrary occurrence of "vqd=" elsewhere in the HTML.
    let token = extract_vqd(&html).ok_or_else(|| {
        provider_error(
            "o DuckDuckGo não forneceu o token VQD da pesquisa atual; a página pode ter mudado ou bloqueado a requisição"
                .to_owned(),
        )
    })?;

    // The image page uses a JSON endpoint. Its parameters mirror the current
    // DuckDuckGo image-search request flow used by SearXNG's maintained code.
    let image_response = client
        .get(build_image_url(query, &token))
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::REFERER, PAGE_URL)
        .header(reqwest::header::COOKIE, "ad=pt_BR; ah=br-pt; l=br-pt")
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
            // metadata; the UI labels these results as unverified.
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

fn build_image_url(query: &str, token: &str) -> reqwest::Url {
    let mut url = reqwest::Url::parse(IMAGE_URL).expect("IMAGE_URL deve ser uma URL válida");
    url.query_pairs_mut()
        .append_pair("o", "json")
        .append_pair("q", query)
        .append_pair("u", "bing")
        .append_pair("l", REGION)
        .append_pair("bpia", "1")
        .append_pair("vqd", token)
        .append_pair("a", "h_")
        .append_pair("ct", CONTENT_LANGUAGE);
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
        "o DuckDuckGo bloqueou esta requisição (proteção contra automação ou bloqueio temporário); aguarde antes de tentar novamente"
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "o DuckDuckGo limitou temporariamente as requisições"
    } else {
        "resposta HTTP inesperada"
    };
    provider_error(format!(
        "{stage}: HTTP {status}; {detail}"
    ))
}

fn extract_vqd(html: &str) -> Option<String> {
    let (_, tail) = html.split_once("vqd=\"")?;
    let (value, _) = tail.split_once('"')?;
    (!value.trim().is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{build_image_url, build_page_url, extract_vqd};

    #[test]
    fn builds_the_requested_url_from_each_search_term() {
        for query in ["carro", "montanha", "carro de corrida", "árvore"] {
            let url = build_page_url(query);
            let parameters = url.query_pairs().collect::<Vec<_>>();

            assert_eq!(parameters[0].0, "q");
            assert_eq!(parameters[0].1, query);
            assert_eq!(parameters[1].0, "iar");
            assert_eq!(parameters[1].1, "images");
            assert_eq!(parameters[2].0, "t");
            assert_eq!(parameters[2].1, "h_");
        }
    }

    #[test]
    fn builds_the_image_json_request_with_the_same_search_term() {
        let url = build_image_url("carro de corrida", "test-vqd");
        let parameters = url.query_pairs().collect::<Vec<_>>();
        let value = |key: &str| {
            parameters
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.as_ref())
        };

        assert_eq!(value("q"), Some("carro de corrida"));
        assert_eq!(value("o"), Some("json"));
        assert_eq!(value("u"), Some("bing"));
        assert_eq!(value("l"), Some("br-pt"));
        assert_eq!(value("bpia"), Some("1"));
        assert_eq!(value("vqd"), Some("test-vqd"));
        assert_eq!(value("a"), Some("h_"));
        assert_eq!(value("ct"), Some("PT"));
    }

    #[test]
    fn extracts_the_vqd_token_from_the_page() {
        assert_eq!(
            extract_vqd(r#"<script>vqd="token-456";</script>"#),
            Some("token-456".to_owned())
        );
    }

    #[test]
    fn rejects_missing_or_empty_vqd_values() {
        assert_eq!(extract_vqd("<html>sem token</html>"), None);
        assert_eq!(extract_vqd(r#"vqd="";"#), None);
    }
}
