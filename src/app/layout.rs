
use slint::{ Model, VecModel };
use crate::TaskForm;

pub const DAY_MINUTES: i32 = 1440;
pub const STEP: i32 = 15;
pub const DEFAULT_DURATION: i32 = 60;

pub fn normalize_duration(d: i32) -> i32 {
    // if duration d is smaller or equal 0, it is the default duration
    let d = if d <= 0 { DEFAULT_DURATION } else { d };
    // first it rounds up to a multiple of STEP
    // then clamp to STEP and DAY_MINUTES (cant get hight or lower than these)
    ((d + STEP - 1) / STEP * STEP).clamp(STEP, DAY_MINUTES) // clamp(min, max)
}

// insert a y-coordinate where you want to put the task
// returns the index where the new item should be
pub fn insertion_index(col: &VecModel<TaskForm>, y: f32, hour_h: f32, min_h: f32, gap: f32) -> usize {
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

pub fn insertion_index_shifted(
    col: &VecModel<TaskForm>, y: f32, hour_h: f32, min_h: f32,
    gap: f32, cur: i32, extra: f32
) -> usize {
    let mut top = gap;
    for (i, t) in col.iter().enumerate() {
        if i as i32 == cur {
            top += extra;
        }
        let h = task_height_px(t.duration, hour_h, min_h);
        if y < top + h / 2.0 {
            return i;
        }
        top += h + gap;
    }
    col.row_count()
}

// return the free spot closest to wish
pub fn find_free_start(sched: &VecModel<TaskForm>, id: i32, col: i32, dur: i32, wish: i32) -> Option<i32> {
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

pub fn update_all_limits(sched: &VecModel<TaskForm>) {
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

pub fn task_height_px(duration: i32, hour_h: f32, min_h: f32) -> f32 {
    let height = (duration as f32 / 60.0) * hour_h;
    height.max(min_h)
}


