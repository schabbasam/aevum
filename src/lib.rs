
slint::include_modules!();

use slint::{Model, ModelRc, VecModel, SharedString};
use std::rc::Rc;

use crate::task::*;
use crate::time::Date as DateThing;
use crate::time::*;
use crate::color::*;

pub mod time;
pub mod task;
pub mod color;

const DAY_MINUTES: i32 = 1440;
const STEP: i32 = 15;
const DEFAULT_DURATION: i32 = 60;

#[derive(Clone)]
struct Store {
    left: Rc<VecModel<TaskForm>>,
    right: Rc<VecModel<TaskForm>>,
    scheduled: Rc<VecModel<TaskForm>>,
}
impl Store {
    fn new() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            scheduled: Default::default(),
        }
    }
    fn panel_column(&self, col: i32) -> &Rc<VecModel<TaskForm>> {
        if col == 0 {
            &self.left
        } else {
            &self.right
        }
    }
    // search where the task is
    fn find(&self, id: i32) -> Option<(&Rc<VecModel<TaskForm>>, usize)> {
        [&self.left, &self.right, &self.scheduled]
            .into_iter()
            .find_map(|m| m.iter().position(|t| t.id == id)
                .map(|i| (m, i)))
    }

    fn shorter_panel_column(&self, hour_h: f32, min_h: f32) -> &Rc<VecModel<TaskForm>> {
        if column_height(&self.left, hour_h, min_h) <= column_height(&self.right, hour_h, min_h) {
            &self.left
        } else {
            &self.right
        }
    }
}

fn column_height(m: &VecModel<TaskForm>, hour_h: f32, min_h: f32) -> f32 {
    m.iter().map(|t| task_height_px(t.duration, hour_h, min_h)).sum()
}

fn normalize_duration(d: i32) -> i32 {
    let d = if d <= 0 { DEFAULT_DURATION } else { d };
    ((d + STEP - 1) / STEP * STEP).clamp(STEP, DAY_MINUTES)
}

fn insertion_index(col: &VecModel<TaskForm>, y: f32, hour_h: f32, min_h: f32, gap: f32) -> usize {
    let mut top = gap;
    for (i, t) in col.iter().enumerate() {
        let h = task_height_px(t.duration, hour_h, min_h);
        if y < top + h / 2.0 {
            return i;
        }
        top += h + gap;
    }
    col.row_count()
}

fn find_free_start(sched: &VecModel<TaskForm>, id: i32, col: i32, dur: i32, wish: i32) -> Option<i32> {
    let mut busy: Vec<(i32, i32)> = sched
        .iter()
        .filter(|t| t.id != id && t.column == col)
        .map(|t| (t.start_minute, t.start_minute + t.duration))
        .collect();
    busy.sort();

    let mut cands = vec![widh, 0, DAY_MINUTES - dur];
    for &(s, e) in &busy {
        cands.push(e);
        cands.push(s - dur);
    }

    cands
        .into_iter()
        .filter(|&c| c >= 0 && c + dur <= DAY_MINUTES)
        .filter(|&c| busy.iter().all(|&(s, e)| c + dur <= s || c >= e))
        .min_by_key(|&c| (c-wish).abs())
}

fn update_all_limits(sched: &VecModel<TaskForm>) {
    for col in 0..7 {
        let items: Vec<(usize, TaskForm)> = sched
            .iter().enumerate().filter(|(_, t)| t.column == col).collect();

        for (row, t) in &items {
            let (start, end) = (t.start_minute, t.start_minute + t.duration);
            let min_start = items.iter()
                .filter(|(_, o)| o.id != t.id && o.start_minute + o.duration <= start)
                .map(|(_, o)| o.start_minute + o.duration)
                .max().unwrap_or(0);
            let max_end = items.iter()
                .filter(|(_, o)| o.id != t.id && o.start_minute >= end)
                .map(|(_, o)| o.start_minute)
                .min().unwrap_or(DAY_MINUTES);
            
            if t.min_start != min_start || t.max_end != max_end {
                let mut t2 = t.clone();
                t2.min_start = min_start;
                t2.max_end = max_end;
                sched.set_row_data(*row, t2);
            }
        }
    }
}

pub fn run() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let app_handle = app.as_weak();

    // set max length
    app.set_max_title(MAX_TITLE as i32);
    app.set_max_description(MAX_DESCRIPTION as i32);
    app.set_max_duration(MAX_DURATION as i32);

    // tasks list
    let tasks: Rc<VecModel<TaskForm>> = Rc::new(VecModel::default());
    let tasks_handle = tasks.clone();

    let scheduled: Rc<VecModel<TaskForm>> = Rc::new(VecModel::default());
    app.set_scheduled_tasks(ModelRc::from(scheduled.clone()));

    let api = app.global::<Api>();
    api.on_int_to_transfer(|id| SharedString::from(id.to_string()).into());
    api.on_transfer_to_int(|data| {
        data.plain_text()
            .unwrap_or_default()
            .to_string()
            .parse::<i32>()
            .unwrap_or(0)
    });

    let refresh = {
        let app_handle = app_handle.clone();
        let task_handle = tasks_handle.clone();

        Rc::new(move || {
            let Some(a) = app_handle.upgrade() else {
                return;
            };

            let hour_height = a.get_hour_height();
            let min_height = a.get_task_block_min_height();

            let snapshot: Vec<TaskForm> = task_handle.iter().collect();
            let (l, r) = balance_columns(&snapshot, hour_height, min_height);

            a.set_left_column_tasks(ModelRc::from(Rc::new(VecModel::from(l))));
            a.set_right_column_tasks(ModelRc::from(Rc::new(VecModel::from(r))));
        })
    };

    let tasks_save = tasks_handle.clone();
    let refresh_save = refresh.clone();

    app.on_save_task(move |form| {
        let app = app_handle.upgrade().unwrap();

        match build_task(&form) {
            Ok(task) => {
                let mut form = form;
                form.id = task.id() as i32;

                let w = app.get_task_block_width();
                let hour_height = app.get_hour_height();
                let min_height = app.get_task_block_min_height();
                let default_color = app.get_default_task_color();

                let h = task_height_px(form.duration, hour_height, min_height);

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

                tasks_save.push(form);
                refresh_save();

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

    let panel = tasks_handle.clone();
    let sched = scheduled.clone();
    let refresh_drop = refresh.clone();

    app.on_task_dropped(move |id, x, y| {
        let mut form = if let Some(idx) = panel.iter().position(|t| t.id == id) {
            panel.remove(idx)
        } else if let Some(idx) = sched.iter().position(|t| t.id == id) {
            sched.remove(idx)
        } else {
            return;
        };

        form.drop_x = x;
        form.drop_y = y;

        sched.push(form);
        refresh_drop();
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
        let dura = Duration::new(form.duration as u16).map_err(|e| {
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

fn make_drag_preview(
    width: f32,
    height: f32,
    hex: Option<&str>,
) -> slint::Image {
    let w = width.round() as u32;
    let h = height.round() as u32;

    let (r, g, b) = hex
        .and_then(|s| Color::from_hex(s).ok())
        .map(|c| (c.red(), c.green(), c.blue()))
        .unwrap_or((124, 114, 232));

    let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
    let slice = buf.make_mut_slice();

    for p in slice {
        *p = slint::Rgba8Pixel {
            r,
            g,
            b,
            a: 255,
        };
    }

    slint::Image::from_rgba8_premultiplied(buf)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_main() {
    run().unwrap();
}

