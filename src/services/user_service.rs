use std::collections::HashMap;

use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::QueryStream;

use crate::contexts::{connection::Transaction, jwt_session::Claims, model::{ActionResult, DataPribadiRequest}};

pub struct UserService;

impl UserService {
    pub async fn save_data_pribadi(connection: web::Data<Pool<ConnectionManager>>, request: DataPribadiRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {
        let mut result: ActionResult<HashMap<String, String>, String> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT AutoNID, Stage 
                    FROM UserKyc 
                    WHERE AutoNID = @P1"#, &[&session.auth_usernid]).await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            let stage: i32 = row.get("Stage").unwrap_or(0);

                            if stage > 1 {
                                result.message = "Stage has ben first or 1".to_owned();
                                println!("Satu");
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                                r#"UPDATE [dbo].[AuthUser]
                                                    set [OTPGeneratedLink] = @P2, [disableLogin] = @P3,
                                                    [ActivateTime] = @P4
                                                    WHERE AuthUserNID = @P1"#,
                                                &[
                                                    &row.get("AuthUserNID").unwrap_or(0),
                                                    &"",
                                                    &false,
                                                    &chrono::Utc::now(),
                                                ],
                                            ).await {
                                                result.error = Some(format!("Fauled: {:?}", err));
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
                                    result.message = "Activation successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                    
                        } else {
                            result.message = format!("No user found for email");
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

        return result;
    }
}