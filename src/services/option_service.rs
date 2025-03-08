use actix_web::web;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tokio_stream::StreamExt;

use crate::contexts::model::{ActionResult, ListData};

pub struct OptionService;

impl OptionService {
    pub async fn get_nationality(connection: web::Data<Pool<ConnectionManager>>) -> ActionResult<Vec<ListData>> {
        let mut result = ActionResult::default();

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result = conn.query("SELECT CountryNID, CIFISOCode, CIFNationalityName FROM [Country]", &[]).await;
                match query_result {
                    Ok(mut rows) => {
                        let mut countries: Vec<ListData> = Vec::new(); // ⬅️ Tampung semua data

                        while let Some(query_item) = rows.try_next().await.unwrap_or(None) {
                            if let Some(row) = query_item.as_row() { // ⬅️ Konversi QueryItem menjadi Row
                                countries.push(ListData {
                                    data_id: row.get::<i32, _>(0).unwrap_or(0), // ⬅️ Ambil berdasarkan index kolom
                                    code: row.get::<&str, _>(1).map_or_else(|| "".to_string(), |s| s.to_string()),
                                    description: row.get::<&str, _>(2).map_or_else(|| "".to_string(), |s| s.to_string()),
                                });
                            }
                        }

                        if countries.is_empty() {
                            result.message = "No country found".to_string();
                        } else {
                            result.result = true;
                            result.message = "Country list retrieved successfully".to_string();
                            result.data = Some(countries); // ⬅️ Simpan array hasil query
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