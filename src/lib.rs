
slint::include_modules!();

pub mod core;
pub mod app;

use slint::{ ModelRc, ComponentHandle };
use crate::app::handlers;
use crate::app::store::Store;
use crate::core::task::{ MAX_TITLE, MAX_DESCRIPTION };
use crate::core::time::{ MAX_DURATION };

fn update_clock(app: &MainWindow) {
    use chrono::{ Datelike, Local };
    let now = Local::now();
    app.set_date_text(now.format("%d.%m.%Y").to_string().into());
    app.set_time_text(now.format("%H:%M:%S").to_string().into());
    app.set_week_text(format!("Week {}", now.iso_week().week()).into());
}

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

    // ---- clock ----
    update_clock(&app);
    let weak = app.as_weak();
    let clock = slint::Timer::default();
    clock.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(1),
        move || {
            if let Some(app) = weak.upgrade() {
                update_clock(&app);
            }
        }
    );

    app.run()?;
    Ok(())
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_main() {
    run().unwrap();
}

