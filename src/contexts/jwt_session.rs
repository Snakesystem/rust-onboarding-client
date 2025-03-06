use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use bigdecimal::BigDecimal;

const SECRET_KEY: &[u8] = b"supersecretkey"; // 🔥 Ganti dengan key yang lebih aman!

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    result: bool,
    email: String,
    expired_token: i64,
    expired_date: String,
    is_active: bool,
    payment: BigDecimal,
    exp: usize,
}

impl Claims {
    pub fn new(email: String, is_active: bool, payment: BigDecimal) -> Self {
        let expired_token = Utc::now() + Duration::days(7); // Token berlaku 7 hari
        let expired_date = expired_token.format("%Y-%m-%d %H:%M:%S").to_string();
        let exp = expired_token.timestamp() as usize; // ⏳ Set exp untuk validasi JWT

        Self {
            result: true,
            email,
            expired_token: expired_token.timestamp(),
            expired_date,
            is_active,
            payment,
            exp, // 🔥 Tambahkan ke struct
        }
    }
}

// 🔥 Generate JWT Token
pub fn create_jwt(
    email: String,
    is_active: bool,
    payment: BigDecimal,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(email, is_active, payment);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )?;
    Ok(token)
}

// 🔥 Validate JWT Token
pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            let claims = token_data.claims;
            let now = Utc::now().timestamp() as usize;

            if claims.exp < now {
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature,
                ));
            }

            Ok(claims)
        }
        Err(err) => {
            println!("❌ JWT Validation Error: {:?}", err);
            Err(err)
        }
    }
}

