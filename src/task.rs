
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
    category:       Option<u64>,
    color:          Option<Color>,
    // completion status
    completed:      bool,
}

const MAX_TITLE: usize = 16;
const MAX_DESCRIPTION: usize = 100;
const MAX_CATEGORY_NAME: usize = 20;

// static means this variable can only exist once for the entire program
// AtomicU64 is able to be changed safely and simultaneously
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_CATEGORY_ID: AtomicU64 = AtomicU64::new(1);

impl Task {

    pub fn validate_title(title: &str) -> Result<(), String> {
        if title.trim().is_empty() {
            return Err("title cannot be empty".to_string());
        }

        if title.chars().count() > MAX_TITLE {
            return Err("title too long".to_string());
        }

        Ok(())
    }

    pub fn validate_description(description: &str) -> Result<(), String> {
        if description.chars().count() > MAX_DESCRIPTION {
            return Err("description too long".to_string());
        }

        Ok(())
    }
    
    pub fn new(title: impl Into<String>, date: Date) -> Result<Self, String> {
        let title = title.into();

        Self::validate_title(&title)?;

        // every time this thing is read, it returns the old value and adds +1 to it for later
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);

        Ok(Self {
            id, title, date,
            description: None,
            start_time: None,
            duration: None,
            recurrence: None,
            category: None,
            color: None,
            completed: false,
        })
    }

    // ---- all getters ----
    pub fn id(&self) -> u64 { self.id }
    pub fn title(&self) -> &str { &self.title }
    pub fn date(&self) -> &Date { &self.date }
    
    // as_ref() converts from &Option<Type> to Option<&Type>, meaning it's perfect for this
    // as_deref() does the same from Option<&String> to Option<&str> (string slices better)
    pub fn description(&self) -> Option<&str> { self.description.as_deref() }
    pub fn start_time(&self) -> Option<&Time> { self.start_time.as_ref() }
    pub fn duration(&self) -> Option<&Duration> { self.duration.as_ref() }
    pub fn recurrence(&self) -> Option<&Recurrence> { self.recurrence.as_ref() }

    pub fn completed(&self) -> bool { self.completed }

    // ---- all setters ----
    pub fn set_title(&mut self, title: impl Into<String>) -> Result<(), String> {
        // using impl Into<String> and .into() to accept anything convertible to a String
        let title = title.into();
        Self::validate_title(&title)?;
        self.title = title;
        Ok(())
    }

    pub fn set_description(&mut self, desc: Option<String>) -> Result<(), String> {
        if let Some(ref d) = desc {
            Self::validate_description(d)?;
        }

        self.description = desc;
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

pub struct Category {
    id:     u64,
    name:   String,
    color:  Color,
}
impl Category {
    pub fn new(name: impl Into<String>, color: Color) -> Result<Self, String> {
        let name = name.into();
        
        if name.trim().is_empty() {
            return Err("category name cannot be empty".to_string());
        }

        if name.chars().count() > MAX_CATEGORY_NAME {
            return Err("category name too long".to_string());
        }

        let id = NEXT_CATEGORY_ID.fetch_add(1, Ordering::Relaxed);

        Ok(Self {id, name, color})
    }

    // ---- getters ----
    pub fn id(&self) -> u64 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn color(&self) -> Color { self.color }

    // ---- setters ----
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<(), String> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err("category name cannot be empty".to_string());
        }

        if name.chars().count() > MAX_CATEGORY_NAME {
            return Err("category name too long".to_string());
        }

        self.name = name;
        Ok(())
    }

    pub fn set_color(&mut self, color: Color) {
        // no validation necessary here 
        self.color = color;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    // beautiful rgb color values from 0 to 255
    red:    u8,
    green:  u8,
    blue:   u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        // fortunately we don't need validation because u8 can't be out of range
        Self {red: r, green: g, blue: b}
    }

    pub fn red(&self) -> u8 { self.red }
    pub fn green(&self) -> u8 { self.green }
    pub fn blue(&self) -> u8 { self.blue }

    pub fn to_hex(&self) -> String {
        // formatting it as an uppercase hexadecimal string
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        // strip the leading # if it exists
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        
        // check if the hex is ascii
        if !hex.is_ascii() {
            return Err("hex color must contain only ASCII characters".to_string());
        }

        if hex.len() != 6 {
            return Err("invalid hex length, expected 6 characters".to_string());
        }

        // helper function
        fn parse_pair(pair: &str) -> Result<u8, String> {
            // from_str_radix converts pair from a base 16 (aka hex) to a u8 as a Result<>
            // if it is an Err, it gets converted to a String with map_err()
            u8::from_str_radix(pair, 16).map_err(|e| format!("failed to parse '{}': {}", pair, e))
        }

        Ok(Color {
            // string slicing: extracting 2 bytes at a time and checking if it is an Err or Ok
            red:    parse_pair(&hex[0..2])?,
            green:  parse_pair(&hex[2..4])?,
            blue:   parse_pair(&hex[4..6])?,
        })
    }

}
