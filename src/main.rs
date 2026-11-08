
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let app_handle = app.as_weak();

    //logic
    
    app.run()?;
    Ok(())
}
