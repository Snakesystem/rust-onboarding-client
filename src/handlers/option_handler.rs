use actix_web::{get, web::{self}, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;

use crate::{contexts::model::{ActionResult, ListData}, services::option_service::OptionService};

pub fn option_scope() -> Scope {
    web::scope("/option")
        .service(get_nationality)
        .service(get_city)
}

#[get("/nationality")]
pub async fn get_nationality(pool: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let result: ActionResult<Vec<ListData>> = OptionService::get_nationality(pool).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, 
        response if response.result => {
            HttpResponse::Ok().json(response)
        }, 
        response => {
            HttpResponse::BadRequest().json(response)
        }
    }
}

#[get("/city/{city}")]
pub async fn get_city(pool: web::Data<Pool<ConnectionManager>>, city: web::Path<String>) -> impl Responder {

    let city_name: String = city.into_inner();

    // Cek apakah city_name kosong atau hanya spasi
    if city_name.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "result": false,
            "message": "City name is empty",
            "error": "Bad Request"
        }));
    }

    let result: ActionResult<Vec<ListData>> = OptionService::get_city(pool, city_name.to_string()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, 
        response if response.result => {
            HttpResponse::Ok().json(response)
        }, 
        response => {
            HttpResponse::BadRequest().json(response)
        }
    }
}