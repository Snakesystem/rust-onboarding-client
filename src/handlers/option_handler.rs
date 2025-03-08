use actix_web::{get, web, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use crate::{contexts::model::{ActionResult, ListData}, services::option_service::OptionService};

pub fn option_scope() -> Scope {
    web::scope("/option")
        .service(get_nationality)
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