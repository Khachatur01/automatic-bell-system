use ds323x::{Alarm1Matching, DayAlarm1, Hours};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alarm {
    /// Day of the month [1-31]
    pub day: u8,
    /// Hour [0-23]
    pub hour: u8,
    /// Minute [0-59]
    pub minute: u8,
    /// Second [0-59]
    pub second: u8,
}

impl From<Alarm> for DayAlarm1 {
    fn from(alarm: Alarm) -> Self {
        DayAlarm1 {
            day: alarm.day,
            hour: Hours::H24(alarm.hour),
            minute: alarm.minute,
            second: alarm.second
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlarmMatching {
    /// Alarm once per second.
    OncePerSecond,
    /// Alarm when seconds match.
    SecondsMatch,
    /// Alarm when minutes and seconds match.
    MinutesAndSecondsMatch,
    /// Alarm when hours, minutes and seconds match.
    HoursMinutesAndSecondsMatch,
    /// Alarm when date/weekday, hours, minutes and seconds match.
    AllMatch,
}

impl From<AlarmMatching> for Alarm1Matching {
    fn from(alarm_matching: AlarmMatching) -> Self {
        match alarm_matching {
            AlarmMatching::OncePerSecond => Alarm1Matching::OncePerSecond,
            AlarmMatching::SecondsMatch => Alarm1Matching::SecondsMatch,
            AlarmMatching::MinutesAndSecondsMatch => Alarm1Matching::MinutesAndSecondsMatch,
            AlarmMatching::HoursMinutesAndSecondsMatch => Alarm1Matching::HoursMinutesAndSecondsMatch,
            AlarmMatching::AllMatch => Alarm1Matching::AllMatch,
        }
    }
}
