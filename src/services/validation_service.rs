pub mod validator {
    use std::collections::HashMap;

    use validator::{ValidationError, ValidationErrors};

    pub fn required(value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            let mut error = ValidationError::new("required");
            error.message = Some("Ini field is required".into());
            return Err(error);
        }
        Ok(())
    }

    pub fn format_validation_errors(errors: &ValidationErrors) -> HashMap<String, String> {
        let mut formatted_errors = HashMap::new();
    
        for (field, field_errors) in errors.field_errors() {
            if let Some(error) = field_errors.first() {
                let error_message = match error.code.as_ref() {
                    "required" => format!("{} is required", capitalize(&field)),
                    _ => error.message.clone().unwrap_or_else(|| "Invalid value".into()).to_string(),
                };
    
                formatted_errors.insert(field.to_string(), error_message);
            }
        }
    
        formatted_errors
    }
    
    // Helper untuk kapitalisasi huruf pertama
    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }    
    
}