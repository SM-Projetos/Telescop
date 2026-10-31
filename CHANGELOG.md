# Changelog

## [0.1.0] - Unreleased

### Added
- Initial Rust and Slint project structure.
- First native desktop interface draft.
- Normalized image result model.
- Initial architecture documentation.
- Experimental DuckDuckGo Images adapter using the session VQD token and JSON endpoint.\n- Experimental Startpage Images adapter parsing the image-search HTML page.\n- Concurrent searches across DuckDuckGo and Startpage.
- Basic search status and textual result list in the interface.

### Notes
- DuckDuckGo and Startpage use undocumented/unstable search interfaces; blocking and markup changes may break them.\n- Openverse remains available in the code but is not used by the default search flow.\n- Visual thumbnails, effective filters and deduplication are not implemented yet.
