
slint::include_modules!();
use aevum::task::*;
use aevum::time::*;
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
    let date = Date::new(form.day, form.month, form.year).unwrap();
    
    Ok(())
}



#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    let app = MainWindow::new().unwrap();
    app.run().unwrap();
}
