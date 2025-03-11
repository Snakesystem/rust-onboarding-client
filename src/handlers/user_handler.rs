use std::collections::HashMap;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;
use validator::Validate;

use crate::{contexts::model::{ActionResult, DataPribadiRequest}, services::validation_service::validator::format_validation_errors};

pub fn user_scope() -> Scope {
    
    web::scope("/user")
        .service(data_pribadi)
}

#[post("/data-pribadi")]
async fn data_pribadi(req: HttpRequest, pool: web::Data<Pool<ConnectionManager>>, request: web::Json<DataPribadiRequest>) -> impl Responder {

    if let Err(errors) = request.validate() {
        let formatted_errors = format_validation_errors(&errors);
        
        let result: ActionResult<HashMap<String, String>, _> = ActionResult {
            result: false,
            message: "Validation failed".to_string(),
            data: None,
            error: Some(formatted_errors),
        };

        return HttpResponse::BadRequest().json(result);
    }

    HttpResponse::Ok().finish()
}