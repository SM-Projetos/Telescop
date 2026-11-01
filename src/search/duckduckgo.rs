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
    // Do not default this field: a changed/challenge response must not be
    // silently reported as a successful search with zero results.
    results: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
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

    // The VQD value is associated with the exact query and User-Agent.
    // Keep one stable User-Agent for both requests.
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(25))
        .build()?;

    // Build the same URL pattern used by the user's browser:
    // https://duckduckgo.com/?q=<current search text>&iar=images&t=h_
    let page_response = client
        .get(build_page_url(query))
        .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,*/*;q=0.8")
        .header(reqwest::header::ACCEPT_LANGUAGE, "pt-BR,pt;q=0.9,en;q=0.8")
        .header(reqwest::header::REFERER, PAGE_URL)
        .header("sec-fetch-dest", "document")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-user", "?1")
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

    if is_challenge_page(&html) {
        return Err(provider_error(
            "o DuckDuckGo apresentou um desafio antirobô. A página não contém resultados utilizáveis; verifique se a busca funciona no navegador normal antes de tentar novamente pelo Telescop"
                .to_owned(),
        ));
    }

    // Use the VQD belonging to this exact query. An empty/missing token must
    // stop the search rather than send a request that may cause IP blocking.
    let token = extract_vqd(&html).ok_or_else(|| {
        provider_error(
            "o DuckDuckGo não forneceu o token VQD da pesquisa atual; a página pode ter mudado ou bloqueado a requisição"
                .to_owned(),
        )
    })?;

    // DuckDuckGo's image endpoint is undocumented. These parameters follow
    // the current SearXNG implementation of the image-search request.
    let image_response = client
        .get(build_image_url(query, &token))
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::REFERER, PAGE_URL)
        .header(reqwest::header::COOKIE, "ad=pt_BR; ah=br-pt; l=br-pt")
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
            "resposta inesperada do DuckDuckGo: não foi possível interpretar o JSON de imagens ({error})"
        ))
    })?;

    let results = validate_results(response.results)?;
    Ok(results
        .into_iter()
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

fn is_challenge_page(html: &str) -> bool {
    html.contains("id=\"challenge-form\"")
        || html.contains("id='challenge-form'")
        || html.contains("anomaly-modal")
}

fn validate_results(
    mut results: Vec<Item>,
) -> Result<Vec<Item>, Box<dyn Error + Send + Sync>> {
    if results.is_empty() {
        return Err(provider_error(
            "o DuckDuckGo respondeu com uma lista JSON vazia. Isso pode indicar ausência de resultados ou limitação temporária do serviço"
                .to_owned(),
        ));
    }

    let received_count = results.len();
    results.retain(|item| !item.image.trim().is_empty());
    if results.is_empty() {
        return Err(provider_error(format!(
            "o DuckDuckGo retornou {received_count} registro(s), mas nenhum contém uma URL de imagem utilizável"
        )));
    }

    Ok(results)
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
    provider_error(format!("{stage}: HTTP {status}; {detail}"))
}

fn extract_vqd(html: &str) -> Option<String> {
    let (_, tail) = html.split_once("vqd=\"")?;
    let (value, _) = tail.split_once('"')?;
    (!value.trim().is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        build_image_url, build_page_url, extract_vqd, is_challenge_page, validate_results, Item,
    };

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

    #[test]
    fn detects_the_duckduckgo_robot_challenge() {
        assert!(is_challenge_page(r#"<form id="challenge-form"></form>"#));
        assert!(is_challenge_page("<div class='anomaly-modal'></div>"));
        assert!(!is_challenge_page("<html><body>Image results</body></html>"));
    }

    #[test]
    fn rejects_empty_or_unusable_image_result_lists() {
        assert!(validate_results(Vec::new()).is_err());

        let unusable = Item {
            image: "  ".into(),
            thumbnail: None,
            title: None,
            url: None,
            width: None,
            height: None,
        };
        assert!(validate_results(vec![unusable]).is_err());
    }

    #[test]
    fn preserves_image_results_with_urls() {
        let usable = Item {
            image: "https://example.test/image.jpg".into(),
            thumbnail: Some("https://example.test/thumb.jpg".into()),
            title: Some("Example".into()),
            url: Some("https://example.test".into()),
            width: Some(640),
            height: Some(480),
        };
        let results = validate_results(vec![usable]).expect("resultado deveria ser válido");
        assert_eq!(results.len(), 1);
    }
}
