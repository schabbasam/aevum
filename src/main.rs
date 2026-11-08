
slint::include_modules!();


#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let _app_handle = app.as_weak();

    
    app.run()?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    let app = MainWindow::new().unwrap();
    app.run().unwrap();
}
