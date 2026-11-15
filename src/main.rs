
slint::include_modules!();
use slint::{Model, ModelRc, VecModel, Rgba8Pixel};
use std::rc::Rc;

use aevum::task::*;
use aevum::time::Date as DateThing;
use aevum::time::*;
use aevum::color::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let app_handle = app.as_weak();
    
    // set max length
    app.set_max_title(MAX_TITLE as i32);
    app.set_max_description(MAX_DESCRIPTION as i32);
    app.set_max_duration(MAX_DURATION as i32);

    // tasks list
    let tasks: Rc<VecModel<TaskForm>> = Rc::new(VecModel::default());
    let tasks_handle = tasks.clone();

    let api = app.global::<Api>();
    api.on_int_to_transfer(Into::into);

    let refresh = {
        let app_handle = app_handle.clone();
        let task_handle = tasks_handle.clone();
        move || {
            let Some(a) = app_handle.upgrade() else { return };
            let hour_height = a.get_hour_height();
            let min_height = a.get_task_block_min_height();
            let snapshot: Vec<TaskForm> = task_handle.iter().collect();
            let (l, r) = balance_columns(&snapshot, hour_height, min_height);
            a.set_left_column_tasks(ModelRc::from(Rc::new(VecModel::from(l))));
            a.set_right_column_tasks(ModelRc::from(Rc::new(VecModel::from(r))));
        }
    };

    app.on_save_task(move |form| {
        let app = app_handle.upgrade().unwrap();
        match build_task(&form) {
            Ok(task) => {
                let mut form = form;
                form.id = task.id() as i32;

                // compute the pixel size the block will have
                let w = app.get_task_block_width();
                let hour_height = app.get_hour_height();
                let min_height = app.get_task_block_min_height();
                let default_color = app.get_default_task_color();
                let h = task_height_px(form.duration, hour_height, min_height);
                
                let hex = task.color().map(|c| c.to_hex());
                form.drag_preview = make_drag_preview(w, h, hex.as_deref());
                
                let parsed = Color::from_hex(&form.color).ok();
                let brush = match parsed {
                    Some(c) => slint::Color::from_rgb_u8(c.red(), c.green(), c.blue()),
                    None => default_color
                };
                
                form.color_brush = brush;

                tasks_handle.push(form);
                refresh();

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

    let mut task = match Task::new(title, date) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error when creating a new task: {e}");
            return Err("invalid title".to_string());
        }
    };
    match parse_date(form.day, form.month, form.year) {
        None => {
            return Err("no date selected".to_string());
        }
        Some(d) => {
            task.set_date(d);
        }
    }
    
    let desc = if form.description.is_empty() {
        None
    } else {
        Some(form.description.to_string())
    };
    if let Err(e) = task.set_description(desc) {
        eprintln!("error when setting description: {e}");
        return Err("description too long".to_string());
    }
   
    if form.duration == 0 {
        task.set_duration(None);
    } else {
        let dura = Duration::new(form.duration as u16)
            .map_err(|e| {
                eprintln!("error when creating Duration: {e}");
                "duration can't be longer than a day".to_string()
            })?;

        task.set_duration(Some(dura));
    }
    
    if form.color.is_empty() {
        task.set_color(None);
    } else {
        let c = Color::from_hex(&form.color);
        if let Err(e) = c {
            eprintln!("error when creating Color: {e}");
            return Err("invalid color".to_string());
        } else {
            task.set_color(Some(c.unwrap()));
        }
    }
    Ok(task)
}

fn parse_date(day: i32, month: i32, year: i32) -> Option<DateThing> {
    if day == 0 || month == 0 || year == 0 {
        return None;
    }
    Some(DateThing::new(day as u8, month as u8, year as u16).unwrap())
}

fn task_height_px(duration: i32, hour_height: f32, min_height: f32) -> f32 {
    let height = (duration as f32 / 60.0) * hour_height;
    height.max(min_height)
}

fn balance_columns(t: &[TaskForm], hour_height: f32, min_height: f32)
    -> (Vec<TaskForm>, Vec<TaskForm>) {
    
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut left_height = 0.0f32;
    let mut right_height = 0.0f32;

    for task in t {
        let h = task_height_px(task.duration, hour_height, min_height);
        if left_height <= right_height {
            left.push(task.clone());
            left_height += h;
        } else {
            right.push(task.clone());
            right_height += h;
        }
    }

    (left, right)
}

fn make_drag_preview(width: f32, height: f32, hex: Option<&str>) -> slint::Image {
    let w = width.round() as u32;
    let h = height.round() as u32;
    
    let (r, g, b) = hex
        .and_then(|s| Color::from_hex(s).ok())
        .map(|c| (c.red(), c.green(), c.blue()))
        .unwrap_or((124, 114, 232)); // fallback color (accent)
    
    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    let slice = buf.make_mut_slice();
    for p in slice {
        *p = slint::Rgba8Pixel { r, g, b, a: 255 };
    }
    eprintln!("first 4 bytes: {:?}", &buf.make_mut_slice()[0..1]);
    slint::Image::from_rgba8_premultiplied(buf)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    let app = MainWindow::new().unwrap();
    app.run().unwrap();
}
