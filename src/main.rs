slint::include_modules!();

use futures_util::stream::{self, StreamExt};
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Não foi possível iniciar o runtime assíncrono");

    let app = MainWindow::new()?;
    let searxng_instance_url =
        std::env::var("SEARXNG_INSTANCE_URL").unwrap_or_default().trim().to_owned();
    if searxng_instance_url.is_empty() {
        app.set_status(
            "Configure SEARXNG_INSTANCE_URL com o endereço-base da sua instância SearXNG."
                .into(),
        );
    }

    let app_handle = app.as_weak();
    let runtime_handle = runtime.handle().clone();

    app.on_search_requested(move |query| {
        let query = query.trim().to_owned();
        if query.is_empty() {
            return;
        }
        if searxng_instance_url.is_empty() {
            if let Some(app) = app_handle.upgrade() {
                app.set_status(
                    "SearXNG não configurado. Defina SEARXNG_INSTANCE_URL e reinicie o Telescop."
                        .into(),
                );
            }
            return;
        }
        let Some(app) = app_handle.upgrade() else {
            return;
        };

        app.set_status("Pesquisando imagens no SearXNG e carregando miniaturas...".into());
        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
            Vec::<GalleryItem>::new(),
        ))));

        let app_handle = app.as_weak();
        let instance_url = searxng_instance_url.clone();
        runtime_handle.spawn(async move {
            let mut images = Vec::new();
            let mut search_errors = Vec::new();

            match telescop::search::searxng::search_images(&instance_url, &query, 30).await {
                Ok(mut found) => images.append(&mut found),
                Err(error) => search_errors.push(format!("SearXNG: {error}")),
            }

            let found_count = images.len();
            if found_count == 0 && !search_errors.is_empty() {
                let message = format!("A pesquisa SearXNG falhou. {}", search_errors.join(" | "));
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = app_handle.upgrade() {
                        app.set_status(message.into());
                    }
                });
                return;
            }

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
                    let url = item
                        .thumbnail_url
                        .as_deref()
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

                    let rights_verified = item
                        .license
                        .as_deref()
                        .is_some_and(|value| !value.trim().is_empty());
                    let license = item
                        .license
                        .unwrap_or_else(|| "Licença não verificada".into());
                    let credit = item
                        .attribution
                        .or(item.creator)
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| "Consulte a página de origem".into());
                    let rights = format!("{license} • {credit}");

                    Some(Ok((
                        thumbnail.into_raw(),
                        thumb_width,
                        thumb_height,
                        title,
                        provider,
                        dimensions,
                        rights,
                        rights_verified,
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
                    let mut details = search_errors;
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
                    .map(
                        |(
                            pixels,
                            width,
                            height,
                            title,
                            provider,
                            dimensions,
                            rights,
                            rights_verified,
                        )| {
                            let buffer =
                                slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                                    &pixels, width, height,
                                );
                            GalleryItem {
                                thumbnail: slint::Image::from_rgba8(buffer),
                                title: title.into(),
                                provider: provider.into(),
                                dimensions: dimensions.into(),
                                rights: rights.into(),
                                rights_verified,
                            }
                        },
                    )
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
                if !search_errors.is_empty() {
                    status.push_str(" Fonte com falha: ");
                    status.push_str(&search_errors.join(" | "));
                }
                app.set_status(status.into());
            });
        });
    });

    app.run()
}
