use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use chrono::{DateTime, Datelike, Month, Timelike, Utc, Weekday};

#[derive(Clone)]
pub enum AlarmMarcher<T: Eq + Hash + Clone> {
    Ignore,
    Match(HashSet<T>),
    DoNotMatch(HashSet<T>)
}

#[derive(Clone)]
pub struct Alarm {
    pub year: AlarmMarcher<u16>,
    pub month: AlarmMarcher<Month>,
    pub month_day: AlarmMarcher<u8>,
    pub week_day: AlarmMarcher<Weekday>,

    pub hour: AlarmMarcher<u8>,
    pub minute: AlarmMarcher<u8>,
    pub second: AlarmMarcher<u8>,

    /* Impulse length in milliseconds when alarm triggered. */
    pub impulse_length_millis: u64,
}

impl Alarm {
    pub fn matches(&self, datetime: &DateTime<Utc>) -> bool {
        let month: Month = match Month::try_from(datetime.month() as u8) {
            Ok(month) => month,
            Err(_) => return false /* todo: add log */
        };

        Alarm::segment_matches(&self.year, &(datetime.year() as u16)) &&
        Alarm::segment_matches(&self.month, &month) &&
        Alarm::segment_matches(&self.month_day, &(datetime.day() as u8)) &&
        Alarm::segment_matches(&self.week_day, &datetime.weekday()) &&

        Alarm::segment_matches(&self.hour, &(datetime.hour() as u8)) &&
        Alarm::segment_matches(&self.minute, &(datetime.minute() as u8)) &&
        Alarm::segment_matches(&self.second, &(datetime.second() as u8))
    }

    fn segment_matches<T: Eq + Hash + Clone>(alarm_matcher: &AlarmMarcher<T>, segment: &T) -> bool {
        match alarm_matcher {
            AlarmMarcher::Ignore => true,
            AlarmMarcher::Match(match_set) => {
                match_set.contains(segment)
            }
            AlarmMarcher::DoNotMatch(do_not_match_set) => {
                !do_not_match_set.contains(segment)
            }
        }
    }
}
