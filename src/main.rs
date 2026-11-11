
slint::include_modules!();
use aevum::task::*;
use aevum::time::Date as DateThing;
use aevum::color::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let app_handle = app.as_weak();
    
    app.on_save_task(move |form| {
        let app = app_handle.upgrade().unwrap();
        match build_task(&form) {
            Ok(task) => {
                // TODO: store ts in a vec or smth
                println!("task created \n{:?}", task);
                app.set_is_error(false);
                app.set_status_message("task created".into());
            }
            Err(e) => {
                app.set_is_error(true);
                app.set_status_message(e.into());
            }
        }
    });
    
    app.run()?;
    Ok(())
}


fn build_task(form: &TaskForm) -> Result<Task, String> {
    let title = form.title.to_string();
    
    let date = match parse_date(form.day, form.month, form.year) {
        Some(v) => v,
        None => return Err("no date selected".to_string()),
    };

    let task = match Task::new(title, date) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error when creating a new task: {e}");
            return Err("invalid title".to_string());
        }
    };

    Ok(task)
}

fn parse_date(day: i32, month: i32, year: i32) -> Option<DateThing> {
    if day == 0 || month == 0 || year == 0 {
        return None;
    }
    Some(DateThing::new(day as u8, month as u8, year as u16).unwrap())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    let app = MainWindow::new().unwrap();
    app.run().unwrap();
}
