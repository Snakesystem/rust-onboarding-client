use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::{NaiveDateTime, TimeZone, Utc};
use tiberius::QueryStream;
use tokio_stream::StreamExt;
use crate::contexts::{connection::DbTransaction, crypto::encrypt_text, model::{ActionResult, LoginRequest, RegisterRequest, WebUser}};
use super::generic_service::GenericService;

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

    pub async fn register(connection: web::Data<Pool<ConnectionManager>>, request: RegisterRequest) -> ActionResult<()> {
        
        let mut result: ActionResult<()> = ActionResult::default();
        let enc_password = encrypt_text(&request.password);

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT Email FROM AuthUser WHERE Email = @P1"#, &[&request.email]
                ).await;
        
                match query_result {
                    Ok(mut stream) => {
                        if stream.next().await.transpose().is_ok() {
                            // Jika email sudah ada, return error
                            result.error = Some("Email sudah terdaftar".to_string());
                            return result;
                        }
                    }
                    Err(err) => {
                        result.error = Some(format!("Query error: {}", err));
                        return result;
                    }
                }
            }
            Err(err) => {
                result.error = Some(format!("Database connection error: {}", err));
                return result;
            }
        }
        match DbTransaction::begin(&connection).await {
            Ok(trans) => {
                let auto_nid: i32;

                // 🔴 Scope pertama: Insert ke UserKyc
                match trans.conn.lock().await.as_mut() {
                    Some(conn) => {
                        match conn.query(
                            r#"INSERT INTO [dbo].[UserKyc] 
                            ([Email],[MobilePhone],[Fullname],[BankAccountNumber],[BankAccountHolder],
                            [QuestionRDN],[Sales],[BankName],[Stage],[CIFNID],[ChangeNID],[PendingCIFNID],
                            [IsRejected],[IsFinished],[IsRevised],[IsImported],[SaveTime],[LastUpdate],[SaveIpAddress])
                            OUTPUT INSERTED.AutoNID
                            VALUES
                            (@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16,@P17,@P18,@P19)"#,
                            &[
                                &request.email, &request.mobile_phone, &request.fullname,
                                &request.bank_account_number, &request.bank_account_holder,
                                &request.question_rdn, &request.sales, &request.bank_name,
                                &1i32, &0i32, &0i32, &0i32, &false, &false, &false, &false,
                                &chrono::Utc::now(), &chrono::Utc::now(), &request.app_ipaddress,
                            ],
                        ).await {
                            Ok(rows) => {
                                auto_nid = match rows.into_row().await {
                                    Ok(Some(row)) => row.get("AutoNID").unwrap_or(0),
                                    _ => {
                                        result.error = Some("Failed to get AutoNID from UserKyc".into());
                                        return result;
                                    }
                                };
                            }
                            Err(err) => {
                                result.error = Some(format!("Failed to insert UserKyc: {:?}", err));
                                return result;
                            }
                        }
                    }
                    None => {
                        result.error = Some("Failed to get connection from pool".into());
                        return result;
                    }
                }

                // 🔴 Scope kedua: Insert ke AuthUser
                match trans.conn.lock().await.as_mut() {
                    Some(conn) => {
                        if let Err(err) = conn.execute(
                            r#"INSERT INTO [dbo].[AuthUser] 
                            ([WebCIFNID],[Email],[Handphone],[ActivateCode],[Password],[RegisterDate],
                            [disableLogin],[OTPGeneratedLink],[OTPGeneratedLinkDate],[Picture],[Sub], [ClientNCategory])
                            VALUES (@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12)"#,
                            &[
                                &auto_nid, &request.email, &request.mobile_phone, &GenericService::random_string(20),
                                &enc_password, &chrono::Utc::now(), &true,
                                &GenericService::random_string(70), &chrono::Utc::now(),
                                &"", &"", &request.client_category,
                            ],
                        ).await {
                            result.error = Some(format!("Failed to insert AuthUser: {:?}", err));
                            return result;
                        }
                    }
                    None => {
                        result.error = Some("Failed to get database connection".into());
                        return result;
                    }
                }

                // 🔴 Scope ketiga: Insert ke TableRequest
                match trans.conn.lock().await.as_mut() {
                    Some(conn) => {
                        if let Err(err) = conn.execute(
                            r#"INSERT INTO [dbo].[TableRequest] ([WebCIFNID], [Referal]) VALUES (@P1, @P2)"#,
                            &[&auto_nid, &request.referal],
                        ).await {
                            result.error = Some(format!("Failed to insert TableRequest: {:?}", err));
                            return result;
                        }
                    }
                    None => {
                        result.error = Some("Failed to get database connection".into());
                        return result;
                    }
                }

                // 🔵 Commit transaksi
                if let Err(err) = trans.commit().await {
                    result.error = Some(format!("Failed to commit transaction: {:?}", err));
                    return result;
                }

                result.result = true;
                result.message = "User registered successfully".to_string();
            }
            Err(err) => {
                result.error = Some(format!("Failed to start transaction: {:?}", err));
            }
        }

        return result;
    }

    pub async  fn activation_user(connection: web::Data<Pool<ConnectionManager>>, otp_link: String) -> ActionResult<()> {

        let mut result: ActionResult<()> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT AuthUserNID, OTPGeneratedLinkDate FROM AuthUser OTPGeneratedLinkDate Email = @P1"#, &[&otp_link]
                ).await;
        
                match query_result {
                    Ok(mut stream) => {
                        if stream.next().await.transpose().is_ok() {
                            println!("Hahahaha")
                        }
                    }
                    Err(err) => {
                        result.error = Some(format!("Query error: {}", err));
                        return result;
                    }
                }
            }
            Err(err) => {
                result.error = Some(format!("Database connection error: {}", err));
                return result;
            }
        }

        return result;
        
    }
}