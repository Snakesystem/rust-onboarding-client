use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use crate::contexts::{crypto::encrypt_text, model::{ActionResult, LoginRequest, RegisterRequest, WebUser}};

pub struct AuthService;

impl AuthService {
    pub async fn login(connection: web::Data<Pool<ConnectionManager>>,request: LoginRequest) -> ActionResult<WebUser> {
        
        let mut result = ActionResult::default();
        let enc_password = encrypt_text(&request.password);

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result = conn.query("SELECT AuthUserNID, Email FROM AuthUser WHERE Email = @P1 AND Password = @P2", &[&request.email, &enc_password]).await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            result.result = true;
                            result.message = format!("Welcome {}", request.email);
                            result.data = Some(WebUser{
                                auth_usernid: row.get(0).unwrap_or(0),
                                email: row.get::<&str, _>(1).map_or_else(|| "".to_string(), |s| s.to_string())
                            });

                            return result;
                        } else {
                            result.message = format!("No user found for email: {}", request.email);
                            return result;
                        }
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
                        return result;
                    },
                }
            },
            Err(err) => {
                result.error = format!("Internal Server error: {:?}", err).into();
                return result;
            }, // Gagal mendapatkan koneksi
        }
    }

    pub async fn register(request: RegisterRequest) -> ActionResult<()> {
        let mut result = ActionResult::default();

        // Logika validasi (misal cek email kosong)
        if request.email.is_empty() {
            result.result = false;
            result.message = "Invalid Request".to_string();
            result.error = Some("Email is required".to_string());
            return result;
        }

        result.result = true;
        result.message = "Register successfully".to_string();
        result
    }
}