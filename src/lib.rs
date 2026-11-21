
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

// this struct stores all tasks in their layouts
#[derive(Clone)]
struct Store {
    // a VecModel is slints observable vector
    // Rc so it can be shared between closure captures
    left: Rc<VecModel<TaskForm>>,
    right: Rc<VecModel<TaskForm>>,
    scheduled: Rc<VecModel<TaskForm>>,
}
impl Store {

    // just creates three empty models
    fn new() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            scheduled: Default::default(),
        }
    }

    // returns all tasks of a column
    fn panel_column(&self, col: i32) -> &Rc<VecModel<TaskForm>> {
        if col == 0 {
            &self.left
        } else {
            &self.right
        }
    }

    // search all three places to find where the task is
    fn find(&self, id: i32) -> Option<(&Rc<VecModel<TaskForm>>, usize)> {
        [&self.left, &self.right, &self.scheduled]
            .into_iter()
            // if im not wrong, find_map is the same as filter_map + find
            .find_map(|m| { // for each element m
                m.iter()    // look inside m
                    .position(|t| t.id == id)   // find the index of matching t
                    .map(|i| (m, i))            // if found, create a pair of m and index i
            })
    }
    
    // the panel with shorter column_height
    // used to balance the mansory layout
    fn shorter_panel_column(&self, hour_h: f32, min_h: f32) -> &Rc<VecModel<TaskForm>> {
        if column_height(&self.left, hour_h, min_h) <= column_height(&self.right, hour_h, min_h) {
            &self.left
        } else {
            &self.right
        }
    }
}

// sum of all task's task_height_px
fn column_height(m: &VecModel<TaskForm>, hour_h: f32, min_h: f32) -> f32 {
    // for each task t, do task_height_px() and sum it up
    m.iter().map(|t| task_height_px(t.duration, hour_h, min_h)).sum()
}

fn normalize_duration(d: i32) -> i32 {
    // if duration d is smaller or equal 0, it is the default duration
    let d = if d <= 0 { DEFAULT_DURATION } else { d };
    // first it rounds up to a multiple of STEP
    // then clamp to STEP and DAY_MINUTES (cant get hight or lower than these)
    ((d + STEP - 1) / STEP * STEP).clamp(STEP, DAY_MINUTES) // clamp(min, max)
}

// insert a y-coordinate where you want to put the task
// returns the index where the new item should be
fn insertion_index(col: &VecModel<TaskForm>, y: f32, hour_h: f32, min_h: f32, gap: f32) -> usize {
    let mut top = gap; // the gap above the first task
    for (i, t) in col.iter().enumerate() { // for each task (i is index, t is task)
        // calculating the height of t
        let h = task_height_px(t.duration, hour_h, min_h);
        // if y is above the midpoint of the task
        if y < top + h / 2.0 {
            // insert above this task
            return i;
        }
        // otherwise, advance to the next task
        top += h + gap;
    }
    // row count is the VecModel's item count
    // cant use .len() because its not a plain Vec
    col.row_count()
}

// return the free spot closest to wish
fn find_free_start(sched: &VecModel<TaskForm>, id: i32, col: i32, dur: i32, wish: i32) -> Option<i32> {
    let mut busy: Vec<(i32, i32)> = sched
        .iter() // collect all tasks
        .filter(|t| t.id != id && t.column == col) // except this one (the moving task)
        // turn every task into a (start, end)
        .map(|t| (t.start_minute, t.start_minute + t.duration))
        .collect(); // materialize the list
    // sort by start time to create a canonical order
    busy.sort();
    
    // vector of candidates, starts with 3 obvious picks already (exact wish, start of day, latest possible)
    let mut cands = vec![wish, 0, DAY_MINUTES - dur];
    // since were using a ref of busy, we 'destruct' it to pull out s and e
    for &(s, e) in &busy { // (s, e) = (start, end)
        // if there is something blocking it, just pick
        // the spot right below or above it
        cands.push(e);
        cands.push(s - dur);
    }

    cands
        .into_iter() // consume and turn into iter
        // filter the ones that start before midnight or end after the end of the day
        .filter(|&c| c >= 0 && c + dur <= DAY_MINUTES)
        // filter if it overlaps with any busy element
        .filter(|&c| busy.iter().all(|&(s, e)| c + dur <= s || c >= e))
        .min_by_key(|&c| (c - wish).abs()) // pick the one closest to wish
}

fn update_all_limits(sched: &VecModel<TaskForm>) {
    // for each weekday column
    for col in 0..7 {
        let items: Vec<(usize, TaskForm)> = sched
            .iter().enumerate()
            // filter out tasks of other columns
            .filter(|(_, t)| t.column == col)
            .collect();

        for (row, t) in &items {
            // create an interval for each task t
            let (start, end) = (t.start_minute, t.start_minute + t.duration);
            
            // the wall above the task
            let min_start = items.iter()
                // excluding the task itself, only keep the tasks before this one
                .filter(|(_, o)| o.id != t.id && o.start_minute + o.duration <= start)
                // for each such left neighbor, take the end time 
                .map(|(_, o)| o.start_minute + o.duration)
                // use the latest end, if there is nothing, use 0 (start of day)
                .max().unwrap_or(0);
            
            // same stuff as min_start, but the start of the task after it
            // meaning the wall below it
            let max_end = items.iter()
                .filter(|(_, o)| o.id != t.id && o.start_minute >= end)
                .map(|(_, o)| o.start_minute)
                .min().unwrap_or(DAY_MINUTES);
            
            // only write the stuff if it actually changed
            // (since slint rewrites the whole thing if it changes)
            if t.min_start != min_start || t.max_end != max_end {
                // cloning is necessary bacause t is a shared ref
                // mutating through it is not an option
                let mut t2 = t.clone();
                t2.min_start = min_start;
                t2.max_end = max_end;
                // pass the updated task clone
                sched.set_row_data(*row, t2);
            }
        }
    }
}

pub fn run() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let app_handle = app.as_weak();

    // set max length constants
    app.set_max_title(MAX_TITLE as i32);
    app.set_max_description(MAX_DESCRIPTION as i32);
    app.set_max_duration(MAX_DURATION as i32);
    
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

fn build_task(form: &TaskForm) -> Result<Task, String> {
    // this function is pretty self explanatory
    // it just tries to put in the value from TaskForm
    // if it failes, it throws an error that gets displayed later

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
    // if no date is selected, return None,
    // else, the date as DateThing (my Date object from time.rs) gets returned as Some

    if day == 0 || month == 0 || year == 0 {
        return None;
    }

    Some(DateThing::new(day as u8, month as u8, year as u16).unwrap())
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

