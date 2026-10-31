slint::include_modules!();

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
        if query.is_empty() {
            return;
        }

        let Some(app) = app_handle.upgrade() else {
            return;
        };

        app.set_status("Pesquisando no Openverse...".into());
        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
            Vec::<slint::SharedString>::new(),
        ))));

        let app_handle = app.as_weak();
        runtime_handle.spawn(async move {
            let result = telescop::search::openverse::search_images(&query, 30).await;

            let _ = slint::invoke_from_event_loop(move || {
                let Some(app) = app_handle.upgrade() else {
                    return;
                };

                match result {
                    Ok(images) => {
                        let titles = images
                            .iter()
                            .map(|image| {
                                let title = image.title.as_deref().unwrap_or("Imagem sem título");
                                let dimensions = match (image.width, image.height) {
                                    (Some(width), Some(height)) => format!(" • {width} × {height}"),
                                    _ => String::new(),
                                };
                                format!("{title}{dimensions} • Openverse")
                            })
                            .map(slint::SharedString::from)
                            .collect::<Vec<_>>();

                        let count = titles.len();
                        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(titles))));
                        app.set_status(format!("{count} resultado(s) encontrado(s) no Openverse.").into());
                    }
                    Err(error) => {
                        app.set_status(format!("Falha na pesquisa: {error}").into());
                    }
                }
            });
        });
    });

    app.run()
}
