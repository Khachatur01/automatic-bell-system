use esp_idf_svc::sys::{settimeofday, time_t, timeval, timezone};
use esp_idf_svc::systime::EspSystemTime;
use std::time::Duration;

pub trait SystemTime {
    fn get_time(&self) -> Duration;
    fn set_time(&mut self, duration: Duration);
}

impl SystemTime for EspSystemTime {
    fn get_time(&self) -> Duration {
        self.now()
    }

    fn set_time(&mut self, duration: Duration) {
        let time_value: timeval = timeval {
            tv_sec: duration.as_secs() as time_t,
            tv_usec: 0
        };

        let time_zone: timezone = timezone {
            tz_minuteswest: 0,
            tz_dsttime: 0, /* DST_NONE */
        };

        unsafe {
            settimeofday(&time_value as *const _, &time_zone as *const _);
        }
    }
}
