
slint::include_modules!();

pub mod core;
pub mod app;

use slint::ModelRc;
use crate::app::handlers;
use crate::app::store::Store;
use crate::core::task::{ MAX_TITLE, MAX_DESCRIPTION };
use crate::core::time::{ MAX_DURATION };

pub fn run() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    
    // ---- set maximum values ----
    app.set_max_title(MAX_TITLE as i32);
    app.set_max_description(MAX_DESCRIPTION as i32);
    app.set_max_duration(MAX_DURATION as i32);
    app.set_edit_mode(false);
    
    // ---- set up the Store ----
    let store = Store::new();
    app.set_left_column_tasks(ModelRc::from(store.left.clone()));
    app.set_right_column_tasks(ModelRc::from(store.right.clone()));
    app.set_scheduled_tasks(ModelRc::from(store.scheduled.clone()));

    // ---- wiring ----
    handlers::wire_all(&app, store.clone());

    app.run()?;
    Ok(())
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_main() {
    run().unwrap();
}

