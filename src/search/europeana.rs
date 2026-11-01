use serde::Deserialize;
use std::time::Duration;

use super::ImageResult;

const API_URL: &str = "https://api.europeana.eu/record/v2/search.json";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    items: Vec<EuropeanaItem>,
}

#[derive(Debug, Deserialize)]
struct EuropeanaItem {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    link: Option<String>,
    #[serde(default)]
    title: Vec<String>,
    #[serde(default, rename = "dcCreator")]
    creator: Vec<String>,
    #[serde(default, rename = "edmPreview")]
    preview: Vec<String>,
    #[serde(default, rename = "edmIsShownBy")]
    shown_by: Vec<String>,
    #[serde(default, rename = "edmIsShownAt")]
    shown_at: Vec<String>,
    #[serde(default)]
    rights: Vec<String>,
    #[serde(default, rename = "dataProvider")]
    data_provider: Vec<String>,
}

/// Searches Europeana for records marked as openly reusable.
///
/// Europeana requires an API key. Personal keys are free; configure one in
/// EUROPEANA_API_KEY instead of embedding credentials in the source code.
pub async fn search_images(
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = std::env::var("EUROPEANA_API_KEY").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Europeana requer uma chave pessoal gratuita. Obtenha-a em https://www.europeana.eu/en/apis e configure EUROPEANA_API_KEY.",
        )
    })?;
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "EUROPEANA_API_KEY está vazia. Configure sua chave pessoal gratuita.",
        )
        .into());
    }

    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "Telescop/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/SM-Projetos/Telescop)"
        ))
        .timeout(Duration::from_secs(20))
        .build()?;

    let rows = limit.clamp(1, 50).to_string();
    let response = client
        .get(API_URL)
        .header("X-Api-Key", api_key)
        .query(&[
            ("query", query),
            ("qf", "TYPE:IMAGE"),
            ("reusability", "open"),
            ("media", "true"),
            ("thumbnail", "true"),
            ("rows", rows.as_str()),
            ("profile", "standard"),
        ])
        .send()
        .await
        .map_err(|_| std::io::Error::other("Falha na requisição da API da Europeana"))?;

    if !response.status().is_success() {
        return Err(std::io::Error::other(format!(
            "A API da Europeana retornou HTTP {}",
            response.status()
        ))
        .into());
    }

    let response = response
        .json::<SearchResponse>()
        .await
        .map_err(|_| std::io::Error::other("A API da Europeana retornou uma resposta JSON inválida"))?;

    if response.success == Some(false) {
        return Err(std::io::Error::other(
            response.error.unwrap_or_else(|| {
                "A API da Europeana informou uma falha sem detalhes.".into()
            }),
        )
        .into());
    }

    Ok(response
        .items
        .into_iter()
        .filter_map(to_image_result)
        .take(limit.clamp(1, 50) as usize)
        .collect())
}

fn to_image_result(item: EuropeanaItem) -> Option<ImageResult> {
    let rights_url = item.rights.first()?.trim();
    let license = allowed_license_label(rights_url)?;
    // media=true guarantees a direct media URL; preview is used only as thumbnail.
    let image_url = item.shown_by.first()?.clone();
    let thumbnail_url = item.preview.first().cloned();
    let title = item.title.first().cloned().or_else(|| item.id.clone());
    let creator = item.creator.first().cloned();
    let institution = item.data_provider.first().cloned();
    let attribution = match (creator.as_deref(), institution.as_deref()) {
        (Some(creator), Some(institution)) => Some(format!("{creator} — {institution}")),
        (Some(creator), None) => Some(creator.to_owned()),
        (None, Some(institution)) => Some(institution.to_owned()),
        (None, None) => None,
    };

    Some(ImageResult {
        image_url,
        thumbnail_url,
        title,
        source_page: item.shown_at.first().cloned().or(item.link),
        provider: "Europeana".into(),
        width: None,
        height: None,
        creator,
        license: Some(license),
        license_url: Some(rights_url.to_owned()),
        attribution,
    })
}

fn allowed_license_label(rights_url: &str) -> Option<String> {
    let rights = rights_url.to_ascii_lowercase();

    if rights.contains("creativecommons.org/publicdomain/zero/") {
        Some("CC0".into())
    } else if rights.contains("creativecommons.org/publicdomain/mark/") {
        Some("Domínio Público (PDM)".into())
    } else if rights.contains("creativecommons.org/licenses/by-sa/") {
        Some("CC BY-SA".into())
    } else if rights.contains("creativecommons.org/licenses/by/") {
        Some("CC BY".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::allowed_license_label;

    #[test]
    fn accepts_only_europeana_open_reuse_rights() {
        assert_eq!(
            allowed_license_label("http://creativecommons.org/publicdomain/zero/1.0/").as_deref(),
            Some("CC0")
        );
        assert_eq!(
            allowed_license_label("http://creativecommons.org/publicdomain/mark/1.0/").as_deref(),
            Some("Domínio Público (PDM)")
        );
        assert_eq!(
            allowed_license_label("http://creativecommons.org/licenses/by/4.0/").as_deref(),
            Some("CC BY")
        );
        assert_eq!(
            allowed_license_label("http://creativecommons.org/licenses/by-sa/4.0/").as_deref(),
            Some("CC BY-SA")
        );
        assert!(allowed_license_label("http://creativecommons.org/licenses/by-nc/4.0/").is_none());
        assert!(allowed_license_label("http://creativecommons.org/licenses/by-nd/4.0/").is_none());
    }
}
