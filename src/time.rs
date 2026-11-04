// this file contains all structs and enums about time

pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

pub enum Recurrence {
    Daily { interval: u32 },
    Weekly {
        interval: u32,
        days: Vec<Weekday>
    },
}

struct Date {
    day:    u8,
    month:  u8,
    year:   u16,
}
impl Date {
    pub fn new(day: u8, month: u8, year: u16) -> Result<Self, String> {
        if year == 0 {
            return Err("invalid year".to_string());
        }

        let max_days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) {
                    29
                } else {
                    28
                }
            },
            _ => return Err("invalid month".to_string()),
        };

        if day == 0 || day > max_days {
            return Err("invalid day".to_string());
        }

        Ok(Self {day, month, year})
    }
}


struct Time {
    hour:   u8,
    minute: u8,
}
impl Time {
    pub fn new(hour: u8, minute: u8) -> Result<Self, String> {
        if hour < 24 && minute < 60 {
            Ok(Self {hour, minute})
        } else {
            Err("out of valid range".to_string())
        }
    }
}

// even tough it is just one attribute, it is better to have a struct
// because we can add methods to it later on
pub struct Duration {
    minutes: u16,
}
