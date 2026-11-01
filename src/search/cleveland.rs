use serde::Deserialize;
use std::time::Duration;

use super::ImageResult;

const API_URL: &str = "https://openaccess-api.clevelandart.org/api/artworks/";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    data: Vec<Artwork>,
}

#[derive(Debug, Deserialize)]
struct Artwork {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    share_license_status: Option<String>,
    #[serde(default)]
    creators: Vec<Creator>,
    #[serde(default)]
    images: Option<ImageSet>,
}

#[derive(Debug, Deserialize)]
struct Creator {
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageSet {
    #[serde(default)]
    web: Option<ImageAsset>,
}

#[derive(Debug, Deserialize)]
struct ImageAsset {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

/// Search only Cleveland Museum of Art works whose image rights are explicitly CC0.
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

    let limit = limit.clamp(1, 50).to_string();
    let response = client
        .get(API_URL)
        .query(&[
            ("q", query),
            ("cc0", "1"),
            ("has_image", "1"),
            ("limit", limit.as_str()),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse>()
        .await?;

    Ok(response
        .data
        .into_iter()
        .filter_map(to_image_result)
        .take(limit.parse::<usize>().unwrap_or(50))
        .collect())
}

fn to_image_result(artwork: Artwork) -> Option<ImageResult> {
    // Defense in depth: never trust the search filter alone for reuse rights.
    if !artwork
        .share_license_status
        .as_deref()
        .is_some_and(|status| status.eq_ignore_ascii_case("CC0"))
    {
        return None;
    }

    let image = artwork.images?.web?;
    let image_url = image.url.filter(|url| url.starts_with("https://"))?;
    let title = artwork.title.filter(|value| !value.trim().is_empty());
    let creator = artwork
        .creators
        .into_iter()
        .filter_map(|creator| creator.description)
        .find(|value| !value.trim().is_empty());
    let attribution = Some(match creator.as_deref() {
        Some(creator) => format!(
            "{creator}. {}. Cleveland Museum of Art.",
            title.as_deref().unwrap_or("")
        ),
        None => format!(
            "{}. Cleveland Museum of Art.",
            title.as_deref().unwrap_or("Artwork")
        ),
    });

    Some(ImageResult {
        image_url: image_url.clone(),
        thumbnail_url: Some(image_url),
        title,
        source_page: artwork.url,
        provider: "Cleveland Museum of Art".into(),
        width: image.width,
        height: image.height,
        creator,
        license: Some("CC0".into()),
        license_url: Some("https://creativecommons.org/publicdomain/zero/1.0/".into()),
        attribution,
    })
}

#[cfg(test)]
mod tests {
    use super::{Artwork, Creator, ImageAsset, ImageSet, to_image_result};

    #[test]
    fn only_returns_records_with_cc0_web_images() {
        let permitted = Artwork {
            title: Some("Open artwork".into()),
            url: Some("https://clevelandart.org/art/example".into()),
            share_license_status: Some("CC0".into()),
            creators: vec![Creator {
                description: Some("Artist".into()),
            }],
            images: Some(ImageSet {
                web: Some(ImageAsset {
                    url: Some("https://openaccess-cdn.clevelandart.org/example.jpg".into()),
                    width: Some(900),
                    height: Some(600),
                }),
            }),
        };
        assert!(to_image_result(permitted).is_some());

        let restricted = Artwork {
            title: Some("Restricted artwork".into()),
            url: None,
            share_license_status: Some("Copyrighted".into()),
            creators: Vec::new(),
            images: None,
        };
        assert!(to_image_result(restricted).is_none());
    }
}
