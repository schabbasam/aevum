
// handles all the wiring
// to keep the main loop clean

use std::rc::Rc;
use slint::{ Color as SlintColor, Weak, Model, SharedString };
use crate::{ Api, DragColors, MainWindow, TaskForm };

use crate::app::form::build_task;
use crate::app::layout::*;
use crate::app::store::Store;
use crate::app::visuals::{ calculate_drag_colors, make_drag_preview };

use crate::core::task::Task;
use crate::core::color::Color;

// ---- entry point ----
pub fn wire_all(app: &MainWindow, store: Store) {
    wire_save_task      (app, store.clone());
    wire_api            (app, store.clone());
    wire_task_placed    (app, store.clone());
    wire_task_resized   (app, store.clone());
    wire_task_to_panel  (app, store.clone());
}


// ---- shared helper ----

// contains the layout constants that are needed by everything
// bundled so we read it only when changed instead of reading it in every callback
#[derive(Clone, Copy)]
struct UiMetrics {
    hour_height:    f32,
    min_height:     f32,
    block_width:    f32,
    panel_gap:      f32,
    default_color:  SlintColor,
}

impl UiMetrics {
    fn from_app(app: &MainWindow) -> Self {
        Self {
            hour_height:    app.get_hour_height(),
            min_height:     app.get_task_block_min_height(),
            block_width:    app.get_task_block_width(),
            panel_gap:      app.get_panel_gap(),
            default_color:  app.get_default_task_color(),
        }
    }
    
    // more comfortable to use (less parameters)
    fn task_height(&self, duration: i32) -> f32 {
        task_height_px(duration, self.hour_height, self.min_height)
    }
}

// set message and error flag in one go
fn set_status(app: &MainWindow, message: impl Into<SharedString>, is_error: bool) {
    app.set_status_message(message.into());
    app.set_is_error(is_error);
}

// convert form.color into SlintColor, fallback is default
fn resolved_color(form: &TaskForm, default: SlintColor) -> SlintColor {
    Color::from_hex(&form.color)
        .map(|c| SlintColor::from_rgb_u8(c.red(), c.green(), c.blue()))
        .unwrap_or(default)
}


// ---- save task ----

pub fn wire_save_task(app: &MainWindow, store: Store) {
    let weak = app.as_weak();
    app.on_save_task(move |form| {
        let Some(app) = weak.upgrade() else { return };
        match build_task(&form) {
            Ok(task) => on_task_created(&app, &store, form, &task),
            Err(e)   => set_status(&app, e, true),
        }
    });
}

fn on_task_created(app: &MainWindow, store: &Store, mut form: TaskForm, task: &Task) {
    let m = UiMetrics.from_app(app);

    form.id             = task.id() as i32;
    form.color_brush    = resolved_color(&form, m.default_color);
    // removed drag-preview since it doesn't matter

    store // put the task form into the shorter panel column
        .shorter_panel_column(m.hour_height, m.min_height)
        .push(form);
    
    set_status(app, "task created", false);
}

// ---- api ----

pub fn wire_api(app: &MainWindow, store: Store) {
    let api = app.global.<Api>();

    // the transfer data converter (int <-> transfer-data)
    api.on_int_to_transfer(|id| {
        SharedString::from(id.to_string().into())
    });

    api.on_transfer_to_int(|data| {
        data.plain_text()
            .unwrap_or_default()
            .to_string()
            .parse::<i32>()
            .unwrap_or(0)
    });
    
    // splitting it into smaller functions to keep everything neat and tidy
    wire_preview_start(&api, store.clone());
    wire_panel_slot_index(&api, app.as_weak(), store);

    api.on_calculate_drag_colors(|original| {
        let (preview, border) = calculate_drag_colors(original);
        DragColors { preview, border }
    });
}

fn wire_preview_start(api: &Api, store: Store) {
    api.on_preview_start(move |id, col, wish| {
        let Some((src, idx)) = store.find(id) else { return -1 };
        let dur = normalize_duration(src.row_data(idx).unwrap().duration);
        let wish = wish.clamp(0, DAY_MINUTES - dur);
        find_free_start(&store.scheduled, id, col, dur, wish).unwrap_or(-1)
    });
}

fn wire_panel_slot_index(api: &Api, weak: Weak<MainWindow>, store: Store) {
// TODO stuff
}

pub fn wire_task_placed(app: &MainWindow, store: Store) {

}

pub fn wire_task_resized(app: &MainWindow, store: Store) {

}

pub fn wire_task_to_panel(app: &MainWindow, store: Store) {

}

