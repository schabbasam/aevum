
use crate::core::task::Task;
use crate::core::time::Duration;
use crate::core::color::Color;
use crate::TaskForm;

pub fn build_task(form: &TaskForm) -> Result<Task, String> {
    // this function is pretty self explanatory
    // it just tries to put in the value from TaskForm
    // if it failes, it throws an error that gets displayed later

    let mut task = match Task::new(form.title.to_string()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error when creating a new task: {e}");
            return Err("invalid title".to_string());
        }
    };

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

