use std::net::Ipv4Addr;
use esp_idf_svc::eventloop::{EspEventLoop, EspSystemEventLoop, System};
use esp_idf_svc::hal::modem;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{AccessPointConfiguration, Configuration, EspWifi};

pub struct AccessPoint<'a> {
    wifi: EspWifi<'a>,
}

impl<'a> AccessPoint<'a> {
    pub fn new(modem: modem::Modem) -> Result<Self, EspError> {
        let sys_loop: EspEventLoop<System> = EspSystemEventLoop::take()?;
        let nvs: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take()?;

        let mut wifi: EspWifi = EspWifi::new(modem, sys_loop, Some(nvs))?;
        wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration::default()))?;

        Ok(Self { wifi })
    }

    pub fn is_started(&self) -> Result<bool, EspError> {
        let is_started: bool = self.wifi.is_started()?;

        Ok(is_started)
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        self.wifi.start()?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), EspError> {
        self.wifi.stop()?;

        Ok(())
    }

    pub fn get_ipv4(&self) -> Result<Ipv4Addr, EspError> {
        let ipv4: Ipv4Addr = self.wifi.ap_netif().get_ip_info()?.ip;

        Ok(ipv4)
    }
}
