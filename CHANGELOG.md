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

### Changed
- Wikimedia Commons is the only integrated search provider in the default flow.
- Removed an API-based search adapter that required paid credentials, along with its key handling, dependency, documentation and UI references.
- Updated provider labels and footer in the Slint interface.
- Wikimedia Commons API page results are parsed as an ID-keyed object, matching the MediaWiki response format.

### Notes
- Yandex Images has an official API, but image-search requests are billed (currently listed at approximately US$ 7.50 per 1,000 for USD-billed customers); it is excluded to preserve the project's free-only requirement.
- No Yandex HTML scraping adapter is included because the service may restrict automated requests under its user agreement.
- DuckDuckGo's undocumented image endpoint returned HTTP 403 and is not used by default.
- Startpage's HTML structure is not a stable public API and may change without notice.
- Openverse remains available in the code but is not used by the default search flow.
- Filters, deduplication, full-size preview, download and favorite actions remain pending.
