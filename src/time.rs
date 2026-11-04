// this file contains all structs and enums about time
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl Weekday {
    pub fn to_number(&self) -> u8 {
        match self {
            Weekday::Monday     => 0,
            Weekday::Tuesday    => 1,
            Weekday::Wednesday  => 2,
            Weekday::Thursday   => 3,
            Weekday::Friday     => 4,
            Weekday::Saturday   => 5,
            Weekday::Sunday     => 6,
        }

    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Recurrence {
    Daily { interval: u32 },
    Weekly {
        interval: u32,
        days: Vec<Weekday>
    },
}
impl Recurrence {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Recurrence::Daily {interval} => {
                // the * is necessary to dereferentize the variable
                if *interval == 0 {
                    return Err("invalid interval".to_string());
                }
            }
            Recurrence::Weekly {interval, days} => {
                if *interval == 0 {
                    return Err("invalid interval".to_string());
                }

                // check for duplicate or no days
                let unique_count: HashSet<&Weekday> = days.iter().collect();
                if days.is_empty() {
                    return Err("no days".to_string());
                }
                if unique_count.len() != days.len() {
                    return Err("duplicate days".to_string());
                }
            }
        }

        Ok(())
    }
    pub fn daily(interval: u32) -> Result<Self, String> {
        if interval == 0 {
            return Err("invalid interval".to_string());
        }

        Ok(Self::Daily{interval})
    }

    pub fn weekly(interval: u32, days: Vec<Weekday>) -> Result<Self, String> {
        if interval == 0 {
            return Err("invalid interval".to_string());
        }
        // check for duplicate or no days
        let unique_count: HashSet<&Weekday> = days.iter().collect();
        if days.is_empty() {
            return Err("no days".to_string());
        }
        if unique_count.len() != days.len() {
            return Err("duplicate days".to_string());
        }

        Ok(Self::Weekly {interval, days})
    }

    pub fn interval(&self) -> u32 {
        match &self {
            Self::Daily {interval} => *interval,
            Self::Weekly {interval, ..} => *interval,
        }
    }

    // these square brackets mean it is a slice, it can be an array or vector or whatever
    pub fn days(&self) -> Option<&[Weekday]> {
        match &self {
            Self::Daily {..} => None,
            Self::Weekly {days, ..} => Some(days),
        }
    }

    pub fn set_interval(&mut self, new_interval: u32) -> Result<(), String> {
        if new_interval == 0 {
            return Err("invalid interval".to_string());
        }
        
        match self {
            Self::Daily {interval} => {
                *interval = new_interval;
            }
            Self::Weekly {interval, ..} => {
                *interval = new_interval;
            }
        }

        Ok(())
    }

    pub fn set_days(&mut self, new_days: Vec<Weekday>) -> Result<(), String> {
        
        let unique_count: HashSet<&Weekday> = new_days.iter().collect();
        if new_days.is_empty() {
            return Err("no days".to_string());
        }
        if unique_count.len() != new_days.len() {
            return Err("duplicate days".to_string());
        }

        match self {
            Self::Daily {..} => {
                return Err("not a Weekly".to_string());
            }
            Self::Weekly {days, ..} => {
                *days = new_days;
            }
        }
        Ok(())
    }

}

pub struct Date {
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


pub struct Time {
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
impl Duration {
    pub fn new(minutes: u16) -> Result<Self, String> {
        if minutes == 0 {
            return Err("invalid minutes".to_string());
        }

        Ok(Self {minutes})
    }
}
