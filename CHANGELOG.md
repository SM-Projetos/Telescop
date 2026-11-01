# Changelog

## [0.1.0] - Unreleased

### Added
- SearXNG image-search adapter using the instance Search API.
- HTML parsing fallback for SearXNG instances that disable JSON output.
- Configuration of the SearXNG base URL through SEARXNG_INSTANCE_URL.
- Unit tests for SearXNG URL construction, JSON normalization, HTML parsing, URL validation and image dimensions.

### Changed
- Make SearXNG the only search mechanism integrated into Telescop.
- Remove the Wikimedia Commons, Openverse, Europeana, Cleveland Museum of Art, DuckDuckGo and Startpage adapters and their UI controls.
- Keep the existing gallery, asynchronous image processing and three-download concurrency limit.
- Mark SearXNG image results as having unverified reuse rights unless reliable license metadata is supplied.
- Replace provider-specific setup and documentation with SearXNG instance configuration and privacy guidance.

### Notes
- Many public SearXNG instances disable JSON output. Telescop therefore falls back to parsing the HTML image-results page when needed.
- Search terms are sent to the configured SearXNG instance and may be forwarded to the external search engines enabled by that instance's operator.
- SearXNG image results do not uniformly provide trustworthy license metadata. Verify rights on the original source page before reuse.
- Search filters, deduplication, full-size preview, download and favorite actions remain pending.
