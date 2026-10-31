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
}
