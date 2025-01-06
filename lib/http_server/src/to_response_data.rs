use serde::Serialize;

pub trait ToResponseData where Self: Serialize {
    fn to_response_data(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

impl ToResponseData for String {
    fn to_response_data(&self) -> String {
        self.to_string()
    }
}

impl ToResponseData for &str {
    fn to_response_data(&self) -> String {
        self.to_string()
    }
}

impl<T: Serialize> ToResponseData for Vec<T> {}
