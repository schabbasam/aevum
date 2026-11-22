
slint::include_modules!();

use slint::{Model, ModelRc, VecModel, SharedString};
use std::rc::Rc;
use slint::Color as SlintColor;

use crate::task::*;
use crate::time::Date as DateThing;
use crate::time::*;
use crate::color::*;

pub mod time;
pub mod task;
pub mod color;

pub fn run() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let app_handle = app.as_weak();

    // set max length constants
    app.set_max_title(MAX_TITLE as i32);
    app.set_max_description(MAX_DESCRIPTION as i32);
    app.set_max_duration(MAX_DURATION as i32);

    // TODO add edit mode
    app.set_edit_mode(false);
    
    // create Store and set slint properties
    let store = Store::new();
    app.set_left_column_tasks(ModelRc::from(store.left.clone()));
    app.set_right_column_tasks(ModelRc::from(store.right.clone()));
    app.set_scheduled_tasks(ModelRc::from(store.scheduled.clone()));

    let st = store.clone();
    app.on_save_task(move |form| {
        // very unlikely, but just in case the window is dropped mid flight or something
        // this defensive way doesnt panic
        let Some(app) = app_handle.upgrade() else { return };
        
        // create the task
        match build_task(&form) {

            // on success
            Ok(task) => {
                let mut form = form;
                form.id = task.id() as i32;
                
                // get some constants
                let w = app.get_task_block_width();
                let hour_height = app.get_hour_height();
                let min_height = app.get_task_block_min_height();
                let default_color = app.get_default_task_color();

                let h = task_height_px(form.duration, hour_height, min_height);
                
                // color
                let hex = task.color().map(|c| c.to_hex());
                form.drag_preview = make_drag_preview(w, h, hex.as_deref());

                let parsed = Color::from_hex(&form.color).ok();
                let brush = match parsed {
                    Some(c) => slint::Color::from_rgb_u8(
                        c.red(),
                        c.green(),
                        c.blue(),
                    ),
                    None => default_color,
                };

                form.color_brush = brush;
                
                // add it to the shorter panel column
                st.shorter_panel_column(hour_height, min_height).push(form);

                println!("task created \n{:?}", task);
                app.set_is_error(false);
                app.set_status_message("task created".into());
            }

            // on failure
            Err(e) => {
                app.set_is_error(true);
                app.set_status_message(e.into());
            }
        }
    });
    
    // Api callbacks
    // necessary because we need a way to convert from and to the drag&drop transfer data type
    let api = app.global::<Api>();
    api.on_int_to_transfer(|id| SharedString::from(id.to_string()).into());
    api.on_transfer_to_int(|data| {
        data.plain_text()
            .unwrap_or_default()
            .to_string()
            .parse::<i32>()
            .unwrap_or(0)
    });
    let st = store.clone();
    api.on_preview_start(move |id, col, wish| {
        let Some((src, idx)) = st.find(id) else { return -1 };
        let dur = normalize_duration(src.row_data(idx).unwrap().duration);
        let wish = wish.clamp(0, DAY_MINUTES - dur);
        find_free_start(&st.scheduled, id, col, dur, wish).unwrap_or(-1)
    });
    let (weak, st) = (app.as_weak(), store.clone());
    api.on_panel_slot_index(move |col, y, cur, extra| {
        let Some(app) = weak.upgrade() else { return 0 };
        insertion_index_shifted(
            st.panel_column(col),
            y,
            app.get_hour_height(),
            app.get_task_block_min_height(),
            app.get_panel_gap(),
            cur,
            extra,
        ) as i32
    });

    api.on_calculate_drag_colors(|original| {
        let (preview, border) = calculate_drag_colors(original);

        DragColors {
            preview,
            border,
        }
    });

    let (weak, st) = (app.as_weak(), store.clone());
    app.on_task_placed(move |id, col, start| {

        // if Some is returned, it gets extracted into these variables
        // else the function is aborted
        let Some(app) = weak.upgrade() else { return };
        let Some((src, idx)) = st.find(id) else { return };

        // clone the form
        let mut form = src.row_data(idx).unwrap();

        let dur = normalize_duration(form.duration);
        let wish = start.clamp(0, DAY_MINUTES - dur);

        let Some(free) = find_free_start(&st.scheduled, id, col, dur, wish) else {
            app.set_is_error(true);
            app.set_status_message("no free space on that day".into());
            return; // block stays where it was, nothing moved
        };

        form.column = col;
        form.start_minute = free;
        form.duration = dur;
        
        // check if the pointers of the Rc are equal
        if Rc::ptr_eq(src, &st.scheduled) {
            st.scheduled.set_row_data(idx, form);
        } else {
            // if not, replace it with the new one
            src.remove(idx);
            st.scheduled.push(form);
        }
        update_all_limits(&st.scheduled);
        app.set_is_error(false);
        app.set_status_message("".into());
    });

    let st = store.clone();
    app.on_task_resized(move |id, start, dur| {
        // find index in scheduled tasks
        let Some(idx) = st.scheduled.iter().position(|t| t.id == id) else { return };
        let mut form = st.scheduled.row_data(idx).unwrap();

        let start = start.max(form.min_start);
        let end = (start + dur).min(form.max_end);

        // if duration gets smaller than the STEP
        if end - start < STEP { return };

        form.start_minute = start;
        form.duration = end - start;

        // set the new thing into scheduled and update
        st.scheduled.set_row_data(idx, form);
        update_all_limits(&st.scheduled);
    });
    
    let (weak, st) = (app.as_weak(), store.clone());
    app.on_task_to_panel(move |id, col, y| {
        let Some(app) = weak.upgrade() else { return };
        let Some((src, idx)) = st.find(id) else { return };
        let form = src.row_data(idx).unwrap();
        let target = st.panel_column(col);

        let mut at = insertion_index(
            target, // column
            y, // wished position
            app.get_hour_height(),
            app.get_task_block_min_height(),
            app.get_panel_gap(),
        );

        if Rc::ptr_eq(src, target) && idx < at {
            // decrement index if source column and target are the same
            // because the dragged task is still in the index of target
            at -= 1;
        }

        // replace and update
        src.remove(idx);
        target.insert(at, form);
        update_all_limits(&st.scheduled);
    });

    app.run()?;
    Ok(())
}

// returns task height in pixels
fn task_height_px(duration: i32, hour_height: f32, min_height: f32) -> f32 {
    let height = (duration as f32 / 60.0) * hour_height;
    height.max(min_height)
}

fn balance_columns(
    t: &[TaskForm],
    hour_height: f32,
    min_height: f32,
) -> (Vec<TaskForm>, Vec<TaskForm>) {
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_main() {
    run().unwrap();
}

