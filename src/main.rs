slint::include_modules!();

use futures_util::stream::{self, StreamExt};
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Não foi possível iniciar o runtime assíncrono");

    // Keep the Brave API token out of source files and Git history.
    let brave_api_key = std::env::var("BRAVE_SEARCH_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty());

    let app = MainWindow::new()?;
    let app_handle = app.as_weak();
    let runtime_handle = runtime.handle().clone();

    app.on_search_requested(move |query| {
        let query = query.trim().to_owned();
        if query.is_empty() {
            return;
        }
        let Some(app) = app_handle.upgrade() else {
            return;
        };

        app.set_status("Pesquisando imagens e carregando miniaturas...".into());
        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
            Vec::<GalleryItem>::new(),
        ))));

        let app_handle = app.as_weak();
        let brave_api_key = brave_api_key.clone();
        runtime_handle.spawn(async move {
            // Brave is the preferred provider when a token is configured.
            // Wikimedia Commons remains the fallback and requires no API key.
            let mut images = Vec::new();
            let mut search_notes = Vec::new();

            if let Some(api_key) = brave_api_key {
                match telescop::search::brave::search_images(&query, &api_key, 30).await {
                    Ok(mut found) if !found.is_empty() => images.append(&mut found),
                    Ok(_) => search_notes.push(
                        "Brave Search não encontrou imagens; usando Wikimedia Commons.".to_owned()
                    ),
                    Err(error) => search_notes.push(format!(
                        "Brave Search falhou ({error}); usando Wikimedia Commons."
                    )),
                }
            } else {
                search_notes.push(
                    "Brave não configurado: defina BRAVE_SEARCH_API_KEY para ativá-lo. Usando Wikimedia Commons.".to_owned()
                );
            }

            if images.is_empty() {
                match telescop::search::commons::search_images(&query, 30).await {
                    Ok(mut found) => images.append(&mut found),
                    Err(error) => search_notes.push(format!("Wikimedia Commons: {error}")),
                }
            }
            let found_count = images.len();

            let client = match reqwest::Client::builder()
                .user_agent("Telescop/0.1.0 (https://github.com/SM-Projetos/Telescop)")
                .timeout(std::time::Duration::from_secs(20))
                .build()
            {
                Ok(client) => client,
                Err(error) => {
                    let message = format!(
                        "Não foi possível preparar o carregamento das miniaturas: {error}"
                    );
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = app_handle.upgrade() {
                            app.set_status(message.into());
                        }
                    });
                    return;
                }
            };

            let download_results = stream::iter(images.into_iter().map(|item| {
                let client = client.clone();
                async move {
                    let title = item.title.unwrap_or_else(|| "Imagem sem título".into());
                    let provider = item.provider;
                    let context = format!("{provider}: {title}");
                    let url = item.thumbnail_url.as_deref()
                        .unwrap_or(&item.image_url)
                        .to_owned();

                    let response = match client.get(&url).send().await {
                        Ok(response) => response,
                        Err(error) => return Some(Err(format!("{context}: {error}"))),
                    };
                    if !response.status().is_success() {
                        return Some(Err(format!("{context}: HTTP {}", response.status())));
                    }

                    let bytes = match response.bytes().await {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            return Some(Err(format!("{context}: falha ao ler imagem: {error}")));
                        }
                    };
                    let decoded = match image::load_from_memory(&bytes) {
                        Ok(decoded) => decoded,
                        Err(error) => {
                            return Some(Err(format!(
                                "{context}: formato de imagem não suportado: {error}"
                            )));
                        }
                    };

                    let (original_width, original_height) = (decoded.width(), decoded.height());
                    let thumbnail = decoded.thumbnail(320, 220).to_rgba8();
                    let (thumb_width, thumb_height) = thumbnail.dimensions();
                    let dimensions = match (item.width, item.height) {
                        (Some(width), Some(height)) => format!("{width} × {height}"),
                        _ => format!("{original_width} × {original_height}"),
                    };

                    Some(Ok((
                        thumbnail.into_raw(),
                        thumb_width,
                        thumb_height,
                        title,
                        provider,
                        dimensions,
                    )))
                }
            }))
            .buffer_unordered(3)
            .filter_map(|result| async move { result })
            .collect::<Vec<_>>()
            .await;

            let _ = slint::invoke_from_event_loop(move || {
                let Some(app) = app_handle.upgrade() else {
                    return;
                };

                let mut cards = Vec::new();
                let mut download_errors = Vec::new();
                for result in download_results {
                    match result {
                        Ok(card) => cards.push(card),
                        Err(error) => download_errors.push(error),
                    }
                }

                if cards.is_empty() {
                    let mut details = search_notes;
                    details.extend(download_errors.into_iter().take(3));
                    let base = if found_count == 0 {
                        "Nenhum resultado de imagem foi encontrado.".to_owned()
                    } else {
                        format!(
                            "{found_count} resultado(s) encontrados, mas nenhuma miniatura foi carregada."
                        )
                    };
                    let message = if details.is_empty() {
                        base
                    } else {
                        format!("{base} {}", details.join(" | "))
                    };
                    app.set_status(message.into());
                    return;
                }

                let gallery_items = cards
                    .into_iter()
                    .map(|(pixels, width, height, title, provider, dimensions)| {
                        let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                            &pixels, width, height,
                        );
                        GalleryItem {
                            thumbnail: slint::Image::from_rgba8(buffer),
                            title: title.into(),
                            provider: provider.into(),
                            dimensions: dimensions.into(),
                        }
                    })
                    .collect::<Vec<_>>();

                let loaded_count = gallery_items.len();
                app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
                    gallery_items,
                ))));

                let failed_count = download_errors.len();
                let mut status = if failed_count == 0 {
                    format!("{loaded_count} imagem(ns) carregada(s) na galeria.")
                } else {
                    let examples = download_errors.into_iter().take(2).collect::<Vec<_>>();
                    format!(
                        "{loaded_count} de {found_count} miniatura(s) carregada(s); {failed_count} falha(s). {}",
                        examples.join(" | ")
                    )
                };
                if !search_notes.is_empty() {
                    status.push_str(" ");
                    status.push_str(&search_notes.join(" | "));
                }
                app.set_status(status.into());
            });
        });
    });

    app.run()
}
