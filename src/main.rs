slint::include_modules!();

use futures_util::stream::{self, StreamExt};
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Não foi possível iniciar o runtime assíncrono");

    let app = MainWindow::new()?;
    let app_handle = app.as_weak();
    let runtime_handle = runtime.handle().clone();

    app.on_search_requested(move |query| {
        let query = query.trim().to_owned();
        if query.is_empty() { return; }
        let Some(app) = app_handle.upgrade() else { return };

        app.set_status("Pesquisando imagens e carregando miniaturas...".into());
        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
            Vec::<GalleryItem>::new(),
        ))));

        let app_handle = app.as_weak();
        runtime_handle.spawn(async move {
            let (ddg, startpage) = tokio::join!(
                telescop::search::duckduckgo::search_images(&query, 30),
                telescop::search::startpage::search_images(&query, 30)
            );

            let mut images = Vec::new();
            let mut errors = Vec::new();
            match ddg {
                Ok(mut found) => images.append(&mut found),
                Err(error) => errors.push(format!("DuckDuckGo: {error}")),
            }
            match startpage {
                Ok(mut found) => images.append(&mut found),
                Err(error) => errors.push(format!("Startpage: {error}")),
            }

            let client = match reqwest::Client::builder()
                .user_agent("Telescop/0.1.0")
                .timeout(std::time::Duration::from_secs(15))
                .build() {
                Ok(client) => client,
                Err(error) => {
                    let message = format!("Não foi possível preparar o carregamento das miniaturas: {error}");
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_handle.upgrade() { app.set_status(message.into()); }
                    });
                    return;
                }
            };

            let cards = stream::iter(images.into_iter().map(|item| {
                let client = client.clone();
                async move {
                    let url = item.thumbnail_url.as_deref().unwrap_or(&item.image_url);
                    let response = client.get(url).send().await.ok()?.error_for_status().ok()?;
                    let bytes = response.bytes().await.ok()?;
                    let decoded = image::load_from_memory(&bytes).ok()?;
                    let (width, height) = (decoded.width(), decoded.height());
                    let thumbnail = decoded.thumbnail(320, 220).to_rgba8();
                    let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                        thumbnail.as_raw(), thumbnail.width(), thumbnail.height()
                    );
                    let dimensions = match (item.width, item.height) {
                        (Some(w), Some(h)) => format!("{w} × {h}"),
                        _ if width > 0 && height > 0 => format!("{width} × {height}"),
                        _ => String::new(),
                    };
                    Some(GalleryItem {
                        thumbnail: slint::Image::from_rgba8(buffer),
                        title: item.title.unwrap_or_else(|| "Imagem sem título".into()).into(),
                        provider: item.provider.into(),
                        dimensions: dimensions.into(),
                    })
                }
            })).buffer_unordered(8).filter_map(|card| async move { card }).collect::<Vec<_>>().await;

            let _ = slint::invoke_from_event_loop(move || {
                let Some(app) = app_handle.upgrade() else { return };
                if cards.is_empty() {
                    let detail = if errors.is_empty() {
                        "Nenhuma miniatura pôde ser carregada. Os provedores podem ter bloqueado as imagens.".to_owned()
                    } else { errors.join(" | ") };
                    app.set_status(format!("Pesquisa concluída sem miniaturas: {detail}").into());
                    return;
                }

                let count = cards.len();
                app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(cards))));
                let status = if errors.is_empty() {
                    format!("{count} imagem(ns) carregada(s) na galeria.")
                } else {
                    format!("{count} imagem(ns) carregada(s). Falhas: {}", errors.join(" | "))
                };
                app.set_status(status.into());
            });
        });
    });

    app.run()
}
