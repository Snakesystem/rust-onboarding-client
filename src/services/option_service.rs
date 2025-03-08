use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::QueryStream;
use tokio_stream::StreamExt;

use crate::contexts::model::{ActionResult, ListData};

pub struct OptionService;

impl OptionService {
    pub async fn get_nationality(connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<Vec<ListData>> {
        let mut result = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query("SELECT CountryNID, CIFISOCode, CIFNationalityName FROM [Country]", &[]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut countries: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data

                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                countries.push(ListData {
                                    data_id: row.get::<i32, _>("CountryNID").unwrap_or(0), // ⬅️ Ambil berdasarkan index kolom
                                    code: row.get::<&str, _>("CIFISOCode").map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>("CIFNationalityName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                });

                                result.result = true;
                                result.message = "Country list retrieved successfully".to_string();
                                result.data = Some(countries.clone()); // ⬅️ Simpan array hasil query
                            } else {
                                result.message = "No country found".to_string();
                                result.data = [].to_vec().into();
                            }
                        }
                        
                        return result;
                    }
                    Err(e) => {
                        result.message = "Internal Server Error".to_string();
                        result.error = Some(e.to_string());
                        return result;
                    }
                }
            }
            Err(e) => {
                result.error = Some(e.to_string());
                return result;
            }
        }
    }

    pub async fn get_city(connection: web::Data<Pool<ConnectionManager>>, city: String) -> ActionResult<Vec<ListData>> {
        let mut result: ActionResult<Vec<ListData>> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let city_param = format!("%{}%", city); // ✅ Tambahkan `%` untuk LIKE
                let query_result: Result<QueryStream, _> = conn.query(r#"SELECT ProvinceCityNID, SBRProvinceName, SBRCityName 
                    FROM [ProvinceCity]
                    WHERE SBRProvinceName IS NOT NULL AND SBRCityName IS NOT NULL AND SBRCityName LIKE @P1"#, &[&city_param]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut cities: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data
                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                cities.push(ListData {
                                    data_id: row.get::<i32, _>("ProvinceCityNID").unwrap_or(0), // ⬅️ Ambil berdasarkan index kolom
                                    code: row.get::<&str, _>("SBRProvinceName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>("SBRCityName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                });

                                result.result = true;
                                result.message = "Cities list retrieved successfully".to_string();
                                result.data = Some(cities.clone()); // ⬅️ Simpan array hasil query
                            } else {
                                result.message = "No Cities found".to_string();
                                result.data = [].to_vec().into();
                            }
                        }
                        
                        return result;
                    }
                    Err(e) => {
                        result.message = "Internal Server Error".to_string();
                        result.error = Some(e.to_string());
                        return result;
                    }
                }
            }
            Err(e) => {
                result.error = Some(e.to_string());
                return result;
            }
        }
    }

    pub async fn get_district(connection: web::Data<Pool<ConnectionManager>>, district: u32) -> ActionResult<Vec<ListData>> {
        let mut result: ActionResult<Vec<ListData>> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let district_param = format!("%{}%", district); // ✅ Tambahkan `%` untuk LIKE
                let query_result: Result<QueryStream, _> = conn.query(r#"SSELECT KecamatanValue 
                    FROM [District] 
                    WHERE BO_NID = @P1 group by KecamatanValue order by KecamatanValue"#, &[&district_param]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut cities: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data
                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                cities.push(ListData {
                                    data_id: row.get::<i32, _>("ProvinceCityNID").unwrap_or(0), // ⬅️ Ambil berdasarkan index kolom
                                    code: row.get::<&str, _>("SBRProvinceName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>("SBRCityName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                });

                                result.result = true;
                                result.message = "Cities list retrieved successfully".to_string();
                                result.data = Some(cities.clone()); // ⬅️ Simpan array hasil query
                            } else {
                                result.message = "No Cities found".to_string();
                                result.data = [].to_vec().into();
                            }
                        }
                        
                        return result;
                    }
                    Err(e) => {
                        result.message = "Internal Server Error".to_string();
                        result.error = Some(e.to_string());
                        return result;
                    }
                }
            }
            Err(e) => {
                result.error = Some(e.to_string());
                return result;
            }
        }
    }


}