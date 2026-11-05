
use std::sync::atomic::{AtomicU64, Ordering};

use crate::time::*;

pub struct Task {
    // the essential attributes
    id:             u64,
    title:          String,
    date:           Date,
    // optional attributes
    description:    Option<String>,
    start_time:     Option<Time>,
    duration:       Option<Duration>,
    recurrence:     Option<Recurrence>,
    // completion status
    completed:      bool,
}

const MAX_TITLE: usize = 16;
const MAX_DESCRIPTION: usize = 100;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl Task {
    
    pub fn new(title: impl Into<String>, date: Date) -> Result<Self, String> {
        let title = title.into();

        if title.is_empty() {
            return Err("title cannot be empty".to_string());
        }
        if title.chars().count() > MAX_TITLE {
            return Err("title too long".to_string());
        }

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        Ok(Self {
            id, title, date,
            description: None,
            start_time: None,
            duration: None,
            recurrence: None,
            completed: false
        })
    }

    // ---- all getters ----
    pub fn id(&self) -> u64 { self.id }
    pub fn title(&self) -> &str { &self.title }
    pub fn date(&self) -> &Date { &self.date }
    
    // as_ref() converts from &Option<Type> to Option<&Type>, meaning it's perfect for this
    pub fn description(&self) -> Option<&String> { self.description.as_ref() }
    pub fn start_time(&self) -> Option<&Time> { self.start_time.as_ref() }
    pub fn duration(&self) -> Option<&Duration> { self.duration.as_ref() }
    pub fn recurrence(&self) -> Option<&Recurrence> { self.recurrence.as_ref() }

    pub fn completed(&self) -> bool { self.completed }

    // ---- all setters ----
    pub fn set_title(&mut self, title: impl Into<String>) -> Result<(), String> {
        // using impl Into<String> and .into() to accept anything convertible to a String
        let title = title.into();
        if title.is_empty() {
            return Err("title connot be empty".to_string());
        }
        if title.chars().count() > MAX_TITLE {
            return Err("title too long".to_string());
        }
        self.title = title;
        Ok(())
    }

    pub fn set_description(&mut self, desc: impl Into<String>) -> Result<(), String> {
        let desc = desc.into();
        
        if desc.chars().count() > MAX_DESCRIPTION {
            return Err("description too long".to_string());
        }
        self.description = Some(desc);
        Ok(())
    }

    pub fn set_date(&mut self, date: Date) {
        self.date = date;
    }

    pub fn set_start_time(&mut self, time: Option<Time>) {
        self.start_time = time;
    }

    pub fn set_duration(&mut self, duration: Option<Duration>) {
        self.duration = duration;
    }

    pub fn set_recurrence(&mut self, recurrence: Option<Recurrence>) {
        self.recurrence = recurrence;
    }
    
    pub fn set_completed(&mut self, done: bool) {
        self.completed = done;
    }
}

