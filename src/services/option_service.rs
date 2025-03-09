use std::collections::HashMap;
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

    pub async fn get_district(connection: web::Data<Pool<ConnectionManager>>, city_id: i32) -> ActionResult<Vec<HashMap<String, String>>> {
        let mut result: ActionResult<Vec<HashMap<String, String>>> = ActionResult::default();

        let mut obj: HashMap<String, String> = HashMap::new();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(r#"SELECT KecamatanValue 
                    FROM [District] 
                    WHERE BO_NID = @P1 group by KecamatanValue order by KecamatanValue"#, &[&city_id]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut districts: Vec<HashMap<String, String>> = Vec::new(); // ⬅️ Tampung semua data
                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                obj.insert("code".to_string(), row.get::<&str, _>("KecamatanValue").map_or_else(|| "".to_string(), |s| s.to_string()));
                                districts.push(obj.clone());

                                result.result = true;
                                result.message = "District list retrieved successfully".to_string();
                                result.data = Some(districts.clone()); // ⬅️ Simpan array hasil query
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

    pub async fn get_sub_district(connection: web::Data<Pool<ConnectionManager>>, district: String) -> ActionResult<Vec<ListData>> {
        let mut result: ActionResult<Vec<ListData>> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(r#"SELECT KelurahanValue, ZIPCODE FROM [District] WHERE KecamatanValue = @P1"#, &[&district]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut sub_district: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data
                        let mut index = 1;
                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                let obj = ListData {
                                    data_id: index,
                                    code: row.get::<&str, _>("KelurahanValue").map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>("ZIPCODE").map_or_else(|| "".to_string(), |s| s.to_string()),
                                };
                                sub_district.push(obj.clone());
                                index += 1;
                                result.result = true;
                                result.message = "Sub District list retrieved successfully".to_string();
                                result.data = Some(sub_district.clone()); // ⬅️ Simpan array hasil query
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

    pub async fn get_sales(connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<Vec<ListData>> {
        let mut result = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query("SELECT DISTINCT SalesPersonNID, SalesPersonID, SalesPersonName FROM SalesPerson", &[]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut sales: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data

                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                sales.push(ListData {
                                    data_id: row.get::<i32, _>("SalesPersonNID").unwrap_or(0), // ⬅️ Ambil berdasarkan index kolom
                                    code: row.get::<&str, _>("SalesPersonID").map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>("SalesPersonName").map_or_else(|| "".to_string(), |s| s.to_string()),
                                });

                                result.result = true;
                                result.message = "Sales list retrieved successfully".to_string();
                                result.data = Some(sales.clone()); // ⬅️ Simpan array hasil query
                            } else {
                                result.message = "No Sales found".to_string();
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

    pub async fn get_lookup_data(connection: web::Data<Pool<ConnectionManager>>, option: String) -> ActionResult<Vec<ListData>> {
        let mut result: ActionResult<Vec<ListData>> = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(r#"SELECT DISTINCT CIFLookupInteger, CIFLookupString, CIFLookupDescription 
                    FROM CIFLookup WHERE CIFLookupID = @P1"#, &[&option]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut list_data: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data
                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                list_data.push(ListData {
                                    data_id: row.get::<i32, _>("CIFLookupInteger").unwrap_or(0), // ⬅️ Ambil berdasarkan index kolom
                                    code: row.get::<&str, _>("CIFLookupString").map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>("CIFLookupDescription").map_or_else(|| "".to_string(), |s| s.to_string()),
                                });

                                result.result = true;
                                result.message = format!("{} list retrieved successfully", option);
                                result.data = Some(list_data.clone()); // ⬅️ Simpan array hasil query
                            } else {
                                result.message = format!("No {} found", option);
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