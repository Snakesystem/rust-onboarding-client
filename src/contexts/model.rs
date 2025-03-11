use chrono::{DateTime, Utc};
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
    pub mobile_phone: i64,
    pub fullname: String,
    pub bank_account_number: i64,
    pub bank_name: String,
    pub bank_account_holder: String,
    pub question_rdn: i32,
    #[serde(default)]
    pub sales: i32,
    #[serde(default)]
    pub referal: String,
    pub client_category: u8,
    #[serde(default)]
    pub app_ipaddress: String
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

#[derive(Debug, Serialize, Clone)]
pub struct WebUser {
    pub auth_usernid: i32,
    pub email: String,
    pub mobile_phone: String,
    pub disabled_login: bool,
    pub picture: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub register_date: chrono::DateTime<Utc>
}

#[derive(Debug, Serialize, Clone)]
pub struct Company {
    pub company_id: String,
    pub company_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ListData {
    pub data_id: i32,
    pub code: String,
    pub description: String,
}

fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&formatted)
}