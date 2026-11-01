# Changelog

## [0.1.0] - Unreleased

### Added
- Servo diagnostics for page load states, JavaScript console/errors, animation state and DuckDuckGo image DOM counts.
- Hide the DuckDuckGo page search field inside Telescop while preserving the site's logo and navigation options.
- Experimental embedded Servo web engine integrated into the Slint window through WGPU 3.0.
- DuckDuckGo Images URL builder that safely encodes the user's current query and applies the Brazilian Portuguese region.
- Embedded browser controls for back, forward and reload.
- Mouse, keyboard, touch and scroll event forwarding from the Slint surface to Servo.
- Unit tests for DuckDuckGo image-search URL construction and query encoding.

### Changed
- Restrict the embedded rendering implementation to Linux, matching the available Vulkan backend.
- Reduce the direct dependency set, disable image crate default features, and remove unused cross-platform build branches.
- Simplify GPU rendering setup, remove unused browser URL/mouse state, and parse the generated DuckDuckGo navigation URL directly.
- Replace the SearXNG search adapter with the real DuckDuckGo Images website rendered inside Telescop.
- Replace the image gallery with an embedded browser surface.
- Remove the SearXNG instance configuration and HTTP/JSON/HTML search implementation.
- Pin Slint to the revision used by the official Servo integration example and enable its WGPU 3.0 backend.
- Update README and architecture documentation for the experimental browser-based workflow.

### Removed
- Unused direct dependencies: gl, log, spin_on, rustls, time-now and termcolor.
- Unused Google search fallback for arbitrary browser input and unused URL/mouse UI properties.
- SearXNG adapter and normalized image-search model.
- Search-result extraction, thumbnail downloads and gallery-only dependencies.
- The SEARXNG_INSTANCE_URL runtime requirement.

### Notes
- Servo + Slint integration is experimental and not production-ready.
- The DuckDuckGo website is loaded directly; Telescop does not use DuckDuckGo's image API or undocumented image endpoint.
- Web compatibility depends on Servo's current implementation and may differ from Firefox or Chromium.
- The initial browser prototype does not include tabs, persistent history, bookmarks, downloads or dedicated cookie/privacy controls.
- Servo substantially increases build time, disk use and native build requirements compared with the previous application.
