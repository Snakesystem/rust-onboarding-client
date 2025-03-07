use serde::{Deserialize, Serialize};

pub struct DateTimeConverter;

impl DateTimeConverter {
    pub fn from_string(date_str: &str) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
        let format = "%Y-%m-%d %H:%M:%S";
        chrono::NaiveDateTime::parse_from_str(date_str, format)
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub mobile_phone: u128,
    pub fullname: String,
    pub bank_account_number: u128,
    pub bank_name: String,
    pub bank_account_holder: String,
    pub question_rdn: u8,
    #[serde(default)]
    pub sales: u32,
    #[serde(default)]
    pub referal: u16,
    pub client_category: u8
}

#[derive(Debug, Serialize)]
pub struct ActionResult<T> {
    pub result: bool,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// Implementasi Default
impl<T> Default for ActionResult<T> {
    fn default() -> Self {
        Self {
            result: false, // Default-nya false
            message: String::new(),
            data: None,
            error: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WebUser {
    pub auth_usernid: i32,
    pub email: String,
}