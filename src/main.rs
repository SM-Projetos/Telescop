slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;

    app.on_search_requested(|query| {
        let query = query.trim();
        if !query.is_empty() {
            println!("Search requested: {query}");
        }
    });

    app.run()
}
