
use crate::core::task::Task;
use crate::core::time::Duration;
use crate::core::color::Color;
use crate::TaskDraft;

pub fn build_task(draft: &TaskDraft) -> Result<Task, String> {
    // this function is pretty self explanatory
    // it just tries to put in the value from TaskForm
    // if it failes, it throws an error that gets displayed later

    let mut task = match Task::new(draft.title.to_string()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error when creating a new task: {e}");
            return Err("invalid title".to_string());
        }
    };

    let desc = if draft.description.is_empty() {
        None
    } else {
        Some(draft.description.to_string())
    };

    if let Err(e) = task.set_description(desc) {
        eprintln!("error when setting description: {e}");
        return Err("description too long".to_string());
    }

    if draft.duration == 0 {
        task.set_duration(None);
    } else {
        let dura = Duration::new(draft.duration as u16).map_err(|e| {
            eprintln!("error when creating Duration: {e}");
            "duration can't be longer than a day".to_string()
        })?;

        task.set_duration(Some(dura));
    }

    if draft.color.is_empty() {
        task.set_color(None);
    } else {
        let c = Color::from_hex(&draft.color);

        if let Err(e) = c {
            eprintln!("error when creating Color: {e}");
            return Err("invalid color".to_string());
        } else {
            task.set_color(Some(c.unwrap()));
        }
    }
    
    // cagetory comes later here

    task.set_completed(draft.completed);

    Ok(task)
}

