use chrono::{Month, Weekday};
use clock::alarm::{Alarm, AlarmMatcher};
use http_server::to_response_data::ToResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

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

impl ToResponseData for WeekdayDTO {}

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

impl ToResponseData for MonthDTO {}

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
#[serde(tag = "tag")]
pub enum AlarmMatcherDTO<T: Eq + Hash + Clone + Serialize> {
    Ignore,
    Match { segments: HashSet<T> },
    DoNotMatch { segments: HashSet<T> },
}

impl<T: Eq + Hash + Clone + Serialize> ToResponseData for AlarmMatcherDTO<T> {}

impl<Dto, T> From<AlarmMatcherDTO<Dto>> for AlarmMatcher<T>
where
    Dto: Eq + Hash + Clone + Serialize,
    T: Eq + Hash + Clone + From<Dto>,
{
    fn from(alarm_matcher_dto: AlarmMatcherDTO<Dto>) -> Self {
        match alarm_matcher_dto {
            AlarmMatcherDTO::Ignore => AlarmMatcher::Ignore,
            AlarmMatcherDTO::Match { segments } => AlarmMatcher::Match(
                /* map all dto elements of hashmap */
                segments.into_iter().map(Into::into).collect(),
            ),
            AlarmMatcherDTO::DoNotMatch { segments } => AlarmMatcher::DoNotMatch(
                /* map all dto elements of hashmap */
                segments.into_iter().map(Into::into).collect(),
            ),
        }
    }
}

impl<Dto, T> From<AlarmMatcher<T>> for AlarmMatcherDTO<Dto>
where
    Dto: Eq + Hash + Clone + From<T> + Serialize,
    T: Eq + Hash + Clone,
{
    fn from(alarm_matcher: AlarmMatcher<T>) -> Self {
        match alarm_matcher {
            AlarmMatcher::Ignore => AlarmMatcherDTO::Ignore,
            AlarmMatcher::Match(hashset) => AlarmMatcherDTO::Match {
                /* map all dto elements of hashmap */
                segments: hashset.into_iter().map(Into::into).collect(),
            },
            AlarmMatcher::DoNotMatch(hashset) => AlarmMatcherDTO::DoNotMatch {
                /* map all dto elements of hashmap */
                segments: hashset.into_iter().map(Into::into).collect(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlarmDTO {
    pub year: AlarmMatcherDTO<u16>,
    pub month: AlarmMatcherDTO<MonthDTO>,
    pub month_day: AlarmMatcherDTO<u8>,
    pub week_day: AlarmMatcherDTO<WeekdayDTO>,

    pub hour: AlarmMatcherDTO<u8>,
    pub minute: AlarmMatcherDTO<u8>,
    pub second: AlarmMatcherDTO<u8>,

    pub impulse_length_millis: u64,
}

impl ToResponseData for AlarmDTO {}

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

            impulse_length_millis: alarm_dto.impulse_length_millis,
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

            impulse_length_millis: alarm.impulse_length_millis,
        }
    }
}
