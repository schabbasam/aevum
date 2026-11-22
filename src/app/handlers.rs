
// handles all the wiring
// to keep the main loop clean

use std::rc::Rc;
use slint::{ Color as SlintColor, Model, SharedString, ComponentHandle, Weak };
use crate::{ Api, DragColors, MainWindow, TaskForm };

use crate::app::form::build_task;
use crate::app::layout::*;
use crate::app::store::Store;
use crate::app::visuals::calculate_drag_colors;

use crate::core::task::{ Task, Category };
use crate::core::color::Color;

// ---- entry point ----
pub fn wire_all(app: &MainWindow, store: Store) {
    let m = UiMetrics::from_app(app);

    wire_save_task      (app, store.clone(), m);
    wire_api            (app, store.clone(), m);
    wire_task_placed    (app, store.clone());
    wire_task_resized   (app, store.clone());
    wire_task_to_panel  (app, store.clone(), m);
    wire_add_category   (app, store.clone());
}


// ---- shared helper ----

// contains the layout constants that are needed by everything
// bundled so we read it only when changed instead of reading it in every callback
#[derive(Clone, Copy)]
struct UiMetrics {
    hour_height:    f32,
    min_height:     f32,
    panel_gap:      f32,
    default_color:  SlintColor,
}

impl UiMetrics {
    fn from_app(app: &MainWindow) -> Self {
        Self {
            hour_height:    app.get_hour_height(),
            min_height:     app.get_task_block_min_height(),
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

fn wire_save_task(app: &MainWindow, store: Store, m: UiMetrics) {
    let weak = app.as_weak();
    app.on_save_task(move |form| {
        let Some(app) = weak.upgrade() else { return };
        match build_task(&form) {
            Ok(task) => on_task_created(&app, &store, form, &task, m),
            Err(e)   => set_status(&app, e, true),
        }
    });
}

fn on_task_created(app: &MainWindow, store: &Store, mut form: TaskForm, task: &Task, m: UiMetrics) {

    form.id             = task.id() as i32;
    form.color_brush    = resolved_color(&form, m.default_color);
    // removed drag-preview since it doesn't matter

    store // put the task form into the shorter panel column
        .shorter_panel_column(m.hour_height, m.min_height)
        .push(form);
    
    set_status(app, "task created", false);
}

// ---- api ----

fn wire_api(app: &MainWindow, store: Store, m: UiMetrics) {
    let api = app.global::<Api>();

    wire_converters(&api);
    wire_preview_start(&api, store.clone());
    wire_panel_slot_index(&api, store, m);
    wire_drag_color_calculation(&api);
}

fn wire_converters(api: &Api) {
    // the transfer data converter (int <-> transfer-data)
    api.on_int_to_transfer(|id| {
        SharedString::from(id.to_string()).into()
    });

    api.on_transfer_to_int(|data| {
        data.plain_text()
            .unwrap_or_default()
            .to_string()
            .parse::<i32>()
            .unwrap_or(0)
    });
}

fn wire_preview_start(api: &Api, store: Store) {
    api.on_preview_start(move |id, col, wish| {
        let Some((_, _, form)) = store.find_task(id) else { return -1 };
        let dur = normalize_duration(form.duration);
        let wish = wish.clamp(0, DAY_MINUTES - dur);
        find_free_start(&store.scheduled, id, col, dur, wish).unwrap_or(-1)
    });
}

fn wire_panel_slot_index(api: &Api, store: Store, m: UiMetrics) {
    api.on_panel_slot_index(move |col, y, current, extra| {
        insertion_index_shifted(
            store.panel_column(col),
            y,
            m.hour_height,
            m.min_height,
            m.panel_gap,
            current,
            extra
        ) as i32
    });
}

fn wire_drag_color_calculation(api: &Api) {
    api.on_calculate_drag_colors(|original| {
        let (preview, border) = calculate_drag_colors(original);
        DragColors { preview, border }
    });
}


// ---- drag to calendar ----

fn wire_task_placed(app: &MainWindow, store: Store) {
    let weak = app.as_weak();
    app.on_task_placed(move |id, col, start| {
        let Some(app) = weak.upgrade() else { return };
        move_to_calendar(&app, &store, id, col, start);
    });
}

fn move_to_calendar(app: &MainWindow, store: &Store, id: i32, col: i32, start: i32) {
    let Some((src, index, mut form)) = store.find_task(id) else { return };

    let dur = normalize_duration(form.duration);
    let wish = start.clamp(0, DAY_MINUTES - dur);

    let Some(free) = find_free_start(&store.scheduled, id, col, dur, wish) else {
        set_status(app, "no free space on that day", true);
        return;
    };

    form.column         = col;
    form.start_minute   = free;
    form.duration       = dur;
    
    // if task already exists in scheduled
    if Rc::ptr_eq(src, &store.scheduled) {
        // replace its form data
        store.scheduled.set_row_data(index, form);
    } else {
        // remove from source (TaskPanel)
        // and move it to scheduled
        src.remove(index);
        store.scheduled.push(form);
    }

    update_all_limits(&store.scheduled);
    set_status(app, "", false);
}


// ---- resize ----   

fn wire_task_resized(app: &MainWindow, store: Store) {
    app.on_task_resized(move |id, start, dur| {
        resize(&store, id, start, dur)
    });
}

fn resize(store: &Store, id: i32, start: i32, dur: i32) {
    // search scheduled for the task with id and save its index
    let Some(index) = store.scheduled.iter()
        .position(|t| t.id == id) else { return };

    // mutable copy of the found task
    let mut form = store.scheduled.row_data(index).unwrap();
    
    // recalculate the max-end and min-start values
    let start = start.max(form.min_start);
    let end = (start + dur).min(form.max_end);
    
    // abort if the gap is less than STEP (shouldn't be possible)
    if end - start < STEP { return }

    form.start_minute = start;
    form.duration = end - start;

    store.scheduled.set_row_data(index, form);
    update_all_limits(&store.scheduled);
}


// ---- drag into task panel column ----

fn wire_task_to_panel(app: &MainWindow, store: Store, m: UiMetrics) {
    app.on_task_to_panel(move |id, col, y| {
        move_to_panel(&store, id, col, y, m);
    });
}

fn move_to_panel(store: &Store, id: i32, col: i32, y: f32, m: UiMetrics) {
    let Some((src, index, form)) = store.find_task(id) else { return };
    let target = store.panel_column(col);
    
    // TODO here are the getters too instead of UiMetrics
    let mut at = insertion_index(
        target,
        y,
        m.hour_height,
        m.min_height,
        m.panel_gap,
    );
    
    // if it is the same column and the dragged task is above target
    if Rc::ptr_eq(src, target) && index < at {
        // subtract the target because the dragged task doesnt count
        at -= 1;
    }

    src.remove(index);
    target.insert(at, form);
    update_all_limits(&store.scheduled);
}


// ---- add category ----


