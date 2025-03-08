use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{ cookie::{time::Duration, Key}, error, get, middleware, web, App, HttpResponse, HttpServer};
use contexts::{connection::create_pool, logger::write_log, model::ActionResult};
use handlers::{auth_handler::auth_scope, generic_handler::generic_scope, option_handler::option_scope};
use log::info;

mod contexts {
    pub mod connection;
    pub mod  model;
    pub mod logger;
    pub mod jwt_session;
    pub mod crypto;
}

mod handlers {
    pub mod auth_handler;
    pub mod generic_handler;
    pub mod option_handler;
}

mod services {
    pub mod auth_service;
    pub mod generic_service;
    pub mod option_service;
}

#[get("/")]
async fn health_check() -> String {
    format!("Custommer onboarding Web Api")
}

fn json_error_handler(err: error::JsonPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
    let error_message = format!("Json deserialize error: {}", err);

    let result = ActionResult::<String> { // <- Ubah dari ActionResult<()> ke ActionResult<String>
        result: false,
        message: "Invalid Request".to_string(),
        error: Some(error_message), // <- Sekarang cocok karena `data: Option<String>`
        data: None,
    };

    error::InternalError::from_response(err, HttpResponse::BadRequest().json(result)).into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Aktifkan logging
    let secret_key: Key = Key::generate(); 
    dotenvy::dotenv().ok();
    let db_pool = create_pool("db12877").await.expect("Failed to create database pool");

    write_log("INFO", "Test log message: Logging is working");
    info!("🚀 Application running on http://127.0.0.1:8000");
    
    HttpServer::new(move || {
        App::new()
            .service(web::scope("/api/v1")
            .service(auth_scope())
            .service(generic_scope())
            .service(option_scope())
        )
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(web::JsonConfig::default().error_handler(json_error_handler))
        .service(health_check)
        .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("token".to_owned())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(7)))
                    .build(),
            )
            .wrap(middleware::NormalizePath::trim()) // 🔥 Normalisasi path (opsional)
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
    .map_err(|e| {
        eprintln!("Server error: {}", e);
        e
    })
}