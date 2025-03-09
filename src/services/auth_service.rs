use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::{NaiveDateTime, TimeZone, Utc};
use tiberius::QueryStream;

use crate::contexts::{connection::DbTransaction, crypto::encrypt_text, model::{ActionResult, LoginRequest, RegisterRequest, WebUser}};

pub struct AuthService;

impl AuthService {
    pub async fn login(connection: web::Data<Pool<ConnectionManager>>,request: LoginRequest) -> ActionResult<WebUser> {
        
        let mut result = ActionResult::default();
        let enc_password = encrypt_text(&request.password);

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT AuthUserNID, Email, Handphone, disableLogin, Picture, RegisterDate FROM AuthUser 
                    WHERE Email = @P1 AND Password = @P2"#, &[&request.email, &enc_password]).await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            result.result = true;
                            result.message = format!("Welcome {}", request.email);
                            result.data = Some(WebUser{
                                auth_usernid: row.get("AuthUserNID").unwrap_or(0),
                                email: row.get::<&str, _>("Email").map_or_else(|| "".to_string(), |s| s.to_string()),
                                mobile_phone: row.get::<&str, _>("Handphone").map_or_else(|| "".to_string(), |s| s.to_string()),
                                disabled_login: row.get("disableLogin").unwrap_or(false),
                                picture: Some(row.get::<&str, _>("Picture").map_or_else(|| "".to_string(), |s| s.to_string())),
                                register_date: row
                                    .get::<NaiveDateTime, _>("RegisterDate")
                                    .map(|dt| dt.and_utc()) // 🔥 Konversi ke DateTime<Utc>
                                    .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap()), // Default jika kosong
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
            }, 
        }
    }

    pub async fn register(
        pool: web::Data<Pool<ConnectionManager>>,
        request: RegisterRequest,
    ) -> ActionResult<()> {
        let mut result = ActionResult::default();
    
        // Enkripsi password sebelum disimpan
        let enc_password = encrypt_text(&request.password);
    
        // Mulai transaksi
        match DbTransaction::begin(&pool).await {
            Ok(trans) => {
                {
                    // Scope agar conn_guard di-drop sebelum commit
                    let mut conn_guard = trans.conn.lock().await;
                    if let Some(ref mut conn) = *conn_guard {
                        // Insert ke tabel `AuthUser`
                        let query1 = conn
                            .execute(
                                r#"INSERT INTO AuthUser (AuthUserNID, Email, Handphone, Password, RegisterDate) 
                                VALUES (@P1, @P2, @P3, @P4, GETUTCDATE())"#,
                                &[],
                            )
                            .await;
                        if let Err(err) = query1 {
                            result.error = format!("Failed to insert AuthUser: {:?}", err).into();
                            return result;
                        }
    
                        // Insert ke tabel `CIFRequest`
                        let query2 = conn
                            .execute(
                                r#"INSERT INTO CIFRequest (CIFRequestNID, AuthUserNID, Status, CreatedAt) 
                                VALUES (@P1, @P2, 'Pending', GETUTCDATE())"#,
                                &[],
                            )
                            .await;
                        if let Err(err) = query2 {
                            result.error = format!("Failed to insert CIFRequest: {:?}", err).into();
                            return result;
                        }
    
                        // Insert ke tabel `UserKyc`
                        let query3 = conn
                            .execute(
                                r#"INSERT INTO UserKyc (UserKycNID, AuthUserNID, VerificationStatus, UpdatedAt) 
                                VALUES (@P1, @P2, 'Unverified', GETUTCDATE())"#,
                                &[],
                            )
                            .await;
                        if let Err(err) = query3 {
                            result.error = format!("Failed to insert UserKyc: {:?}", err).into();
                            return result;
                        }
                    } else {
                        result.error = format!("Failed to get database connection").into();
                        return result;
                    }
                } // 🔥 `conn_guard` keluar dari scope dan otomatis di-drop
    
                // Commit transaksi setelah semua query berhasil
                if let Err(err) = trans.commit().await {
                    result.error = format!("Failed to commit transaction: {:?}", err).into();
                    return result;
                }
    
                result.result = true;
                result.message = "User registered successfully".to_string();
            }
            Err(err) => {
                result.error = format!("Failed to start transaction: {:?}", err).into();
            }
        }
    
        result
    }
       
    
}