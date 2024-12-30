use std::collections::HashMap;
use clock::alarm::Alarm;
use crate::model::alarm::alarm_with_id::AlarmWithIdDTO;
use crate::schedule_system::alarm_id::AlarmId;

pub trait ToAlarmsWithId {
    fn to_alarms_with_id(self) -> Vec<AlarmWithIdDTO>;
}

impl ToAlarmsWithId for HashMap<AlarmId, Alarm> {
    fn to_alarms_with_id(self) -> Vec<AlarmWithIdDTO> {
        let alarms_count: usize = self.capacity();
        self
            .into_iter()
            .fold(Vec::with_capacity(alarms_count), |mut accumulator, (alarm_id, alarm)| {
                let alarm_with_id_dto: AlarmWithIdDTO = (alarm_id.into(), alarm.into()).into();

                accumulator.push(alarm_with_id_dto);
                accumulator
            })
    }
}
