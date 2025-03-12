use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use validator::Validate;
use crate::services::validation_service::validator::{
    required, valid_phone_number, valid_name, valid_number_card, required_int,
    required_datetime, validate_base64_image, valid_password
};
// pub struct DateTimeConverter;

// impl DateTimeConverter {
//     pub fn from_string(date_str: &str) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
//         let format = "%Y-%m-%d %H:%M:%S";
//         chrono::NaiveDateTime::parse_from_str(date_str, format)
//     }
// }

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_password"))]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_password"))]
    pub password: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_phone_number"))]
    pub mobile_phone: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_name"))]
    pub full_name: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"))]
    pub bank_account_number: Option<String>,
    #[validate(custom(function = "required"))]
    pub bank_name: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_name"))]
    pub bank_account_holder: Option<String>,
    #[validate(custom(function = "required_int"))]
    pub question_rdn: i32,
    #[serde(default)]
    pub sales: i32,
    #[serde(default)]
    pub referal: String,
    pub client_category: u8,
    #[serde(default)]
    pub app_ipaddress: String
}

#[derive(Debug, Deserialize, Validate)]
pub struct DataPribadiRequest {
    #[validate(custom(function = "required"), email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_phone_number"))]
    pub mobile_phone: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_name"))]
    pub full_name: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_name"))]
    pub mother_name: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"), length(min = 15, message = "Minimum 15 characters"))]
    pub idcard_number: Option<String>,
    #[validate(custom(function = "required_int"))]
    pub nationality: i32,
    #[validate(custom(function = "required_int"))]
    pub sex: i32,
    #[validate(custom(function = "required_int"))]
    pub residence_status: i32,
    #[validate(custom(function = "required_int"))]
    pub beneficiary_owner: i32,
    #[validate(custom(function = "required"))]
    pub birth_place: Option<String>,
    #[validate(custom(function = "required_datetime"))]
    #[serde(deserialize_with  = "deserialize_date_only")]
    pub birth_date: Option<DateTime<Utc>>,
    #[validate(custom(function = "required"))]
    pub birth_country: Option<String>,
    #[validate(custom(function = "required_int"))]
    pub religion: i32,
    #[validate(custom(function = "required_int"))]
    pub marital_status: i32,
    #[validate(custom(function = "required_int"))]
    pub education: i32,
    pub copy_id: Option<bool>,
    #[validate(custom(function = "required_datetime"))]
    #[serde(deserialize_with  = "deserialize_date_only")]
    pub idcard_expireddate  : Option<DateTime<Utc>>,
    #[validate(custom(function = "required"))]
    pub idcard_country  : Option<String>,

    #[validate(custom(function = "validate_base64_image"))]
    pub idcard_file: String,
    #[validate(custom(function = "validate_base64_image"))]
    pub selfie_file: String,
    #[validate(custom(function = "validate_base64_image"))]
    pub signature_file: String,

    // #region ID CARD FIELD INFORMAATION
    #[validate(custom(function = "required_int"))]
    pub idcard_city: i32,
    #[validate(custom(function = "required"))]
    pub idcard_district: Option<String>,
    #[validate(custom(function = "required"))]
    pub idcard_subdistrict: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"))]
    pub idcard_rt: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"),)]
    pub idcard_rw: Option<String>,
    #[validate(custom(function = "required"))]
    pub idcard_address: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"),)]
    pub idcard_zipcode: Option<String>,
    // #endregion

    // #region DOMICILE FIELD INFORMAATION
    #[validate(custom(function = "required_int"))]
    pub domicile_city: i32,
    #[validate(custom(function = "required"))]
    pub domicile_district: Option<String>,
    #[validate(custom(function = "required"))]
    pub domicile_subdistrict: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"))]
    pub domicile_rt: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"),)]
    pub domicile_rw: Option<String>,
    #[validate(custom(function = "required"))]
    pub domicile_address: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_number_card"),)]
    pub domicile_zipcode: Option<String>,
    // #endregion
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(required, email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(custom(function = "required"), custom(function = "valid_password"))]
    pub password: Option<String>,
    pub reset_password_key: String
}

#[derive(Debug, Serialize)]
pub struct ActionResult<T, E> {
    pub result: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

// Implementasi Default
impl<T, E> Default for ActionResult<T, E> {
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

fn deserialize_date_only<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str: Option<String> = Option::deserialize(deserializer)?;
    if let Some(date) = date_str {
        let naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map_err(serde::de::Error::custom)?;
        let datetime = Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap());
        return Ok(Some(datetime));
    }
    Ok(None)
}