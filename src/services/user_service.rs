use std::collections::HashMap;

use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::QueryStream;

use crate::contexts::{connection::Transaction, jwt_session::Claims, model::{ActionResult, DataBankRequest, DataPribadiRequest}};

pub struct UserService;

impl UserService {
    pub async fn save_data_pribadi(connection: web::Data<Pool<ConnectionManager>>, request: DataPribadiRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

        let mut result: ActionResult<HashMap<String, String>, String> = ActionResult::default();
        let current_stage: i32 = 1;

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
                            let auto_nid: i32 = row.get("AutoNID").unwrap_or(0);

                            if stage > current_stage {
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
                                        r#"UPDATE [dbo].[UserKyc]
                                            SET [Stage] = @P1, [Fullname] = @P2, [Nationality] = @P3, [IDCardNumber] = @P4, [IDCardExpireDate] = @P5, 
                                                [Sex] = @P6,[BirthDate] = @P7, [BirthPlace] = @P8, [BirthCountry] = @P9, [MotherName] = @P10, [Religion] = @P11, 
                                                [MaritalStatus] = @P12,[Education] = @P13, [IDCardCity] = @P14, [IDCardDistrict] = @P15, 
                                                [IDCardSubdistrict] = @P16, [IDCardRT] = @P17, [IDCardRW] = @P18, [IDCardZipcode] = @P19, 
                                                [IDCardAddress] = @P20, [CopyID] = @P21, [DomicileCity] = @P22, [DomicileDistrict] = @P23, 
                                                [DomicileSubdistrict] = @P24, [DomicileRT] = @P25, [DomicileRW] = @P26, [DomicileZipcode] = @P27, 
                                                [DomicileAddress] = @P28, [IDCardFile] = @P29, [SelfieFile] = @P30, [SignatureFile] = @P31, IDCardCountry = @P32
                                            WHERE AutoNID = @P33"#,
                                                &[
                                                    &2i32,
                                                    &request.full_name,
                                                    &request.nationality,
                                                    &request.idcard_number,
                                                    &request.idcard_expireddate,
                                                    &request.sex,
                                                    &request.birth_date,
                                                    &request.birth_place,
                                                    &request.birth_country,
                                                    &request.mother_name,
                                                    &request.religion,
                                                    &request.marital_status,
                                                    &request.education,
                                                    &request.idcard_city,
                                                    &request.idcard_district,
                                                    &request.idcard_subdistrict,
                                                    &request.idcard_rt,
                                                    &request.idcard_rw,
                                                    &request.idcard_zipcode,
                                                    &request.idcard_address,
                                                    &request.copy_id,
                                                    &request.domicile_city,
                                                    &request.domicile_district,
                                                    &request.domicile_subdistrict,
                                                    &request.domicile_rt,
                                                    &request.domicile_rw,
                                                    &request.domicile_zipcode,
                                                    &request.domicile_address,
                                                    &"request.idcard_file",
                                                    &"request.selfie_file",
                                                    &"request.signature_file",
                                                    &request.idcard_country,
                                                    &auto_nid
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
                    
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                                r#"UPDATE [dbo].[TableRequest] SET [ResidencyNStatus] = @P1, [CIFInvestorBeneficiaryOwner] = @P2 WHERE [AutoNID] = @P3"#,
                                                &[&request.residence_status, &request.beneficiary_owner, &auto_nid],
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
                                    result.message = "Update personal data successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                    
                        } else {
                            result.message = format!("No user found for email");
                        }
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
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

    pub async fn save_data_bank(connection: web::Data<Pool<ConnectionManager>>, request: DataBankRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

        let mut result: ActionResult<HashMap<String, String>, String> = ActionResult::default();
        let current_stage: i32 = 2;

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
                            let auto_nid: i32 = row.get("AutoNID").unwrap_or(0);

                            if stage > current_stage {
                                result.message = "Stage has ben second or 2".to_owned();
                                println!("Satu");
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                            r#"UPDATE [dbo].[UserKYC]
                                                SET [Stage] = @P1, [QuestionRDN] = @P2, [BankName] = @P3, [BankAccountHolder] = @P4, [BankAccountNumber] = @P5, [BankBranch] = @P6
                                            WHERE AutoNID = @P7"#,
                                                &[
                                                    &3i32,
                                                    &request.question_rdn,
                                                    &request.bank_name,
                                                    &request.bank_account_holder,
                                                    &request.bank_account_number,
                                                    &request.bank_branch,
                                                    &auto_nid
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
                                    result.message = "Update personal data successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                    
                        } else {
                            result.message = format!("No user found for email");
                        }
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
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

    pub async fn save_data_pekerjaan(connection: web::Data<Pool<ConnectionManager>>, request: DataPribadiRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

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
                            let auto_nid: i32 = row.get("AutoNID").unwrap_or(0);

                            if stage > 2 {
                                result.message = "Stage has ben second or 2".to_owned();
                                println!("Satu");
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                        r#"UPDATE [dbo].[UserKYC]
                                                SET [Stage] = @P1, [QuestionRDN] = @P2, [BankName] = @P3, [BankAccountHolder] = @P4, [BankAccountNumber] = @P5, [BankBranch] = @P6
                                                WHERE AutoNID = @P6"#,
                                                &[
                                                    &3i32,
                                                    &request.full_name,
                                                    &request.nationality,
                                                    &request.idcard_number,
                                                    &request.idcard_expireddate,
                                                    &request.sex,
                                                    &request.birth_date,
                                                    &request.birth_place,
                                                    &request.birth_country,
                                                    &request.mother_name,
                                                    &request.religion,
                                                    &request.marital_status,
                                                    &request.education,
                                                    &request.idcard_city,
                                                    &request.idcard_district,
                                                    &request.idcard_subdistrict,
                                                    &request.idcard_rt,
                                                    &request.idcard_rw,
                                                    &request.idcard_zipcode,
                                                    &request.idcard_address,
                                                    &request.copy_id,
                                                    &request.domicile_city,
                                                    &request.domicile_district,
                                                    &request.domicile_subdistrict,
                                                    &request.domicile_rt,
                                                    &request.domicile_rw,
                                                    &request.domicile_zipcode,
                                                    &request.domicile_address,
                                                    &"request.idcard_file",
                                                    &"request.selfie_file",
                                                    &"request.signature_file",
                                                    &request.idcard_country,
                                                    &auto_nid
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
                                    result.message = "Update personal data successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                    
                        } else {
                            result.message = format!("No user found for email");
                        }
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
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

    pub async fn save_data_pendukung(connection: web::Data<Pool<ConnectionManager>>, request: DataPribadiRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

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
                            let auto_nid: i32 = row.get("AutoNID").unwrap_or(0);

                            if stage > 2 {
                                result.message = "Stage has ben second or 2".to_owned();
                                println!("Satu");
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                        r#"UPDATE [dbo].[UserKYC]
                                                SET [Stage] = @P1, [QuestionRDN] = @P2, [BankName] = @P3, [BankAccountHolder] = @P4, [BankAccountNumber] = @P5, [BankBranch] = @P6
                                                WHERE AutoNID = @P6"#,
                                                &[
                                                    &3i32,
                                                    &request.full_name,
                                                    &request.nationality,
                                                    &request.idcard_number,
                                                    &request.idcard_expireddate,
                                                    &request.sex,
                                                    &request.birth_date,
                                                    &request.birth_place,
                                                    &request.birth_country,
                                                    &request.mother_name,
                                                    &request.religion,
                                                    &request.marital_status,
                                                    &request.education,
                                                    &request.idcard_city,
                                                    &request.idcard_district,
                                                    &request.idcard_subdistrict,
                                                    &request.idcard_rt,
                                                    &request.idcard_rw,
                                                    &request.idcard_zipcode,
                                                    &request.idcard_address,
                                                    &request.copy_id,
                                                    &request.domicile_city,
                                                    &request.domicile_district,
                                                    &request.domicile_subdistrict,
                                                    &request.domicile_rt,
                                                    &request.domicile_rw,
                                                    &request.domicile_zipcode,
                                                    &request.domicile_address,
                                                    &"request.idcard_file",
                                                    &"request.selfie_file",
                                                    &"request.signature_file",
                                                    &request.idcard_country,
                                                    &auto_nid
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
                                    result.message = "Update personal data successfully".to_string();
                                }
                                Err(err) => {
                                    result.error = Some(format!("Failed to start transaction: {:?}", err));
                                }
                            }
                    
                        } else {
                            result.message = format!("No user found for email");
                        }
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
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