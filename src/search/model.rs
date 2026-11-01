/// An image result normalized across all search providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageResult {
    pub image_url: String,
    pub thumbnail_url: Option<String>,
    pub title: Option<String>,
    pub source_page: Option<String>,
    pub provider: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Name of the creator/artist when supplied by the source.
    pub creator: Option<String>,
    /// Human-readable reuse license (for example CC0 or CC BY 4.0).
    pub license: Option<String>,
    /// Canonical URL describing the license or rights statement.
    pub license_url: Option<String>,
    /// Plain-text attribution guidance supplied or assembled from source metadata.
    pub attribution: Option<String>,
}
