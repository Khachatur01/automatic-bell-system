use esp_idf_svc::hal::gpio::{IOPin, Input, PinDriver};


pub enum SynchronizeBy<INT: IOPin> {
    Delay {
        /* synchronize on every N seconds */
        seconds: u32
    },
    Interrupt {
        /* synchronize on every pin interrupt */
        pin: PinDriver<'static, INT, Input>
    }
}
