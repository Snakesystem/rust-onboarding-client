use actix_identity::Identity;
use actix_web::{cookie::{Cookie, SameSite}, get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use crate::{contexts::{jwt_session::{create_jwt, validate_jwt}, logger::write_log, model::{ActionResult, LoginRequest, RegisterRequest, WebUser}}, services::auth_service::AuthService};

pub fn auth_scope() -> Scope {
    
    web::scope("/auth")
        .service(login)
        .service(register)
        .service(check_session)
}

#[post("/login")]
async fn login(req: HttpRequest,pool: web::Data<Pool<ConnectionManager>>, request: web::Json<LoginRequest>) -> impl Responder {

    let result: ActionResult<WebUser> = AuthService::login(pool, request.into_inner()).await;

    match result {
        response if response.error.is_some() => {
            HttpResponse::InternalServerError().json(response)
        }, // Jika error, HTTP 500
        response if response.result => {
            if let Some(user) = &response.data {
                // ✅ Buat token JWT
                match create_jwt(user.clone()) {
                    Ok(token) => {
                        Identity::login(&req.extensions(), token.clone()).unwrap(); // ✅ Simpan sesi

                        // ✅ Simpan token dalam cookie
                        let cookie = Cookie::build("token", token.clone())
                            .path("/")
                            .http_only(true)
                            .same_site(SameSite::Strict)
                            .secure(false) // Ubah ke `true` jika pakai HTTPS
                            .finish();

                        return HttpResponse::Ok()
                            .cookie(cookie)
                            .json(response);
                    }
                    Err(err) => {
                        write_log("ERROR", format!("Failed to create JWT: {}", err).as_str());
                        return HttpResponse::InternalServerError().json(response);
                    }
                }
            }

            HttpResponse::BadRequest().json(response) // Jika tidak ada user, return 400
        },
        response => HttpResponse::BadRequest().json(response), // Jika gagal login, HTTP 400
    }
}

#[get("/session")]
async fn check_session(identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(claims) => {
                    result.result = true;
                    result.message = "Session active".to_string();
                    result.data = Some(claims);
                    return HttpResponse::Ok().json(result);
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
    };
}


#[post("/register")]
async fn register(request: web::Json<RegisterRequest>) -> impl Responder {
    match AuthService::register(request.into_inner()).await {
        result if result.result => HttpResponse::Ok().json(result),
        result => HttpResponse::BadRequest().json(result),
    }
}

