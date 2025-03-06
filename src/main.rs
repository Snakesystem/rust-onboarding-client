use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{ cookie::{time::Duration, Key}, get, middleware, web, App, HttpServer};
use contexts::logger::{init_log, write_log};

mod contexts {
    pub mod connection;
    pub mod  model;
    pub mod logger;
    pub mod jwt_session;
    pub mod crypto;
}

#[get("/")]
async fn health_check() -> String {
    format!("Custommer onboarding Web Api")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Aktifkan logging
    let secret_key: Key = Key::generate(); 

    let log_file = init_log().expect("Failed to initialize log file");
    write_log("INFO", "Test log message: Logging is working");
    dbg!("🚀 Application running on http://{}", log_file);
    
    HttpServer::new(move || {
        App::new()
            .service(web::scope("/v1")
        )
        // .app_data(web::Data::new(pool))
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