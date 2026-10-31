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
- DuckDuckGo and Startpage use undocumented/unstable search interfaces; blocking and markup changes may break them.\n- Openverse remains available in the code but is not used by the default search flow.\n- Added a first visual gallery with downloaded thumbnails, title, provider and dimensions. Up to eight thumbnail requests run concurrently.\n- Failed thumbnail downloads or unsupported image formats are skipped. Filters, deduplication, full-size preview and save/favorite actions remain pending.
