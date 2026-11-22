
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


// ---- functions / callback handlers ----

pub fn wire_save_task(app: &MainWindow, store: Store) {

}

pub fn wire_api(app: &MainWindow, store: Store) {

}

pub fn wire_task_placed(app: &MainWindow, store: Store) {

}

pub fn wire_task_resized(app: &MainWindow, store: Store) {

}

pub fn wire_task_to_panel(app: &MainWindow, store: Store) {

}

