use scraper::Html;
use serde::Deserialize;
use std::time::Duration;

use super::ImageResult;

const API_URL: &str = "https://api.openverse.org/v1/images/";
const ALLOWED_LICENSES: &str = "cc0,by,by-sa,pdm";

#[derive(Debug, Deserialize)]
struct OpenverseResponse {
    #[serde(default)]
    results: Vec<OpenverseImage>,
}

#[derive(Debug, Deserialize)]
struct OpenverseImage {
    #[serde(default)]
    url: String,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    foreign_landing_url: Option<String>,
    #[serde(default)]
    creator: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    license_version: Option<String>,
    #[serde(default)]
    license_url: Option<String>,
    #[serde(default)]
    attribution: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

/// Search Openverse, accepting only licenses that allow commercial reuse and adaptations.
///
/// Openverse documents free anonymous API access. The explicit license allow-list
/// excludes non-commercial and no-derivatives licenses.
pub async fn search_images(
    query: &str,
    limit: u8,
) -> Result<Vec<ImageResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "Telescop/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/SM-Projetos/Telescop)"
        ))
        .timeout(Duration::from_secs(20))
        .build()?;

    let page_size = limit.clamp(1, 20).to_string();
    let response = client
        .get(API_URL)
        .query(&[
            ("q", query),
            ("page_size", page_size.as_str()),
            ("license", ALLOWED_LICENSES),
            ("filter_dead", "true"),
            ("mature", "false"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<OpenverseResponse>()
        .await?;

    Ok(response
        .results
        .into_iter()
        .filter_map(to_image_result)
        .take(limit.clamp(1, 20) as usize)
        .collect())
}

fn to_image_result(image: OpenverseImage) -> Option<ImageResult> {
    let license = image.license.as_deref()?;
    let license_label = allowed_license_label(license, image.license_version.as_deref())?;
    if image.url.trim().is_empty() {
        return None;
    }

    let creator = image.creator.filter(|value| !value.trim().is_empty());
    let attribution = image
        .attribution
        .map(|value| plain_text(&value))
        .filter(|value| !value.is_empty());

    Some(ImageResult {
        image_url: image.url,
        thumbnail_url: image.thumbnail,
        title: image.title,
        source_page: image.foreign_landing_url,
        provider: "Openverse".into(),
        width: image.width,
        height: image.height,
        creator,
        license: Some(license_label),
        license_url: image.license_url,
        attribution,
    })
}

fn allowed_license_label(license: &str, version: Option<&str>) -> Option<String> {
    let label = match license.to_ascii_lowercase().as_str() {
        "cc0" => "CC0",
        "pdm" => "Domínio Público (PDM)",
        "by" => "CC BY",
        "by-sa" => "CC BY-SA",
        _ => return None,
    };

    Some(match version.filter(|value| !value.trim().is_empty()) {
        Some(version) => format!("{label} {version}"),
        None => label.to_owned(),
    })
}

fn plain_text(value: &str) -> String {
    Html::parse_fragment(value)
        .root_element()
        .text()
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::allowed_license_label;

    #[test]
    fn accepts_only_commercial_and_adaptation_friendly_licenses() {
        assert_eq!(allowed_license_label("cc0", Some("1.0")).as_deref(), Some("CC0 1.0"));
        assert_eq!(allowed_license_label("by", Some("4.0")).as_deref(), Some("CC BY 4.0"));
        assert_eq!(allowed_license_label("by-sa", Some("4.0")).as_deref(), Some("CC BY-SA 4.0"));
        assert_eq!(allowed_license_label("pdm", None).as_deref(), Some("Domínio Público (PDM)"));
        assert!(allowed_license_label("by-nc", Some("4.0")).is_none());
        assert!(allowed_license_label("by-nd", Some("4.0")).is_none());
    }
}
