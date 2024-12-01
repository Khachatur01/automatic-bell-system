use std::collections::HashSet;
use std::hash::Hash;
use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};

pub enum AlarmMarcher<T: Eq + Hash> {
    Ignore,
    Match(HashSet<T>),
    DoNotMatch(HashSet<T>)
}

pub struct Alarm {
    pub year: AlarmMarcher<u16>,
    pub month: AlarmMarcher<u8>,
    pub month_day: AlarmMarcher<u8>,
    pub week_day: AlarmMarcher<Weekday>,

    pub hour: AlarmMarcher<u8>,
    pub minute: AlarmMarcher<u8>,
    pub second: AlarmMarcher<u8>,
}

impl Alarm {
    pub fn matches(&self, datetime: &DateTime<Utc>) -> bool {
        Alarm::segment_matches(&self.year, &(datetime.year() as u16)) &&
        Alarm::segment_matches(&self.month, &(datetime.month() as u8)) &&
        Alarm::segment_matches(&self.month_day, &(datetime.day() as u8)) &&
        Alarm::segment_matches(&self.week_day, &datetime.weekday()) &&

        Alarm::segment_matches(&self.hour, &(datetime.hour() as u8)) &&
        Alarm::segment_matches(&self.minute, &(datetime.minute() as u8)) &&
        Alarm::segment_matches(&self.second, &(datetime.second() as u8))
    }

    fn segment_matches<T: Eq + Hash>(alarm_matcher: &AlarmMarcher<T>, segment: &T) -> bool {
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
