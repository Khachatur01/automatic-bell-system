use crate::alarm::{Alarm, AlarmMatching};
use esp_idf_svc::hal::gpio::{IOPin, Input, PinDriver};

pub enum SynchronizeBy<INT: IOPin> {
    Delay {
        /* synchronize on every N seconds */
        seconds: u32
    },
    Interruption {
        alarm: Option<(Alarm, AlarmMatching)>,
        /* synchronize on every pin interrupt */
        pin: PinDriver<'static, INT, Input>
    }
}
