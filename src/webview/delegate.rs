// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

use slint::ComponentHandle;
use std::rc::Rc;

use crate::{MainWindow, webview::SlintServoAdapter};
use servo::{ConsoleLogLevel, LoadStatus, WebView, WebViewDelegate};

const DUCKDUCKGO_PAGE_SETUP_SCRIPT: &str = r##"
(() => {
    if (!/(^|\.)duckduckgo\.com$/i.test(location.hostname)) {
        return "not a DuckDuckGo page";
    }

    if (!window.__telescopDiagnosticsInstalled) {
        window.__telescopDiagnosticsInstalled = true;

        window.addEventListener("error", (event) => {
            const detail = event.error?.stack || event.message || "Unknown JavaScript error";
            console.error("[Telescop][window.error]", detail);
        });

        window.addEventListener("unhandledrejection", (event) => {
            const detail = event.reason?.stack || String(event.reason);
            console.error("[Telescop][unhandledrejection]", detail);
        });
    }

    if (!document.getElementById("telescop-hide-ddg-search")) {
        const style = document.createElement("style");
        style.id = "telescop-hide-ddg-search";
        style.textContent = [
            "#search_form { display: none !important; }",
            ".header__content.header__search { display: none !important; }"
        ].join("\n");
        (document.head || document.documentElement).appendChild(style);
    }

    return JSON.stringify({
        diagnosticsInstalled: true,
        searchStyleInstalled: Boolean(document.getElementById("telescop-hide-ddg-search"))
    });
})()
"##;

const DUCKDUCKGO_DOM_DIAGNOSTICS_SCRIPT: &str = r##"
(() => {
    if (!document.getElementById("telescop-hide-ddg-search")) {
        const style = document.createElement("style");
        style.id = "telescop-hide-ddg-search";
        style.textContent = [
            "#search_form { display: none !important; }",
            ".header__content.header__search { display: none !important; }"
        ].join("\n");
        (document.head || document.documentElement).appendChild(style);
    }

    const snapshot = () => {
        const images = Array.from(document.images);
        const imageTiles = Array.from(document.querySelectorAll("img.tile--img__img"));
        const searchForm = document.querySelector("#search_form");

        console.info("[Telescop][DOM] " + JSON.stringify({
            readyState: document.readyState,
            imageElements: images.length,
            loadedImageElements: images.filter((image) => image.complete && image.naturalWidth > 0).length,
            duckduckgoImageTiles: imageTiles.length,
            loadedDuckduckgoImageTiles: imageTiles.filter((image) => image.complete && image.naturalWidth > 0).length,
            imageSearchRootPresent: Boolean(document.querySelector("#image_results")),
            searchFormPresent: Boolean(searchForm),
            searchFormHidden: searchForm ? getComputedStyle(searchForm).display === "none" : null
        }));
    };

    snapshot();
    window.setTimeout(snapshot, 3000);
    return "DOM snapshots scheduled";
})()
"##;


/// Servo delegate for handling browser engine callbacks.
///
/// `AppDelegate` implements Servo's `WebViewDelegate` trait to receive notifications
/// about rendering events. It acts as a bridge, forwarding Servo's frame updates to
/// the Slint UI for display.
///
/// # Responsibilities
///
/// - Receives frame-ready notifications from Servo
/// - Triggers frame painting in Servo
/// - Updates the Slint UI with the latest rendered content
///
/// # Lifecycle
///
/// The delegate holds a weak reference to the Slint app to avoid circular references.
/// If the app is dropped, frame updates are silently ignored.
pub struct AppDelegate {
    /// Weak reference to the Slint application
    app: slint::Weak<MainWindow>,
    /// Reference to the Slint-Servo adapter for state access
    adapter: Rc<SlintServoAdapter>,
}

impl AppDelegate {
    /// Creates a new delegate instance.
    ///
    /// # Arguments
    ///
    /// * `app` - Weak reference to the Slint application
    /// * `adapter` - Reference to the Slint-Servo adapter
    pub fn new(app: &MainWindow, adapter: Rc<SlintServoAdapter>) -> Self {
        Self { app: app.as_weak(), adapter }
    }
}

impl WebViewDelegate for AppDelegate {
    /// Called by Servo when a new frame is ready to be displayed.
    /// Triggers painting and updates the Slint UI with the new frame.
    fn notify_new_frame_ready(&self, webview: WebView) {
        webview.paint();
        if let Some(app) = self.app.upgrade() {
            self.adapter.update_web_content_with_latest_frame(&app);
        }
    }

    fn notify_load_status_changed(&self, webview: WebView, status: LoadStatus) {
        let current_url = webview.url();
        let page = current_url
            .as_ref()
            .map(|url| format!("{}{}", url.host_str().unwrap_or("unknown"), url.path()))
            .unwrap_or_else(|| "<URL indisponível>".to_string());

        eprintln!("[SERVO][LOAD] status={status:?} page={page}");

        let is_duckduckgo = current_url
            .as_ref()
            .and_then(|url| url.domain())
            == Some("duckduckgo.com");

        if !is_duckduckgo {
            return;
        }

        let script = match status {
            LoadStatus::HeadParsed => DUCKDUCKGO_PAGE_SETUP_SCRIPT,
            LoadStatus::Complete => DUCKDUCKGO_DOM_DIAGNOSTICS_SCRIPT,
            LoadStatus::Started => return,
        };

        let label = match status {
            LoadStatus::HeadParsed => "page setup",
            LoadStatus::Complete => "DOM snapshot",
            LoadStatus::Started => unreachable!(),
        };

        webview.evaluate_javascript(script, move |result| match result {
            Ok(value) => eprintln!("[SERVO][EVAL][{label}] {value:?}"),
            Err(error) => eprintln!("[SERVO][EVAL][{label}][ERROR] {error:?}"),
        });
    }

    fn show_console_message(
        &self,
        _webview: WebView,
        level: ConsoleLogLevel,
        message: String,
    ) {
        eprintln!("[SERVO][JS][{level:?}] {message}");
    }

    fn notify_animating_changed(&self, _webview: WebView, animating: bool) {
        eprintln!("[SERVO][ANIMATION] active={animating}");
    }

    fn notify_crashed(&self, _webview: WebView, reason: String, backtrace: Option<String>) {
        eprintln!("[SERVO][CRASH] {reason}");
        if let Some(backtrace) = backtrace {
            eprintln!("[SERVO][CRASH][BACKTRACE]\n{backtrace}");
        }
    }
}
