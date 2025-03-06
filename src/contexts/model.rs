pub struct DateTimeConverter;

impl DateTimeConverter {
    pub fn from_string(date_str: &str) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
        let format = "%Y-%m-%d %H:%M:%S";
        chrono::NaiveDateTime::parse_from_str(date_str, format)
    }
}