pub mod error;

use std::ops::Deref;
use std::sync::{Arc, RwLock};
use esp_idf_svc::nvs::{EspCustomNvsPartition, EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use synchronized::synchronized;
use crate::security::error::SecurityError;

const NVS_NAMESPACE: &str = "secure";
const ACCESS_TOKEN_LENGTH: usize = 256;

const WIFI_PASSWORD_KEY: &str = "wifi_password";
const WIFI_DEFAULT_PASSWORD: &str = "scheduler-rs"; /* should be minimum 8 chars */

const USER_PASSWORD_KEY: &str = "api_password";
const USER_DEFAULT_PASSWORD: &str = "scheduler-rs";


static mut SECURITY_CONTEXT: Option<SecurityContext> = None;

pub type SecurityResult<T> = Result<T, SecurityError>;

pub struct SecurityContext {
    access_token: Arc<RwLock<String>>,
}

impl SecurityContext {
    pub fn get() -> Result<&'static SecurityContext, EspError> {
        let access_token: String = Self::generate_access_token();

        let this: Self = Self {
            access_token: Arc::new(RwLock::new(access_token)),
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
        let actual_password: String = SecurityContext::nvs_read_str(USER_PASSWORD_KEY)
            .map_err(SecurityError::EspError)?
            .unwrap_or(String::from(USER_DEFAULT_PASSWORD));

        if password != actual_password {
            return Err(SecurityError::WrongCredentials);
        }

        let access_token = self.access_token
            .read()
            .map_err(|_| SecurityError::ReadLockError)?;

        Ok(access_token.clone())
    }

    pub fn get_access_point_password(&self) -> Result<String, EspError> {
        let password: String = SecurityContext::nvs_read_str(WIFI_PASSWORD_KEY)?
            .unwrap_or(String::from(WIFI_DEFAULT_PASSWORD));

        Ok(password)
    }

    pub fn set_access_point_password(&self, new_password: &str) -> Result<(), EspError> {
        SecurityContext::nvs_write_str(WIFI_PASSWORD_KEY, new_password)
    }

    pub fn reset_access_point_password(&self) -> Result<(), EspError> {
        SecurityContext::nvs_write_str(WIFI_PASSWORD_KEY, WIFI_DEFAULT_PASSWORD)
    }

    pub fn set_api_password(&self, new_password: &str) -> Result<(), EspError> {
        SecurityContext::nvs_write_str(USER_PASSWORD_KEY, new_password)
    }

    pub fn reset_api_password(&self) -> Result<(), EspError> {
        SecurityContext::nvs_write_str(USER_PASSWORD_KEY, USER_DEFAULT_PASSWORD)
    }

    pub fn is_valid_wifi_password(&self, password: &str) -> Result<bool, EspError> {
        let actual_password: String = self.get_access_point_password()?;

        Ok(password == actual_password)
    }

    pub fn is_valid_access_token_token(&self, access_token: &str) -> SecurityResult<bool> {
        let actual_access_token = self.access_token
            .read()
            .map_err(|_| SecurityError::ReadLockError)?;

        Ok(access_token == *actual_access_token)
    }

    fn generate_access_token() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(ACCESS_TOKEN_LENGTH)
            .map(char::from)
            .collect()
    }


    fn nvs_read_str(key: &str) -> Result<Option<String>, EspError> {
        synchronized!{
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
    }

    fn nvs_write_str(key: &str, value: &str) -> Result<(), EspError> {
        synchronized! {
            log::info!("Writing string '{value}' to NVS by key '{key}'.");
            let esp_nvs_partition: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take()?;
            let mut esp_nvs: EspNvs<NvsDefault> = EspNvs::new(esp_nvs_partition, NVS_NAMESPACE, true)?;
            log::info!("NVS initialized.");

            esp_nvs.set_str(key, value)
        }
    }
}
