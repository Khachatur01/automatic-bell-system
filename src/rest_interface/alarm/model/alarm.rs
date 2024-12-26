use chrono::{Month, Weekday};
use clock::alarm::{Alarm, AlarmMarcher};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use crate::rest_interface::alarm::model::alarm_id::AlarmIdDTO;
use crate::schedule_system::alarm_id::AlarmId;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub enum WeekdayDTO {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<WeekdayDTO> for Weekday {
    fn from(weekday_dto: WeekdayDTO) -> Self {
        match weekday_dto {
            WeekdayDTO::Monday => Weekday::Mon,
            WeekdayDTO::Tuesday => Weekday::Tue,
            WeekdayDTO::Wednesday => Weekday::Wed,
            WeekdayDTO::Thursday => Weekday::Thu,
            WeekdayDTO::Friday => Weekday::Fri,
            WeekdayDTO::Saturday => Weekday::Sat,
            WeekdayDTO::Sunday => Weekday::Sun,
        }
    }
}

impl From<Weekday> for WeekdayDTO {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Mon => WeekdayDTO::Monday,
            Weekday::Tue => WeekdayDTO::Tuesday,
            Weekday::Wed => WeekdayDTO::Wednesday,
            Weekday::Thu => WeekdayDTO::Thursday,
            Weekday::Fri => WeekdayDTO::Friday,
            Weekday::Sat => WeekdayDTO::Saturday,
            Weekday::Sun => WeekdayDTO::Sunday,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub enum MonthDTO {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl From<MonthDTO> for Month {
    fn from(month_dto: MonthDTO) -> Self {
        match month_dto {
            MonthDTO::January => Month::January,
            MonthDTO::February => Month::February,
            MonthDTO::March => Month::March,
            MonthDTO::April => Month::April,
            MonthDTO::May => Month::May,
            MonthDTO::June => Month::June,
            MonthDTO::July => Month::July,
            MonthDTO::August => Month::August,
            MonthDTO::September => Month::September,
            MonthDTO::October => Month::October,
            MonthDTO::November => Month::November,
            MonthDTO::December => Month::December,
        }
    }
}

impl From<Month> for MonthDTO {
    fn from(month: Month) -> Self {
        match month {
            Month::January => MonthDTO::January,
            Month::February => MonthDTO::February,
            Month::March => MonthDTO::March,
            Month::April => MonthDTO::April,
            Month::May => MonthDTO::May,
            Month::June => MonthDTO::June,
            Month::July => MonthDTO::July,
            Month::August => MonthDTO::August,
            Month::September => MonthDTO::September,
            Month::October => MonthDTO::October,
            Month::November => MonthDTO::November,
            Month::December => MonthDTO::December,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub enum AlarmMarcherDTO<T: Eq + Hash + Clone> {
    Ignore,
    Match(HashSet<T>),
    DoNotMatch(HashSet<T>),
}

impl<Dto: Eq + Hash + Clone, T: Eq + Hash + Clone + From<Dto>> From<AlarmMarcherDTO<Dto>> for AlarmMarcher<T> {
    fn from(alarm_matcher_dto: AlarmMarcherDTO<Dto>) -> Self {
        match alarm_matcher_dto {
            AlarmMarcherDTO::Ignore => AlarmMarcher::Ignore,
            AlarmMarcherDTO::Match(hashset) => AlarmMarcher::Match(
                /* map all dto elements of hashmap */
                hashset
                    .into_iter()
                    .map(Into::into)
                    .collect()
            ),
            AlarmMarcherDTO::DoNotMatch(hashset) => AlarmMarcher::DoNotMatch(
                /* map all dto elements of hashmap */
                hashset
                    .into_iter()
                    .map(Into::into)
                    .collect()
            )
        }
    }
}

impl<Dto: Eq + Hash + Clone + From<T>, T: Eq + Hash + Clone> From<AlarmMarcher<T>> for AlarmMarcherDTO<Dto> {
    fn from(alarm_matcher: AlarmMarcher<T>) -> Self {
        match alarm_matcher {
            AlarmMarcher::Ignore => AlarmMarcherDTO::Ignore,
            AlarmMarcher::Match(hashset) => AlarmMarcherDTO::Match(
                /* map all dto elements of hashmap */
                hashset
                    .into_iter()
                    .map(Into::into)
                    .collect()
            ),
            AlarmMarcher::DoNotMatch(hashset) => AlarmMarcherDTO::DoNotMatch(
                /* map all dto elements of hashmap */
                hashset
                    .into_iter()
                    .map(Into::into)
                    .collect()
            )
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AlarmDTO {
    pub year: AlarmMarcherDTO<u16>,
    pub month: AlarmMarcherDTO<MonthDTO>,
    pub month_day: AlarmMarcherDTO<u8>,
    pub week_day: AlarmMarcherDTO<WeekdayDTO>,

    pub hour: AlarmMarcherDTO<u8>,
    pub minute: AlarmMarcherDTO<u8>,
    pub second: AlarmMarcherDTO<u8>,
}

impl From<AlarmDTO> for Alarm {
    fn from(alarm_dto: AlarmDTO) -> Self {
        Self {
            year: alarm_dto.year.into(),
            month: alarm_dto.month.into(),
            month_day: alarm_dto.month_day.into(),
            week_day: alarm_dto.week_day.into(),

            hour: alarm_dto.hour.into(),
            minute: alarm_dto.minute.into(),
            second: alarm_dto.second.into(),
        }
    }
}

impl From<Alarm> for AlarmDTO {
    fn from(alarm: Alarm) -> Self {
        Self {
            year: alarm.year.into(),
            month: alarm.month.into(),
            month_day: alarm.month_day.into(),
            week_day: alarm.week_day.into(),

            hour: alarm.hour.into(),
            minute: alarm.minute.into(),
            second: alarm.second.into(),
        }
    }
}
