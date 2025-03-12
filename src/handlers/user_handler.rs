use std::collections::HashMap;

use actix_identity::Identity;
use actix_web::{post, web, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use validator::Validate;

use crate::{contexts::{jwt_session::validate_jwt, model::{ActionResult, DataPribadiRequest}}, services::{user_service::UserService, validation_service::validator::format_validation_errors}};

pub fn user_scope() -> Scope {
    
    web::scope("/user")
        .service(data_pribadi)
}

#[post("/data-pribadi")]
async fn data_pribadi(pool: web::Data<Pool<ConnectionManager>>, request: web::Json<DataPribadiRequest>, session: Option<Identity>) -> impl Responder {

    if let Err(errors) = request.validate() {
        let formatted_errors: HashMap<String, String> = format_validation_errors(&errors);
        
        let result: ActionResult<HashMap<String, String>, _> = ActionResult {
            result: false,
            message: "Validation failed".to_string(),
            data: None,
            error: Some(formatted_errors),
        };

        return HttpResponse::BadRequest().json(result);
    }

    let mut result: ActionResult<HashMap<String, String>, _> = ActionResult::default();

    match session.map(|id: Identity| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(claims) => {
                    let response = UserService::save_data_pribadi(pool, request.into_inner(), claims).await;

                    match response {
                        result if result.error.is_some() => {
                            HttpResponse::InternalServerError().json(result)
                        }, 
                        result if result.result => HttpResponse::Ok().json(result), // Jika berhasil, HTTP 200
                        result => HttpResponse::BadRequest().json(result), // Jika gagal, HTTP 400
                    }
                },
                Err(err) => {
                    result.error = Some(err.to_string());
                    return HttpResponse::Unauthorized().json(result);
                },
            }
        },
        Some(Err(_)) => {
            result.error = Some("Invalid token".to_string());
            return HttpResponse::BadRequest().json(result);
        },
    }
}