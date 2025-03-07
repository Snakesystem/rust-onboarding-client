use actix_web::{post, web, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use crate::{contexts::model::{LoginRequest, RegisterRequest}, services::auth_service::AuthService};

pub fn auth_scope() -> Scope {
    
    web::scope("/auth")
        .service(login)
        .service(register)
}

#[post("/login")]
async fn login(pool: web::Data<Pool<ConnectionManager>>, request: web::Json<LoginRequest>) -> impl Responder {

    let result = AuthService::login(pool, request.into_inner()).await;
    match result {
        r if r.error.is_some() => HttpResponse::InternalServerError().json(r), // Jika error, HTTP 500
        r if r.result => HttpResponse::Ok().json(r), // Jika berhasil, HTTP 200
        r => HttpResponse::BadRequest().json(r), // Jika gagal login, HTTP 400
    }
}

#[post("/register")]
async fn register(request: web::Json<RegisterRequest>) -> impl Responder {
    match AuthService::register(request.into_inner()).await {
        result if result.result => HttpResponse::Ok().json(result),
        result => HttpResponse::BadRequest().json(result),
    }
}

