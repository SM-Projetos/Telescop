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
        if query.is_empty() { return; }
        let Some(app) = app_handle.upgrade() else { return };

        app.set_status("Pesquisando no DuckDuckGo e no Startpage...".into());
        app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(
            Vec::<slint::SharedString>::new(),
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

            let _ = slint::invoke_from_event_loop(move || {
                let Some(app) = app_handle.upgrade() else { return };
                if images.is_empty() {
                    let detail = if errors.is_empty() {
                        "Nenhuma imagem encontrada.".to_owned()
                    } else { errors.join(" | ") };
                    app.set_status(format!("Falha na pesquisa: {detail}").into());
                    return;
                }

                let results = images.iter().map(|image| {
                    let title = image.title.as_deref().unwrap_or("Imagem sem título");
                    let dimensions = match (image.width, image.height) {
                        (Some(w), Some(h)) => format!(" • {w} × {h}"),
                        _ => String::new(),
                    };
                    format!("{title}{dimensions} • {}", image.provider)
                }).map(slint::SharedString::from).collect::<Vec<_>>();

                let count = results.len();
                app.set_results(slint::ModelRc::from(Rc::new(slint::VecModel::from(results))));
                let status = if errors.is_empty() {
                    format!("{count} resultado(s) encontrado(s).")
                } else {
                    format!("{count} resultado(s); alguns provedores falharam: {}", errors.join(" | "))
                };
                app.set_status(status.into());
            });
        });
    });

    app.run()
}
