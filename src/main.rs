slint::include_modules!();

use futures_util::stream::{self, StreamExt};
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Não foi possível iniciar o runtime assíncrono");

    let app = MainWindow::new()?;
    // Europeana starts enabled only when the user has configured a non-empty key.
    let europeana_key_configured = matches!(
        std::env::var("EUROPEANA_API_KEY"),
        Ok(key) if !key.trim().is_empty()
    );
    app.set_use_europeana(europeana_key_configured);
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

        let use_commons = app.get_use_commons();
        let use_openverse = app.get_use_openverse();
        let use_europeana = app.get_use_europeana();
        let use_cleveland = app.get_use_cleveland();
        let use_duckduckgo = app.get_use_duckduckgo();

        if !use_commons && !use_openverse && !use_europeana && !use_cleveland && !use_duckduckgo {
            app.set_status("Selecione pelo menos um buscador para pesquisar.".into());
            return;
        }

        app.set_status("Pesquisando nas fontes selecionadas e carregando miniaturas...".into());
        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
            Vec::<GalleryItem>::new(),
        ))));

        let app_handle = app.as_weak();
        runtime_handle.spawn(async move {
            // Disabled providers are not called. Enabled providers run concurrently.
            let commons_search = async {
                if use_commons {
                    Some(telescop::search::commons::search_images(&query, 30).await)
                } else {
                    None
                }
            };
            let openverse_search = async {
                if use_openverse {
                    Some(telescop::search::openverse::search_images(&query, 30).await)
                } else {
                    None
                }
            };
            let europeana_search = async {
                if use_europeana {
                    Some(telescop::search::europeana::search_images(&query, 30).await)
                } else {
                    None
                }
            };
            let cleveland_search = async {
                if use_cleveland {
                    Some(telescop::search::cleveland::search_images(&query, 30).await)
                } else {
                    None
                }
            };
            let duckduckgo_search = async {
                if use_duckduckgo {
                    Some(telescop::search::duckduckgo::search_images(&query, 30).await)
                } else {
                    None
                }
            };

            let (commons, openverse, europeana, cleveland, duckduckgo) = tokio::join!(
                commons_search,
                openverse_search,
                europeana_search,
                cleveland_search,
                duckduckgo_search,
            );

            let mut images = Vec::new();
            let mut search_errors = Vec::new();

            for (provider, result) in [
                ("Wikimedia Commons", commons),
                ("Openverse", openverse),
                ("Europeana", europeana),
                ("Cleveland Museum of Art", cleveland),
                ("DuckDuckGo", duckduckgo),
            ] {
                let Some(result) = result else {
                    continue;
                };
                match result {
                    Ok(mut found) => images.append(&mut found),
                    Err(error) => search_errors.push(format!("{provider}: {error}")),
                }
            }
            let found_count = images.len();

            if found_count == 0 && !search_errors.is_empty() {
                let message = format!(
                    "Nenhum provedor retornou resultados. {}",
                    search_errors.join(" | ")
                );
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

                    let rights_verified = item.license.as_deref()
                        .is_some_and(|value| !value.trim().is_empty());
                    let license = item.license.unwrap_or_else(|| {
                        if provider == "DuckDuckGo" {
                            "Licença não verificada".into()
                        } else {
                            "Licença não informada".into()
                        }
                    });
                    let credit = item.attribution.or(item.creator)
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
                    .map(|(pixels, width, height, title, provider, dimensions, rights, rights_verified)| {
                        let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
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
                if !search_errors.is_empty() {
                    status.push_str(" Fontes com falha: ");
                    status.push_str(&search_errors.join(" | "));
                }
                app.set_status(status.into());
            });
        });
    });

    app.run()
}
