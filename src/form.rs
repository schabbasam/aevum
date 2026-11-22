
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
