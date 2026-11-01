use std::cell::Cell;

use slint::ComponentHandle;
use telescop::{MainWindow, WebviewLogic, webview::WebView};

fn main() -> Result<(), slint::PlatformError> {
    setup_slint_with_wgpu();

    let app = MainWindow::new()?;

    // Servo must be initialized after Slint has created its WGPU device.
    let initialized = Cell::new(false);
    let app_weak = app.as_weak();

    app.window()
        .set_rendering_notifier(move |state, graphics_api| {
            if !matches!(state, slint::RenderingState::RenderingSetup) || initialized.get() {
                return;
            }

            let slint::GraphicsAPI::WGPU30 { device, .. } = graphics_api else {
                panic!(
                    "Slint did not select a wgpu-30 renderer; \
                     ensure the WGPU 3.0 backend is enabled"
                );
            };

            let app = app_weak.upgrade().expect("Application closed before Servo initialization");
            WebView::new(
                app,
                "https://duckduckgo.com/?ia=images&iax=images&kl=br-pt".into(),
                device.clone(),
            );
            initialized.set(true);
        })
        .expect("Failed to install Slint rendering notifier");

    let app_weak = app.as_weak();
    app.on_search_requested(move |query| {
        let query = query.trim();
        if query.is_empty() {
            return;
        }

        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let url = duckduckgo_images_url(query);
        app.global::<WebviewLogic>().invoke_loadUrl(url.into());
    });

    app.run()
}

fn duckduckgo_images_url(query: &str) -> String {
    let mut parameters = url::form_urlencoded::Serializer::new(String::new());
    parameters
        .append_pair("q", query)
        .append_pair("ia", "images")
        .append_pair("iax", "images")
        .append_pair("kl", "br-pt");

    format!("https://duckduckgo.com/?{}", parameters.finish())
}

fn setup_slint_with_wgpu() {
    use slint::wgpu_30::{WGPUConfiguration, WGPUSettings};

    let settings = WGPUSettings::default();

    slint::BackendSelector::new()
        .require_wgpu_30(WGPUConfiguration::Automatic(settings))
        .select()
        .expect(
            "Failed to create Slint WGPU backend. Check that the system has a supported GPU \
             driver and Vulkan/OpenGL runtime.",
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_duckduckgo_images_url_from_current_query() {
        let url = url::Url::parse(&duckduckgo_images_url("carro de corrida")).unwrap();
        let parameters = url.query_pairs().collect::<std::collections::HashMap<_, _>>();

        assert_eq!(url.host_str(), Some("duckduckgo.com"));
        assert_eq!(parameters.get("q").map(|value| value.as_ref()), Some("carro de corrida"));
        assert_eq!(parameters.get("ia").map(|value| value.as_ref()), Some("images"));
        assert_eq!(parameters.get("iax").map(|value| value.as_ref()), Some("images"));
        assert_eq!(parameters.get("kl").map(|value| value.as_ref()), Some("br-pt"));
    }

    #[test]
    fn safely_encodes_special_characters_and_accents() {
        let url = url::Url::parse(&duckduckgo_images_url("árvore & céu")).unwrap();
        let query = url.query_pairs().find(|(key, _)| key == "q").unwrap();

        assert_eq!(query.1, "árvore & céu");
    }
}
