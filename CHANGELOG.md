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
- Openverse image search with an explicit CC0, PDM, CC BY and CC BY-SA allow-list.
- Europeana Search API adapter with open-reuse filtering and a free personal API key read from EUROPEANA_API_KEY.
- Cleveland Museum of Art Open Access adapter restricted to records marked CC0.
- Visual gallery with downloaded thumbnails, title, provider, dimensions and displayed rights/attribution.
- Concurrent thumbnail downloads, limited to three requests at a time.
- Search status with counts and details for provider, download or decoding failures.
- Unit tests for reusable-license filtering in Openverse, Europeana, Wikimedia Commons and Cleveland Museum of Art.

### Changed
- Search Wikimedia Commons, Openverse, Europeana and Cleveland Museum of Art concurrently.
- Keep provider failures independent so results from successful providers can still be displayed.
- Wikimedia Commons results now require explicit reusable-rights metadata; non-commercial, no-derivatives and unknown licenses are excluded.
- Extended ImageResult with creator, license, license URL and attribution metadata.
- Display license and attribution alongside each gallery result.
- Updated README and architecture documentation for the four-provider search flow and the free-only reuse policy.

### Notes
- Openverse supports free anonymous API access but applies request and pagination limits.
- Europeana requires a free API key. Personal keys are for individual non-production use; request a project key before a public production distribution. Never embed personal credentials in source code.
- Cleveland Museum of Art provides CC0 open-access image data through a public API without a key.
- Europeana and Wikimedia results are accepted only when the returned rights statement matches the allow-list implemented by Telescop.
- DuckDuckGo's undocumented image endpoint returned HTTP 403 and is not used by default.
- Startpage's HTML structure is not a stable public API and may change without notice.
- Yandex and Brave require paid API access and remain excluded. Bing's official image-search API is retired.
- Filters, deduplication, full-size preview, download and favorite actions remain pending.
