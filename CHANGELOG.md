# Changelog

## [0.1.0] - Unreleased

### Added
- Initial Rust and Slint project structure.
- First native desktop interface draft.
- Normalized image result model.
- Initial architecture documentation.
- Experimental DuckDuckGo Images adapter using a session VQD token and JSON endpoint.
- Experimental Startpage Images adapter parsing the image-search HTML page.
- Wikimedia Commons search adapter using the documented MediaWiki API.
- Brave Images Search adapter using the official API and environment-based token.
- Visual gallery with downloaded thumbnails, title, provider and dimensions.
- Concurrent thumbnail downloads, limited to three requests at a time, following Wikimedia usage guidance.
- Search status with counts and details for thumbnail download or decoding failures.

### Changed
- Brave Search API is the preferred image provider when `BRAVE_SEARCH_API_KEY` is configured.
- Wikimedia Commons is used when Brave is not configured, fails, or returns no results.
- Updated provider labels and footer in the Slint interface.
- Wikimedia Commons API page results are parsed as an ID-keyed object, matching the MediaWiki response format.

### Notes
- Brave Image Search requires an API key sent in the `X-Subscription-Token` header. The key is read from the `BRAVE_SEARCH_API_KEY` environment variable and must not be committed.
- DuckDuckGo's undocumented image endpoint returned HTTP 403 and is not used by default.
- Startpage's HTML structure is not a stable public API and may change without notice.
- Openverse remains available in the code but is not used by the default search flow.
- Filters, deduplication, full-size preview, download and favorite actions remain pending.
