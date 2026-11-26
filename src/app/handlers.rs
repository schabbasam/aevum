
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

    wire_save_task      (app, store.clone(), m.clone());
    wire_api            (app, store.clone(), m.clone());
    wire_task_placed    (app, store.clone());
    wire_task_resized   (app, store.clone());
    wire_task_to_panel  (app, store.clone(), m);
    wire_add_category   (app, store.clone());
}


// ---- shared helper ----

// these fallbacks are actually unreachable in praxis
// because if the slint getter returns None, the app is already dead
// but because there is no way around having to use a fallback, and even
// let else blocks with return 0 would return a valid lied value,
// lets just write these fallbacks
const FALLBACK_HOUR_HEIGHT: f32 = 50.0;
const FALLBACK_MIN_HEIGHT:  f32 = 40.0;
const FALLBACK_PANEL_GAP:   f32 = 8.0;
const FALLBACK_TASK_COLOR: SlintColor = SlintColor::from_rgb_u8(255, 0, 0);

// contains the layout constants that are needed by everything
// you may think that we could skip this entirely and use the getters for everything
// but that would clone the Weak in each callback and burden the boilerplate
// this makes it slightly more complex but it is cleaner
// and I'm all in for performance optimizations, even if they are just tiny things
#[derive(Clone)]
struct UiMetrics {
    // reminder: need to use a Weak to prevent a cycle where everything points at each
    // other and never exits
    app: Weak<MainWindow>
}

impl UiMetrics {
    fn from_app(app: &MainWindow) -> Self {
        Self { app: app.as_weak() }
    }

    // generic get function
    // Fn would also work, but FnOnce also allows comsumables (if i use move |a|)
    // FnMut would be overkill since nothign is getting mutated
    fn get<T>(&self, f: impl FnOnce(&MainWindow) -> T, fallback: T) -> T {
        match self.app.upgrade() {
            Some(a) => f(&a), // a is an Rc<MainWindow>, it auto derefs
            None    => fallback
        }
    }

    fn hour_height(&self) -> f32 {
        self.get(|a| a.get_hour_height(), FALLBACK_HOUR_HEIGHT)
    }
    fn min_height(&self) -> f32 {
        self.get(|a| a.get_task_block_min_height(), FALLBACK_MIN_HEIGHT)
    }
    fn panel_gap(&self) -> f32 {
        self.get(|a| a.get_panel_gap(), FALLBACK_PANEL_GAP)
    }
    fn default_color(&self) -> SlintColor {
        self.get(|a| a.get_default_task_color(), FALLBACK_TASK_COLOR)
    }
    fn task_height(&self, duration: i32) -> f32 {
        task_height_px(duration, self.hour_height(), self.min_height())
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
            Ok(task) => on_task_created(&app, &store, form, &task, m.clone()),
            Err(e)   => set_status(&app, e, true),
        }
    });
}

fn on_task_created(app: &MainWindow, store: &Store, mut form: TaskForm, task: &Task, m: UiMetrics) {

    form.id             = task.id() as i32;
    form.color_brush    = resolved_color(&form, m.default_color());
    form.duration       = normalize_duration(form.duration);
    // removed drag-preview since it doesn't matter

    store // put the task form into the shorter panel column
        .shorter_panel_column(m.hour_height(), m.min_height())
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
    api.on_panel_slot_index(move |col, y, current, extra, dragged| {
        insertion_index_shifted(
            store.panel_column(col),
            y,
            m.hour_height(),
            m.min_height(),
            m.panel_gap(),
            current,
            extra,
            dragged
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
    app.on_task_to_panel(move |id, col, at| {
        move_to_panel(&store, id, col, at.max(0) as usize);
    });
}

fn move_to_panel(store: &Store, id: i32, col: i32, mut at: usize) {
    let Some((src, index, form)) = store.find_task(id) else { return };
    let target = store.panel_column(col);
    
    if Rc::ptr_eq(src, target) && index < at {
        at -= 1;
    }
    src.remove(index);
    target.insert(at.min(target.row_count()), form);
    update_all_limits(&store.scheduled);
}


// ---- add category ----

fn wire_add_category(app: &MainWindow, store: Store) {
    let weak = app.as_weak();
    app.on_add_category(move |name, color| {
        let Some(app) = weak.upgrade() else { return };

        let Ok(color) = Color::from_hex(&color) else {
            set_status(&app, "invalid color", true);
            return;
        };

        match Category::new(name.to_string(), color) {
            Ok(cat) => {
                store.categories.borrow_mut().push(cat);
                set_status(&app, "category created", false);
            }
            Err(e) => set_status(&app, e, true),
        }
    });
}
