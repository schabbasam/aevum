
// handles all the wiring
// to keep the main loop clean

use std::rc::Rc;
use slint::{ Model, SharedString };
use crate::{ Api, DragColors, MainWindow };

use crate::app::form::build_task;
use crate::app::layout::*;
use crate::app::store::Store;
use crate::app::visuals::{ calculate_drag_colors, make_drag_preview };

use crate::core::color::Color;

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

