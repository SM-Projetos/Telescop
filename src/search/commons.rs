use scraper::Html;
use serde::Deserialize;
use std::collections::HashMap;

use super::ImageResult;

const API_URL: &str = "https://commons.wikimedia.org/w/api.php";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    query: Query,
    error: Option<ApiError>,
}

#[derive(Debug, Default, Deserialize)]
struct Query {
    // MediaWiki returns query.pages as an object keyed by page ID, not an array.
    #[serde(default)]
    pages: HashMap<String, Page>,
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
    #[serde(default)]
    extmetadata: HashMap<String, MetadataValue>,
}

#[derive(Debug, Deserialize)]
struct MetadataValue {
    #[serde(default)]
    value: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    info: String,
}

/// Search Wikimedia Commons through its documented MediaWiki API.
/// Results are retained only when their metadata identifies CC0, Public Domain,
/// CC BY or CC BY-SA reuse rights.
pub async fn search_images(
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Telescop/0.1.0 (https://github.com/SM-Projetos/Telescop)")
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let limit = limit.clamp(1, 50).to_string();

    let response = client
        .get(API_URL)
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("generator", "search"),
            ("gsrnamespace", "6"),
            ("gsrsearch", query),
            ("gsrlimit", limit.as_str()),
            ("prop", "imageinfo"),
            ("iiprop", "url|size|extmetadata"),
            ("iiurlwidth", "640"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse>()
        .await?;

    if let Some(error) = response.error {
        return Err(std::io::Error::other(format!(
            "Wikimedia Commons API ({}): {}",
            error.code, error.info
        )).into());
    }

    let mut pages = response.query.pages.into_values().collect::<Vec<_>>();
    pages.sort_by(|a, b| a.title.cmp(&b.title));

    Ok(pages
        .into_iter()
        .filter_map(|page| {
            let info = page.imageinfo.into_iter().next()?;
            let license_short = metadata_value(&info.extmetadata, "LicenseShortName");
            let license_url = metadata_value(&info.extmetadata, "LicenseUrl");
            let license = allowed_license_label(
                license_short.as_deref(),
                license_url.as_deref(),
            )?;

            let creator = metadata_value(&info.extmetadata, "Artist");
            let attribution = metadata_value(&info.extmetadata, "Attribution")
                .or_else(|| creator.clone());

            Some(ImageResult {
                image_url: info.url,
                thumbnail_url: info.thumburl,
                title: Some(page.title.trim_start_matches("File:").to_owned()),
                source_page: info.descriptionurl,
                provider: "Wikimedia Commons".into(),
                width: info.width,
                height: info.height,
                creator,
                license: Some(license),
                license_url,
                attribution,
            })
        })
        .collect())
}

fn allowed_license_label(short_name: Option<&str>, license_url: Option<&str>) -> Option<String> {
    let url = license_url.unwrap_or_default().to_ascii_lowercase();
    let name = short_name.unwrap_or_default().trim();

    if url.contains("creativecommons.org/publicdomain/zero/") {
        Some("CC0".into())
    } else if url.contains("creativecommons.org/publicdomain/mark/") {
        Some("Domínio Público (PDM)".into())
    } else if url.contains("creativecommons.org/licenses/by-sa/") {
        Some(name_if_cc(name, "CC BY-SA"))
    } else if url.contains("creativecommons.org/licenses/by/") {
        Some(name_if_cc(name, "CC BY"))
    } else if name.eq_ignore_ascii_case("Public domain")
        || name.to_ascii_lowercase().starts_with("public domain ")
    {
        Some("Domínio Público".into())
    } else {
        None
    }
}

fn name_if_cc(name: &str, fallback: &str) -> String {
    if name.is_empty() {
        fallback.to_owned()
    } else {
        name.to_owned()
    }
}

fn metadata_value(
    metadata: &HashMap<String, MetadataValue>,
    key: &str,
) -> Option<String> {
    let value = metadata.get(key)?.value.trim();
    let plain_text = Html::parse_fragment(value)
        .root_element()
        .text()
        .collect::<String>();
    let plain_text = plain_text.trim();

    if plain_text.is_empty() {
        None
    } else {
        Some(plain_text.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::allowed_license_label;

    #[test]
    fn accepts_only_public_domain_and_reusable_creative_commons_licenses() {
        assert_eq!(
            allowed_license_label(None, Some("https://creativecommons.org/publicdomain/zero/1.0/")).as_deref(),
            Some("CC0")
        );
        assert_eq!(
            allowed_license_label(None, Some("https://creativecommons.org/publicdomain/mark/1.0/")).as_deref(),
            Some("Domínio Público (PDM)")
        );
        assert_eq!(
            allowed_license_label(Some("CC BY 4.0"), Some("https://creativecommons.org/licenses/by/4.0/")).as_deref(),
            Some("CC BY 4.0")
        );
        assert_eq!(
            allowed_license_label(Some("CC BY-SA 4.0"), Some("https://creativecommons.org/licenses/by-sa/4.0/")).as_deref(),
            Some("CC BY-SA 4.0")
        );
        assert!(allowed_license_label(Some("CC BY-NC 4.0"), Some("https://creativecommons.org/licenses/by-nc/4.0/")).is_none());
        assert!(allowed_license_label(Some("CC BY-ND 4.0"), Some("https://creativecommons.org/licenses/by-nd/4.0/")).is_none());
        assert!(allowed_license_label(Some("unknown"), None).is_none());
    }
}
