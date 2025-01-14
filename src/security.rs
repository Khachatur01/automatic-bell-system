pub mod error;

use esp_idf_svc::nvs::{EspCustomNvsPartition, EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use crate::security::error::SecurityError;

const NVS_PARTITION: &str = "secure";
const NVS_NAMESPACE: &str = "secure";
const ACCESS_TOKEN_LENGTH: usize = 256;

const WIFI_PASSWORD_KEY: &str = "wifi_password";
const WIFI_DEFAULT_PASSWORD: &str = "scheduler-rs";

const USER_PASSWORD_KEY: &str = "api_password_key";
const USER_DEFAULT_PASSWORD: &str = "scheduler-rs";


static mut SECURITY_CONTEXT: Option<SecurityContext> = None;

pub type SecurityResult<T> = Result<T, SecurityError>;

pub struct SecurityContext {
    access_token: String,
}

impl SecurityContext {
    pub fn new() -> Result<&'static SecurityContext, EspError> {
        let this = Self {
            access_token: Self::generate_access_token(),
        };

        unsafe {
            Ok(SECURITY_CONTEXT.get_or_insert(this))
        }
    }

    /**
     * Check if provided credentials are correct before giving an access token.
     * Right using only password, because this system only should have one user, but keeping username parameter for future improvements (e.g. JWT token generation)
     */
    pub fn get_access_token(&self, _username: &str, password: &str) -> SecurityResult<String> {
        let actual_password: String = self
            .nvs_read_str(USER_PASSWORD_KEY)
            .map_err(SecurityError::EspError)?
            .unwrap_or(String::from(USER_DEFAULT_PASSWORD));

        if password != actual_password {
            return Err(SecurityError::WrongCredentials);
        }

        Ok(self.access_token.clone())
    }

    pub fn get_access_point_password(&self) -> Result<String, EspError> {
        let password: String = self
            .nvs_read_str(WIFI_PASSWORD_KEY)?
            .unwrap_or(String::from(WIFI_DEFAULT_PASSWORD));

        Ok(password)
    }

    pub fn set_access_point_password(&mut self, new_password: &str) -> Result<(), EspError> {
        self.nvs_write_str(WIFI_PASSWORD_KEY, new_password)
    }

    pub fn set_api_password(&mut self, new_password: &str) -> Result<(), EspError> {
        self.nvs_write_str(USER_PASSWORD_KEY, new_password)
    }

    pub fn is_valid_wifi_password(&self, password: &str) -> Result<bool, EspError> {
        let actual_password: String = self.get_access_point_password()?;

        Ok(password == actual_password)
    }

    pub fn is_valid_access_token_token(&self, access_token: &str) -> bool {
        access_token == self.access_token
    }

    fn generate_access_token() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(ACCESS_TOKEN_LENGTH)
            .map(char::from)
            .collect()
    }


    fn nvs_read_str(&self, key: &str) -> Result<Option<String>, EspError> {
        log::info!("Reading from NVS by key '{key}'.");;
        let esp_nvs_partition: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take()?;
        let esp_nvs: EspNvs<NvsDefault> = EspNvs::new(esp_nvs_partition, NVS_NAMESPACE, true)?;
        log::info!("NVS initialized.");

        if let Some(password_length) = esp_nvs.str_len(key)? {
            log::info!("Length of value is {password_length} for key '{key}'.");
            let mut password_buffer: Vec<u8> = vec![0; password_length];

            let password: Option<&str> = esp_nvs.get_str(key, password_buffer.as_mut_slice())?;

            Ok(password.map(String::from))
        } else {
            log::warn!("Value in NVS does not exist for key {key}.");
            Ok(None)
        }
    }

    fn nvs_write_str(&mut self, key: &str, value: &str) -> Result<(), EspError> {
        log::info!("Writing string '{value}' to NVS by key '{key}'.");
        let esp_nvs_partition: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take()?;
        let mut esp_nvs: EspNvs<NvsDefault> = EspNvs::new(esp_nvs_partition, NVS_NAMESPACE, true)?;
        log::info!("NVS initialized.");

        esp_nvs.set_str(key, value)
    }
}
