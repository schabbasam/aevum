
slint::include_modules!();
use aevum::task::*;
use aevum::time::Date;
use aevum::color::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
//    let app_handle = app.as_weak();
    
    app.on_save_task(move |form| {
        match build_task(&form) {
            Ok(task) => {
                // TODO: store ts in a vec or smth
                println!("task created \n{:?}", task);
            }
            Err(e) => eprintln!("invalid task: {e}")
        }
    });
    
    app.run()?;
    Ok(())
}


fn build_task(form: &TaskForm) -> Result<Task, String> {
    let title = form.title.to_string();
    
    match parse_date(form.day, form.month, form.year) {
        Some(d) => {
            let date = d;
        }
        None => {
            return Err("no date selected");
        }
    }
    Ok(())
}

fn parse_date(day: i32, month: i32, year: i32) -> Option<Date> {
    if day == 0 || month == 0 || year == 0 {
        return None;
    }
    Some(Date::new(day as u8, month as u8, year as u16))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    let app = MainWindow::new().unwrap();
    app.run().unwrap();
}
