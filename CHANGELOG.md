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
- Visual gallery with downloaded thumbnails, title, provider and dimensions.
- Concurrent thumbnail downloads, limited to three requests at a time, following Wikimedia usage guidance.
- Search status with counts and details for thumbnail download or decoding failures.
- Button to open the current query in Bing Images using the system's default browser.

### Changed
- Wikimedia Commons is the only integrated search provider in the default flow.
- Removed the Brave Search API adapter, API-key handling, documentation and UI references.
- Bing Images is offered as an external browser search, not as an internal result provider.
- Updated provider labels and footer in the Slint interface.
- Wikimedia Commons API page results are parsed as an ID-keyed object, matching the MediaWiki response format.

### Notes
- Microsoft retired the Bing Search APIs, including Image Search, on August 11, 2025; no new customer signup is available.
- DuckDuckGo's undocumented image endpoint returned HTTP 403 and is not used by default.
- Startpage's HTML structure is not a stable public API and may change without notice.
- Openverse remains available in the code but is not used by the default search flow.
- Filters, deduplication, full-size preview, download and favorite actions remain pending.
