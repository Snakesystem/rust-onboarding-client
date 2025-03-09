use std::collections::HashMap;
use actix_identity::Identity;
use actix_web::{get, web::{self}, HttpResponse, Responder, Scope};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use serde_json::json;

use crate::{contexts::{jwt_session::validate_jwt, model::{ActionResult, ListData}}, services::{generic_service::GenericService, option_service::OptionService}};

pub fn option_scope() -> Scope {
    web::scope("/option")
        .service(get_nationality)
        .service(get_city)
        .service(get_district)
        .service(get_sub_district)
        .service(get_sales)
        .service(get_income)
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

    let params: String = city.into_inner();

    // Cek apakah params kosong atau hanya spasi
    if params.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "result": false,
            "message": "Bad Request",
            "error": "City is empty"
        }));
    }

    let result: ActionResult<Vec<ListData>> = OptionService::get_city(pool, params.to_string()).await;

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

#[get("/district/{city_id}")]
pub async fn get_district(pool: web::Data<Pool<ConnectionManager>>, city_id: web::Path<String>) -> impl Responder {

    let params: i32 = match GenericService::parse_param(&city_id.into_inner()) {
        Ok(value) => value,
        Err(response) => return response,
    };

    // Cek apakah parameter kosong atau hanya spasi
    if params == 0 {
        return HttpResponse::BadRequest().json(json!({
            "result": false,
            "message": "Bad Request",
            "error": "City id is empty"
        }));
    }

    let result: ActionResult<Vec<HashMap<String, String>>> = OptionService::get_district(pool, params).await;

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

#[get("/subdistrict/{district}")]
pub async fn get_sub_district(pool: web::Data<Pool<ConnectionManager>>, district: web::Path<String>) -> impl Responder {

    let params: String = district.into_inner();

    // Cek apakah params kosong atau hanya spasi
    if params.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "result": false,
            "message": "Bad Request",
            "error": "District is empty"
        }));
    }

    let result: ActionResult<Vec<ListData>> = OptionService::get_sub_district(pool, params.to_string()).await;

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

#[get("/sales")]
pub async fn get_sales(pool: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let result: ActionResult<Vec<ListData>> = OptionService::get_sales(pool).await;

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

#[get("/income")]
pub async fn get_income(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "IncomePerAnnum".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/education")]
pub async fn get_education(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "Educational".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/maritalstatus")]
pub async fn get_maritalstatus(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "MaritalStatus".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/gender")]
pub async fn get_gender(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "Sex".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/religion")]
pub async fn get_religion(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "Religion".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/fundsource")]
pub async fn get_fundsource(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "FundSource".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/residencystatus")]
pub async fn get_residency_status(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "ResidencyStatus".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/beneficiary")]
pub async fn get_beneficiary(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "AssetOwner".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/investmentobjective")]
pub async fn get_investment_objective(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "InvestmentObjectives".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/risk")]
pub async fn get_risk(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "Risk".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/spouse-relationship")]
pub async fn get_spouse_relationship(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "SpouseRelationship".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/spouse-occupation")]
pub async fn get_spouse_occupation(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "Occupation".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/spouse-position")]
pub async fn get_spouse_position(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "Position".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/spouse-naturebusiness")]
pub async fn get_spouse_naturebusiness(pool: web::Data<Pool<ConnectionManager>>, identity: Option<Identity>) -> impl Responder {

    let mut result = ActionResult::default();

    match identity.map(|id| id.id()) {
        None => {
            result.error = Some("Token not found".to_string());
            return HttpResponse::Unauthorized().json(result);
        },
        Some(Ok(token)) => {
            match validate_jwt(&token) {
                Ok(_) => {

                    let data: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "NatureOfBusiness".to_string()).await;
                    
                    result.result = data.result;
                    result.message = data.message;
                    result.data = data.data;
                    result.error = data.error;

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

#[get("/bank-rdn")]
pub async fn get_bank_rdn(pool: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let result: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "QuestionRDN".to_string()).await;

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

#[get("/category")]
pub async fn get_category(pool: web::Data<Pool<ConnectionManager>>) -> impl Responder {

    let result: ActionResult<Vec<ListData>> = OptionService::get_lookup_data(pool, "ClientNCategory".to_string()).await;

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
