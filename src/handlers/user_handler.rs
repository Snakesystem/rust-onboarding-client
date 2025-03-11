use actix_web::{post, web, HttpResponse, Responder, Scope};

pub fn user_scope() -> Scope {
    
    web::scope("/auth")
        .service(data_pribadi)
}

#[post("/data-pribadi")]
async fn data_pribadi() -> impl Responder {
    HttpResponse::Ok().finish()
}