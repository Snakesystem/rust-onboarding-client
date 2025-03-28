use std::collections::HashMap;
use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde_json::{json, Value as JsonValue};
use tiberius::{QueryStream, Row};
use std::fmt::Write;

use crate::contexts::{
    connection::Transaction, jwt_session::Claims, model::{ActionResult, CIFFileRequest, DataBankRequest, DataBeneficiaryRequest, DataPekerjaanRequest, DataPendukungRequest, DataPribadiRequest, QueryClass, ResultList, TableDataParams, UserInfo}
};

pub struct AdminService;

impl AdminService {

    pub async fn get_table_data( allparams: TableDataParams, connection: web::Data<Pool<ConnectionManager>>) -> Result<ResultList, Box<dyn std::error::Error>> {
        let mut result = ResultList {
            total: 0,
            total_with_filter: 0,
            rows: vec![],
        };
    
        let query = Self::get_query_table(allparams.clone(), false);
    
        let mut client = connection.get().await?;
    
        if !allparams.tablename.is_empty() {
            let row: Option<Row> = client.query(query.query_total_all.clone(), &[]).await?.into_row().await?;
            if let Some(r) = row {
                result.total = r.try_get::<i64, _>("total")
                .or_else(|_| r.try_get::<i32, _>("total").map(|opt| opt.map(|v| v as i64)))? // ✅ Perbaiki konversi
                .unwrap_or(0);
            }
        }
    
        // Hitung total data yang sesuai filter
        if let Some(filter) = &allparams.filter {
            if filter != "{filter:undefined}" {
                let row: Option<Row> = client.query(query.query_total_with_filter.clone(), &[]).await?.into_row().await?;
                if let Some(r) = row {
                    result.total_with_filter = r.try_get::<i64, _>("totalWithFilter")
                    .or_else(|_| r.try_get::<i32, _>("totalWithFilter").map(|opt| opt.map(|v| v as i64)))? // ✅ Perbaiki konversi
                    .unwrap_or(0);
                }
            }
        } else {
            result.total_with_filter = result.total;
        }
    
        // write_log("INFO", &format!("Query: {}", query.query));
    
        let rows = client.query(query.query.clone(), &[]).await?.into_results().await?;
        result.rows = rows.into_iter()
            .flat_map(|r| r.into_iter())
            .map(|row| Self::row_to_json(row))  // 🔥 Ubah `Row` ke JSON
            .collect();
    
        Ok(result)
    }
    
    fn get_query_table(allparams: TableDataParams, bypass_skip: bool) -> QueryClass {
        let mut result = QueryClass {
            query: String::new(),
            query_total_all: String::new(),
            query_total_with_filter: String::new(),
        };
    
        if allparams.limit == 0 {
            return result;
        }
    
        let tablename = format!("[{}]", allparams.tablename);
        let query_total_all = format!(
            "SELECT SUM(rows) as total FROM sys.partitions WHERE object_id = OBJECT_ID('{}') AND index_id IN (0,1)",
            tablename
        );        
        let mut q_and_where = String::from(" WHERE 1=1 ");
        let mut q_order_by = String::new();
        let mut q_skip_row = String::new();
        let mut q_and_where_for_total_with_filter = String::from(" WHERE 1=1 ");
    
        // Gunakan `nidkey` sebagai primary key jika tersedia
        let q_primary_key = allparams.nidkey.clone().unwrap_or_else(|| "AutoNID".to_string());
        let q_primary_key_order = q_primary_key.clone();
    
        // Tambahkan filter jika ada
        if let Some(filter) = &allparams.filter {
            if filter != "{filter:undefined}" {
                if let Ok(filter_name) = serde_json::from_str::<HashMap<String, String>>(filter) {
                    if !filter_name.is_empty() {
                        let q_and_where_result = Self::get_query_table_where(q_and_where.clone(), filter_name);
    
                        q_and_where = q_and_where_result.clone();
                        q_and_where_for_total_with_filter = q_and_where_result.clone();
                    }
                }
            }
        }
    
        // query_total_all.push_str(&q_and_where);
    
        let query_total_with_filter = format!(
            "SELECT count(*) as totalWithFilter FROM {} {}",
            tablename, q_and_where_for_total_with_filter
        );
    
        result.query_total_with_filter = query_total_with_filter;
    
        // Sorting & pagination
        if !bypass_skip {
            if let Some(sort) = &allparams.sort {
                if let Some(order) = &allparams.order {
                    let _ = write!(q_order_by, " ORDER BY {} {}", sort, order);
                }
            } else {
                let _ = write!(q_order_by, " ORDER BY {} DESC", q_primary_key_order);
            }
    
            let _ = write!(
                q_skip_row,
                " OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                allparams.offset, allparams.limit
            );
        }
    
        // Query utama
        result.query = format!(
            "SELECT * FROM {} {} {} {}",
            tablename, q_and_where, q_order_by, q_skip_row
        );
    
        result.query_total_all = query_total_all;
        result
    }

    pub fn get_query_table_where(mut fquery: String, filter_name: HashMap<String, String>) -> String {
        for (key, value) in filter_name {
            if let Ok(temp_date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                if key.ends_with("Date") {
                    let next_date = temp_date.succ_opt().unwrap_or(temp_date);
                    let _ = write!(
                        fquery,
                        " AND {} BETWEEN '{}' AND '{}'",
                        key, value, next_date
                    );
                } else {
                    let _ = write!(fquery, " AND {} = '{}'", key, value);
                }
            } else if key.ends_with("Time") {
                let dates: Vec<&str> = value.split("to").collect();
                if dates.len() == 2 {
                    let _ = write!(
                        fquery,
                        " AND {} BETWEEN '{} 00:00:00' AND '{} 23:59:59'",
                        key, dates[0], dates[1]
                    );
                }
            } else if key.starts_with('_') || key.ends_with("NID") || key.ends_with("ID") {
                let _ = write!(fquery, " AND {} = '{}'", key, value);
            } else {
                let _ = write!(fquery, " AND {} LIKE '%{}%'", key, value);
            }
        }
    
        fquery
    }

    fn row_to_json(row: Row) -> JsonValue {
        let mut json_obj = serde_json::Map::new();
    
        for (i, col) in row.columns().iter().enumerate() {
            let col_name = col.name();
            
            // Ambil nilai dari row sesuai tipe data
            if let Ok(value) = row.try_get::<&str, _>(i) {
                json_obj.insert(col_name.to_string(), json!(value));
            } else if let Ok(value) = row.try_get::<i32, _>(i) {
                json_obj.insert(col_name.to_string(), json!(value));
            } else if let Ok(value) = row.try_get::<f64, _>(i) {
                json_obj.insert(col_name.to_string(), json!(value));
            } else if let Ok(value) = row.try_get::<bool, _>(i) {
                json_obj.insert(col_name.to_string(), json!(value));
            } else {
                json_obj.insert(col_name.to_string(), json!(null));
            }
        }
    
        JsonValue::Object(json_obj)
    }

    pub async fn get_user_info(connection: web::Data<Pool<ConnectionManager>>, session: Claims) -> ActionResult<UserInfo, String> {
        let mut result: ActionResult<UserInfo, String> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT 
                            A.*,
                            B.ContactPersonName, 
                            B.ContactPersonRelation,
                            B.CIFInvestorBeneficiaryOwner,
                            B.ResidencyNStatus
                            FROM UserKyc A
                        LEFT JOIN TableRequest B ON B.WebCIFNID = A.AutoNID 
                        WHERE A.AutoNID = @P1"#, &[&session.auth_usernid]).await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            result.result = true;
                            result.message = format!("Retrieve successfully");
                            result.data = Some(UserInfo {
                                autonid: row.get::<i32, _>("AutoNID").unwrap_or(0),
                                stage: row.get::<i32, _>("Stage").unwrap_or(0),
                                client_id: row.get::<&str, _>("CIFID").map_or_else(|| "".to_string(), |s| s.to_string()),
                                cif_id: row.get::<&str, _>("ClientID").map_or_else(|| "".to_string(), |s| s.to_string()),
                                is_revised: row.get("IsRevised").unwrap_or(false),
                                is_rejected: row.get("IsRejected").unwrap_or(false),
                                is_finished: row.get("IsFinished").unwrap_or(false),
                                account_status: row.get::<i32, _>("AccountStatus").unwrap_or(0),
                                mobile_phone: row.get::<&str, _>("MobilePhone").map_or_else(|| "".to_string(), |s| s.to_string()),
                                email: row.get::<&str, _>("Email").map_or_else(|| "".to_string(), |s| s.to_string()),
                                full_name: row.get::<&str, _>("Fullname").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_relationship: row.get::<i32, _>("SpouseRelationship").unwrap_or(0),
                                spouse_name: row.get::<&str, _>("SpouseName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                mother_name: row.get::<&str, _>("MotherName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                nationality: row.get::<i32, _>("Nationality").unwrap_or(0),
                                idcard_country: row.get::<&str, _>("IDCardCountry").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_number: row.get::<&str, _>("IDCardNumber").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_expire_date: row
                                    .get::<NaiveDateTime, _>("IDCardExpireDate")
                                    .map(|dt| dt.and_utc()) // 🔥 Konversi ke DateTime<Utc>
                                    .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap()), // Default jika kosong
                                sex: row.get::<i32, _>("Sex").unwrap_or(0),
                                birth_place: row.get::<&str, _>("BirthPlace").map_or_else(|| "".to_string(), |s| s.to_string()),
                                birth_date: row
                                    .get::<NaiveDateTime, _>("BirthDate")
                                    .map(|dt| dt.and_utc()) // 🔥 Konversi ke DateTime<Utc>
                                    .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap()), // Default jika kosong
                                birth_country: row.get::<&str, _>("BirthCountry").map_or_else(|| "".to_string(), |s| s.to_string()),
                                religion: row.get::<i32, _>("Religion").unwrap_or(0),
                                marital_status: row.get::<i32, _>("MaritalStatus").unwrap_or(0),
                                education: row.get::<i32, _>("Education").unwrap_or(0),
                                idcard_city: row.get::<i32, _>("IDCardCity").unwrap_or(0),
                                idcard_district: row.get::<&str, _>("IDCardDistrict").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_subdistrict: row.get::<&str, _>("IDCardSubdistrict").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_rt: row.get::<&str, _>("IDCardRT").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_rw: row.get::<&str, _>("IDCardRW").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_zipcode: row.get::<&str, _>("IDCardZipcode").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_address: row.get::<&str, _>("IDCardAddress").map_or_else(|| "".to_string(), |s| s.to_string()),
                                copy_id: row.get("CopyID").unwrap_or(false),
                                domicile_city: row.get::<i32, _>("DomicileCity").unwrap_or(0),
                                domicile_district: row.get::<&str, _>("DomicileDistrict").map_or_else(|| "".to_string(), |s| s.to_string()),
                                domicile_subdistrict: row.get::<&str, _>("DomicileSubdistrict").map_or_else(|| "".to_string(), |s| s.to_string()),
                                domicile_rt: row.get::<&str, _>("DomicileRT").map_or_else(|| "".to_string(), |s| s.to_string()),
                                domicile_rw: row.get::<&str, _>("DomicileRW").map_or_else(|| "".to_string(), |s| s.to_string()),
                                domicile_zipcode: row.get::<&str, _>("DomicileZipcode").map_or_else(|| "".to_string(), |s| s.to_string()),
                                domicile_address: row.get::<&str, _>("DomicileAddress").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_rdn: row.get::<i32, _>("QuestionRDN").unwrap_or(0),
                                bank_name: row.get::<&str, _>("BankName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                bank_branch: row.get::<&str, _>("BankBranch").map_or_else(|| "".to_string(), |s| s.to_string()),
                                bank_account_holder: row.get::<&str, _>("BankAccountHolder").map_or_else(|| "".to_string(), |s| s.to_string()),
                                bank_account_number: row.get::<&str, _>("BankAccountNumber").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_npwp: row.get::<i32, _>("QuestionNPWP").unwrap_or(0),
                                npwp_number: row.get::<&str, _>("NPWPNumber").map_or_else(|| "".to_string(), |s| s.to_string()),
                                npwp_reason: row.get::<&str, _>("NPWPReason").map_or_else(|| "".to_string(), |s| s.to_string()),
                                company_name: row.get::<&str, _>("CompanyName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                fund_source: row.get::<&str, _>("FundSource").map_or_else(|| "".to_string(), |s| s.to_string()),
                                fund_source_text: row.get::<&str, _>("FundSourceText").map_or_else(|| "".to_string(), |s| s.to_string()),
                                occupation: row.get::<i32, _>("Occupation").unwrap_or(0),
                                occupation_text: row.get::<&str, _>("OccupationText").map_or_else(|| "".to_string(), |s| s.to_string()),
                                nature_bussiness: row.get::<i32, _>("NatureOfBusiness").unwrap_or(0),
                                nature_bussiness_text: row.get::<&str, _>("NatureOfBusinessText").map_or_else(|| "".to_string(), |s| s.to_string()),
                                position: row.get::<i32, _>("Position").unwrap_or(0),
                                position_text: row.get::<&str, _>("PositionText").map_or_else(|| "".to_string(), |s| s.to_string()),
                                income_peranum: row.get::<i32, _>("IncomePerAnnum").unwrap_or(0),
                                question_1: row.get("Question1").unwrap_or(false),
                                question_1text: row.get::<&str, _>("Question1Text").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_2: row.get("Question2").unwrap_or(false),
                                question_2text: row.get::<&str, _>("Question2Text").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_3: row.get("Question3").unwrap_or(false),
                                question_3text: row.get::<&str, _>("Question3Text").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_4: row.get("Question4").unwrap_or(false),
                                question_4text: row.get::<&str, _>("Question4Text").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_5: row.get("Question5").unwrap_or(false),
                                question_5text: row.get::<&str, _>("Question5Text").map_or_else(|| "".to_string(), |s| s.to_string()),
                                question_6: row.get("Question6").unwrap_or(false),
                                question_6text: row.get::<&str, _>("Question6Text").map_or_else(|| "".to_string(), |s| s.to_string()),
                                invesment_objective: row.get::<i32, _>("InvestmentObjectives").unwrap_or(0),
                                risk: row.get::<i32, _>("Risk").unwrap_or(0),
                                question_fatca: row.get::<&str, _>("QuestionFATCA").map_or_else(|| "".to_string(), |s| s.to_string()),
                                fatca_1: row.get::<&str, _>("FATCA1").map_or_else(|| "".to_string(), |s| s.to_string()),
                                fatca_2: row.get::<&str, _>("FATCA2").map_or_else(|| "".to_string(), |s| s.to_string()),
                                fatca_3: row.get::<&str, _>("FATCA3").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_income_peranum: row.get::<i32, _>("SpouseIncomePerAnnum").unwrap_or(0),
                                spouse_occupation: row.get::<i32, _>("SpouseOccupation").unwrap_or(0),
                                spouse_occupation_text: row.get::<&str, _>("SpouseOccupationText").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_position: row.get::<i32, _>("SpousePosition").unwrap_or(0),
                                spouse_nature_bussiness: row.get::<i32, _>("SpouseNatureOfBusiness").unwrap_or(0),
                                spouse_fund_source: row.get::<&str, _>("SpouseFundSource").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_fund_source_text: row.get::<&str, _>("SpouseFundSourceText").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_company_name: row.get::<&str, _>("SpouseCompanyName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_company_city: row.get::<i32, _>("SpouseCompanyCity").unwrap_or(0),
                                spouse_company_address: row.get::<&str, _>("SpouseCompanyAddress").map_or_else(|| "".to_string(), |s| s.to_string()),
                                spouse_company_zipcode: row.get::<&str, _>("SpouseCompanyZipcode").map_or_else(|| "".to_string(), |s| s.to_string()),
                                idcard_file: row.get::<&str, _>("IDCardFile").map_or_else(|| "".to_string(), |s| s.to_string()),
                                selfie_file: row.get::<&str, _>("SelfieFile").map_or_else(|| "".to_string(), |s| s.to_string()),
                                signature_file: row.get::<&str, _>("SignatureFile").map_or_else(|| "".to_string(), |s| s.to_string()),
                                npwp_file: row.get::<&str, _>("NPWPFile").map_or_else(|| "".to_string(), |s| s.to_string()),
                                sales: row.get::<i32, _>("Sales").unwrap_or(0),
                                company_city: row.get::<i32, _>("CompanyCity").unwrap_or(0),
                                company_address: row.get::<&str, _>("CompanyAddress").map_or_else(|| "".to_string(), |s| s.to_string()),
                                company_zipcode: row.get::<&str, _>("CompanyZipcode").map_or_else(|| "".to_string(), |s| s.to_string()),
                                beneficiary_owner: row.get::<i32, _>("CIFInvestorBeneficiaryOwner").unwrap_or(0),
                                residence_status: row.get::<i32, _>("ResidencyNStatus").unwrap_or(0),
                            }); 

                            return result;
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

        
    }

    pub async fn save_cif_file(connection: web::Data<Pool<ConnectionManager>>, request: CIFFileRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

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

                            if stage < current_stage {
                                result.message = "Stage has ben second or 2".to_owned();
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                            r#"UPDATE [dbo].[UserKYC]
                                                SET [IDCardFile] = @P1, [SelfieFile] = @P2, [SignatureFile] = @P3
                                            WHERE AutoNID = @P4"#,
                                                &[
                                                    &request.idcard_file,
                                                    &request.selfie_file,
                                                    &request.signature_file,
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

    pub async fn save_data_pribadi(connection: web::Data<Pool<ConnectionManager>>, request: DataPribadiRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

        let mut result: ActionResult<HashMap<String, String>, String> = ActionResult::default();
        let current_stage: i32 = 1;
        let set_stage: i32;
        
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

                            if stage < current_stage {
                                result.message = "Stage has ben first or 1".to_owned();
                                return result;
                            }
                            
                            if request.beneficiary_owner == 1 {
                                set_stage = 2;
                            } else if request.beneficiary_owner == 2 {
                                set_stage = 1;
                            } else {
                                result.message = "Invalid beneficiary owner".to_owned();
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
                                                [DomicileAddress] = @P28, IDCardCountry = @P29
                                            WHERE AutoNID = @P30"#,
                                                &[
                                                    &set_stage,
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

                            if stage < current_stage {
                                result.message = "Stage has ben second or 2".to_owned();
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

    pub async fn save_data_pekerjaan(connection: web::Data<Pool<ConnectionManager>>, request: DataPekerjaanRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

        let mut result: ActionResult<HashMap<String, String>, String> = ActionResult::default();
        let curent_stage: i32 = 3;

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

                            if stage < curent_stage {
                                result.message = "Stage has ben second or 3".to_owned();
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                        r#"UPDATE [dbo].[UserKyc]  
                                                SET [Stage] = @P1, [QuestionNPWP] = @P2, [NPWPNumber] = @P3, [NPWPReason] = @P4, [CompanyAddress] = @P5,   
                                                [CompanyName] = @P6, [Fundsource] = @P7, [Occupation] = @P8, [OccupationText] = @P9,   
                                                [NatureOfBusiness] = @P10, [NatureOfBusinessText] = @P11, [Position] = @P12,  
                                                [PositionText] = @P13, [IncomePerAnnum] = @P14, [FundsourceText] = @P15, [SpouseIncomePerAnnum] = @P16,  
                                                [SpouseOccupation] = @P17, [SpousePosition] = @P18,  
                                                [SpouseNatureOfBusiness] = @P19,[SpouseCompanyName] = @P20,[SpouseCompanyCity] = @P21,  
                                                [SpouseCompanyZipcode] = @P22,[SpouseCompanyAddress] = @P23,  
                                                [SpouseFundSource] = @P24,[SpouseFundSourceText] = @P25, [SpouseOccupationText] = @P26, [SpouseName] = @P27,
                                                [SpouseRelationship] = @P28, [NPWPFile] = @P29
                                                WHERE AutoNID = @P30"#,
                                                &[
                                                    &4i32,
                                                    &request.question_npwp,
                                                    &request.npwp_number,
                                                    &request.npwp_reason,
                                                    &request.company_address,
                                                    &request.company_name,
                                                    &request.fund_source,
                                                    &request.occupation,
                                                    &request.occupation_text,
                                                    &request.nature_bussiness,
                                                    &request.nature_bussiness_text,
                                                    &request.position,
                                                    &request.position_text,
                                                    &request.income_peranum,
                                                    &request.fund_source_text,
                                                    &request.spouse_income_peranum,
                                                    &request.spouse_occupation,
                                                    &request.spouse_position,
                                                    &request.spouse_nature_bussiness,
                                                    &request.spouse_company_name,
                                                    &request.spouse_company_city,
                                                    &request.spouse_company_zipcode,
                                                    &request.spouse_company_address,
                                                    &request.spouse_fund_source,
                                                    &request.spouse_fund_source_text,
                                                    &request.spouse_occupation_text,
                                                    &request.spouse_name,
                                                    &request.spouse_relationship,
                                                    &request.npwp_file,
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

    pub async fn save_data_pendukung(connection: web::Data<Pool<ConnectionManager>>, request: DataPendukungRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

        let mut result: ActionResult<HashMap<String, String>, String> = ActionResult::default();
        let current_stage: i32 = 4;

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

                            if stage < current_stage {
                                result.message = "Stage has ben second or 4".to_owned();
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                        r#"UPDATE [dbo].[UserKyc]  
                                                SET [Stage] = @P1, [Question1] = @P2, [Question1Text] = @P3,   
                                                [Question2] = @P4, [Question2Text] = @P5,   
                                                [Question3] = @P6, [Question3Text] = @P7,   
                                                [Question4] = @P8, [Question4Text] = @P9, [Question5] = @P10,  
                                                [Question5Text] = @P11, [Question6] = @P12,   
                                                [Question6Text] = @P13, [InvestmentObjectives] = @P14, [Risk] = @P15,  
                                                [QuestionFATCA] = @P16, [FATCA1] = @P17, [FATCA2] = @P18, [FATCA3] = @P19, IsFinished = 1 
                                                WHERE AutoNID = @P20"#,
                                                &[
                                                    &5i32,
                                                    &request.question_1,
                                                    &request.question_1text,
                                                    &request.question_2,
                                                    &request.question_2text,
                                                    &request.question_3,
                                                    &request.question_3text,
                                                    &request.question_4,
                                                    &request.question_4text,
                                                    &request.question_5,
                                                    &request.question_5text,
                                                    &request.question_6,
                                                    &request.question_6text,
                                                    &request.investment_objective,
                                                    &request.risk,
                                                    &request.question_fatca,
                                                    &request.fatca_1,
                                                    &request.fatca_2,
                                                    &request.fatca_3,
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
                                        r#"UPDATE [dbo].[TableRequest]  
                                                SET [ContactPersonName] = @P1, [ContactPersonRelation] = @P2, [ContactPersonHomePhone] = @P3,   
                                                [ContactPersonMobilePhone] = @P4, [ContactPersonAddress] = @P5  
                                                WHERE WebCIFNID = @P6"#,
                                                &[
                                                    &request.contact_person_name,
                                                    &request.contact_person_relation,
                                                    &request.contact_person_home_phone,
                                                    &request.contact_person_mobile_phone,
                                                    &request.contact_person_address,
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

                                    // 🔵 Commit transaction
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
    
    pub async fn save_data_beneficiary(connection: web::Data<Pool<ConnectionManager>>, request: DataBeneficiaryRequest, session: Claims) -> ActionResult<HashMap<String, String>, String> {

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

                            if stage < current_stage {
                                result.message = "Stage has ben second or 1".to_owned();
                                return result;
                            }

                            if request.beneficiary_owner == 1 {
                                result.message = "Invalid beneficiary owner".to_owned();
                                return result;
                            }

                            match Transaction::begin(&connection).await {
                                Ok(trans) => {
                                    // 🔴 Scope ketiga: Insert ke TableRequest
                                    match trans.conn.lock().await.as_mut() {
                                        Some(conn) => {
                                            if let Err(err) = conn.execute(
                                        r#"UPDATE [dbo].[TableRequest]  
                                                SET [CIFInvestorBeneficiaryOwnerName] = @P1, [CIFInvestorBeneficiaryMothersMaidenName] = @P2, 
                                                [CIFInvestorBeneficiaryOwnerRelation] = @P3, [CIFInvestorBeneficiaryOwnerSex] = @P4, 
                                                [CIFInvestorBeneficiaryOwnerBirthPlace] = @P5, [CIFInvestorBeneficiaryOwnerBirthDate] = @P6, 
                                                [CIFInvestorBeneficiaryOwnerNationality] = @P7, [CIFInvestorBeneficiaryOwnerIDCardType] = @P8, 
                                                [CIFInvestorBeneficiaryOwnerIDCardNumber] = @P9, [CIFInvestorBeneficiaryOwnerIDCardExpiredDate] = @P10,  
                                                [CIFInvestorBeneficiaryOwnerEmail] = @P11, [CIFInvestorBeneficiaryOwnerNPWPNumber] = @P12,   
                                                [CIFInvestorBeneficiaryOwnerAddress1] = @P13, [CIFInvestorBeneficiaryOwnerAddress2] = @P14, 
                                                [CIFInvestorBeneficiaryOwnerAddress3] = @P15, [CIFInvestorBeneficiaryOwnerKelurahan] = @P16, 
                                                [CIFInvestorBeneficiaryOwnerKecamatan] = @P17, [CIFInvestorBeneficiaryOwnerRT] = @P18, 
                                                [CIFInvestorBeneficiaryOwnerRW] = @P19, [CIFInvestorBeneficiaryOwnerCity] = @P20,
                                                [CIFInvestorBeneficiaryOwnerProvince] = @P21, [CIFInvestorBeneficiaryOwnerCountry] = @P22,
                                                [CIFInvestorBeneficiaryOwnerPostalCode] = @P23, [CIFInvestorBeneficiaryOwnerMobilePhone] = @P24,
                                                [CIFInvestorBeneficiaryOwnerOccupation] = @P25, [CIFInvestorBeneficiaryOwnerOccupationText] = @P26,
                                                [CIFInvestorBeneficiaryOwnerCompanyName] = @P27, [CIFInvestorBeneficiaryOwnerPosition] = @P28,
                                                [CIFInvestorBeneficiaryOwnerNatureOfBusiness] = @P29, [CIFInvestorBeneficiaryOwnerIncomePerAnnum] = @P30,
                                                [CIFInvestorBeneficiaryOwnerCompanyAddress] = @P31, [CIFInvestorBeneficiaryOwnerCompanyAddress2] = @P32,
                                                [CIFInvestorBeneficiaryOwnerCompanyAddress3] = @P33, [CIFInvestorBeneficiaryOwnerCompanyCity] = @P34,
                                                [CIFInvestorBeneficiaryOwnerCompanyProvince] = @P35, [CIFInvestorBeneficiaryOwnerCompanyPostalCode] = @P36,
                                                [CIFInvestorBeneficiaryOwnerCompanyCountry] = @P37, [CIFInvestorBeneficiaryOwnerFundSource] = @P38,
                                                [CIFInvestorBeneficiaryOwnerFundSourceText] = @P39
                                                WHERE WebCIFNID = @P40"#,
                                                &[
                                                    &request.beneficiary_name,
                                                    &request.beneficiary_mother_maiden_name,
                                                    &request.beneficiary_relation,
                                                    &request.beneficiary_sex,
                                                    &request.beneficiary_birth_place,
                                                    &request.beneficiary_birth_date,
                                                    &request.beneficiary_nationality,
                                                    &request.beneficiary_idcard_type,
                                                    &request.beneficiary_idcard_number,
                                                    &request.beneficiary_idcard_expiredate,
                                                    &request.beneficiary_email,
                                                    &request.beneficiary_npwp_number,
                                                    &request.beneficiary_address1,
                                                    &request.beneficiary_address2,
                                                    &request.beneficiary_address3,
                                                    &request.beneficiary_kelurahan,
                                                    &request.beneficiary_kecamatan,
                                                    &request.beneficiary_rt,
                                                    &request.beneficiary_rw,
                                                    &request.beneficiary_city,
                                                    &request.beneficiary_province,
                                                    &request.beneficiary_country,
                                                    &request.beneficiary_postalcode,
                                                    &request.beneficiary_mobile_phone,
                                                    &request.beneficiary_occupation,
                                                    &request.beneficiary_occupation_text,
                                                    &request.beneficiary_company_name,
                                                    &request.beneficiary_position,
                                                    &request.beneficiary_nature_bussiness,
                                                    &request.beneficiary_income_peranum,
                                                    &request.beneficiary_company_address,
                                                    &request.beneficiary_company_address2,
                                                    &request.beneficiary_company_address3,
                                                    &request.beneficiary_company_city,
                                                    &request.beneficiary_company_province,
                                                    &request.beneficiary_company_postalcode,
                                                    &request.beneficiary_company_country,
                                                    &request.beneficiary_fund_source,
                                                    &request.beneficiary_fund_source_text,
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

                                    match Transaction::begin(&connection).await {
                                        Ok(trans) => {
                                            // 🔴 Scope ketiga: Insert ke TableRequest
                                            match trans.conn.lock().await.as_mut() {
                                                Some(conn) => {
                                                    if let Err(err) = conn.execute(
                                                    r#"UPDATE [dbo].[UserKYC]
                                                        SET [Stage] = @P1 WHERE AutoNID = @P2"#,
                                                        &[
                                                            &2i32,
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