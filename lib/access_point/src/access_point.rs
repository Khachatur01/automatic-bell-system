use std::net::Ipv4Addr;
use esp_idf_svc::eventloop::{EspEventLoop, EspSystemEventLoop, System};
use esp_idf_svc::hal::modem;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, EspWifi};

pub struct AccessPoint<'a> {
    wifi: EspWifi<'a>,
    configuration: AccessPointConfiguration
}

impl<'a> AccessPoint<'a> {
    pub fn new(modem: modem::Modem, ssid: &str, password: &str) -> Result<Self, EspError> {
        let sys_loop: EspEventLoop<System> = EspSystemEventLoop::take()?;

        let mut wifi: EspWifi = EspWifi::new(modem, sys_loop, None)?;
        let mut configuration: AccessPointConfiguration = AccessPointConfiguration::default();
        configuration.ssid = ssid.parse().unwrap();
        configuration.password = password.parse().unwrap();
        configuration.auth_method = AuthMethod::WPA3Personal;

        wifi.set_configuration(&Configuration::AccessPoint(configuration.clone()))?;

        Ok(Self { wifi, configuration })
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

    pub fn get_configuration(&self) -> &AccessPointConfiguration {
        &self.configuration
    }
}
