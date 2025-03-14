use std::{collections::HashMap, fs};
use actix_web::{get, web, HttpResponse, Responder, Scope};
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: String,
    name: String,
}

pub fn file_scope() -> Scope {
    
    web::scope("/reports")
        .service(render_template)
}

#[get("/view-invoice")]
pub async fn render_template() -> impl Responder {
    // Bikin data dummy
    let users = vec![
        User { id: "001".to_string(), name: "Sandi Yoswandi".to_string() },
        User { id: "002".to_string(), name: "Yosua".to_string() },
        User { id: "003".to_string(), name: "Roberto Boentarya".to_string() },
    ];

    let mut data = HashMap::new();
    data.insert("users", users);

    let template_path = "./reports/template.mustache";

    // Baca file template
    let template_content = match fs::read_to_string(template_path) {
        Ok(content) => content,
        Err(_) => return HttpResponse::InternalServerError().body("Gagal membaca template"),
    };

    // Register template di handlebars
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("report", template_content).unwrap();

    let body = handlebars.render("report", &data).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}