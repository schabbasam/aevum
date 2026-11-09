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
    pub fn to_number(self) -> u8 {
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
    
    pub fn from_number(n: u8) -> Result<Self, String> {
        match n {
            0 => Ok(Weekday::Monday),
            1 => Ok(Weekday::Tuesday),
            2 => Ok(Weekday::Wednesday),
            3 => Ok(Weekday::Thursday),
            4 => Ok(Weekday::Friday),
            5 => Ok(Weekday::Saturday),
            6 => Ok(Weekday::Sunday),
            _ => Err("invalid number".to_string())
        }

    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let recurrence = Self::Daily {interval};
        recurrence.validate()?;

        Ok(recurrence)
    }

    pub fn weekly(interval: u32, days: Vec<Weekday>) -> Result<Self, String> {
        let recurrence = Self::Weekly {interval, days};
        recurrence.validate()?;

        Ok(recurrence)
    }

    pub fn interval(&self) -> u32 {
        match &self {
            Self::Daily {interval}
            | Self::Weekly {interval, ..} => *interval,
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
        
        match self {
            Self::Daily {..} => {
                return Err("not a Weekly".to_string());
            }
            Self::Weekly {days, ..} => {
                let unique_count: HashSet<&Weekday> = new_days.iter().collect();
                if new_days.is_empty() {
                    return Err("no days".to_string());
                }
                if unique_count.len() != new_days.len() {
                    return Err("duplicate days".to_string());
                }

                *days = new_days;
            }
        }
        Ok(())
    }

}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date {
    day:    u8,
    month:  u8,
    year:   u16,
}
impl Date {

    fn is_leap_year(year: u16) -> bool {
        if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) {
            return true;
        }
        return false;
    }

    fn days_in_month(month: u8, year: u16) -> Result<u8, String> {
        let max_days: u8 = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if is_leap_year(year) {
                    29
                } else {
                    28
                }
            },
            _ => return Err("invalid month".to_string()),
        };
        
        Ok(max_days)
    }

    fn days_since_epoch(&self) -> u64 {
        
    }

    pub fn validate(day: u8, month: u8, year: u16) -> Result<(), String> {

        if year == 0 {
            return Err("invalid year".to_string());
        }

        if day == 0 || day > days_in_month(month, year)? {
            return Err("invalid day".to_string());
        }

        Ok(())
    }

    pub fn new(day: u8, month: u8, year: u16) -> Result<Self, String> {
        Self::validate(day, month, year)?;
        Ok(Self {day, month, year})
    }
    
    pub fn day(self) -> u8 {
        self.day
    }
    pub fn month(self) -> u8 {
        self.month
    }
    pub fn year(self) -> u16 {
        self.year
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    hour:   u8,
    minute: u8,
}
impl Time {
    pub fn validate(hour: u8, minute: u8) -> Result<(), String> {
        if hour < 24 && minute < 60 {
            Ok(())
        } else {
            Err("out of valid range".to_string())
        }
    }

    pub fn new(hour: u8, minute: u8) -> Result<Self, String> {
        Self::validate(hour, minute)?;
        Ok(Self {hour, minute})
    }

    pub fn hour(self) -> u8{
        self.hour
    }
    pub fn minute(self) -> u8{
        self.minute
    }

    pub fn to_minutes(self) -> u16 {
        self.hour() as u16 * 60 + self.minute() as u16
    }
}

// even tough it is just one attribute, it is better to have a struct
// because we can add methods to it later on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    minutes: u16,
}
impl Duration {
    // setters, validate and to_hours functions are unneeded
    pub fn new(minutes: u16) -> Result<Self, String> {
        if minutes == 0 {
            return Err("invalid minutes".to_string());
        }

        Ok(Self {minutes})
    }

    pub fn minutes(self) -> u16 {
        self.minutes
    }
}

// --- TEST ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recurrence() {
        // the same day twice
        let invalid_days = vec![Weekday::Monday, Weekday::Friday, Weekday::Monday];
        assert!(Recurrence::weekly(3, invalid_days.clone()).is_err());
        
        let days = vec![Weekday::Thursday, Weekday::Sunday];

        let mut a_recurrence = Recurrence::weekly(2, days.clone()).unwrap();
        assert!(a_recurrence.set_days(days.clone()).is_ok());
        assert!(a_recurrence.set_interval(9).is_ok());
        assert_eq!(a_recurrence.interval(), 9);
        assert_eq!(a_recurrence.days().unwrap(), days);

        // setter validation
        assert!(a_recurrence.set_interval(0).is_err());
        assert!(a_recurrence.set_days(vec![]).is_err());
        assert!(a_recurrence.set_days(invalid_days.clone()).is_err());

        let mut daily_recurrence = Recurrence::daily(3).unwrap();
        assert!(daily_recurrence.set_days(days.clone()).is_err());
        assert_eq!(daily_recurrence.interval(), 3);

        
        // invalid interval
        assert!(Recurrence::daily(0).is_err());
        assert!(Recurrence::weekly(0, days).is_err());

        // empty weekday vector
        assert!(Recurrence::weekly(1, Vec::new()).is_err());

    }

    #[test]
    fn test_date() {
        let valid_date = Date::new(30, 3, 2010).unwrap();
        // getters
        assert_eq!(valid_date.day(), 30);
        assert_eq!(valid_date.month(), 3);
        assert_eq!(valid_date.year(), 2010);

        // invalid values
        assert!(Date::new(0, 3, 2010).is_err());
        assert!(Date::new(2, 0, 23).is_err());
        assert!(Date::new(23, 1, 0).is_err());

        assert!(Date::new(32, 3, 890).is_err());
        assert!(Date::new(28, 13, 323).is_err());
        
        // leap year testing
        // last leap year was 2024
        assert!(Date::new(29, 2, 2024).is_ok());
        assert!(Date::new(29, 2, 2000).is_ok());

        assert!(Date::new(29, 2, 2026).is_err());
        assert!(Date::new(29, 2, 1900).is_err());

    }

    #[test]
    fn test_time() {
        let valid_time = Time::new(12, 33).unwrap();
        
        // getters
        assert_eq!(valid_time.hour(), 12);
        assert_eq!(valid_time.minute(), 33);

        
        assert_eq!(valid_time.to_minutes(), 753);

        assert!(Time::new(0, 0).is_ok());
        assert!(Time::new(23, 59).is_ok());

        // invalid times
        assert!(Time::new(24, 2).is_err());
        assert!(Time::new(22, 60).is_err());
    }
    
    #[test]
    fn test_duration() {
        let valid_duration = Duration::new(30).unwrap();

        assert_eq!(valid_duration.minutes(), 30);
        assert!(Duration::new(0).is_err());
    }
}
