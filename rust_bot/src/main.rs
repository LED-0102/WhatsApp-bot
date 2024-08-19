mod config;
mod workflow;
mod user_state;
mod pdf;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use lazy_static::lazy_static;
use actix_web::{web, App, HttpResponse, HttpServer};
use dashmap::DashMap;
use user_state::UserState;

lazy_static! {
    static ref USER_STATES: DashMap<String, UserState> = DashMap::new();
}

pub async fn hello_world () -> HttpResponse {
    HttpResponse::Ok().body("Hello World!")
}

pub async fn process_message(form: web::Form<HashMap<String, String>>) -> HttpResponse {
    println!("Hit");
    let body = form.get("Body").unwrap().trim();
    let from = form.get("From").unwrap().trim();

    let mut user_state = USER_STATES.entry(from.to_string()).or_insert_with(|| UserState::default());

    user_state.process_message(body, from).await;

    HttpResponse::Ok().finish()
}

async fn serve_pdf(file_name: web::Path<String>) -> HttpResponse {
    // Define the base directory relative to your Rust server
    let base_dir = PathBuf::from("../pdf_microservice/generated_pdfs");

    // Construct the full path to the PDF file
    let file_path = base_dir.join(file_name.into_inner());

    // Open the file
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    // Read the file into a byte vector
    let mut file_contents = Vec::new();
    if let Err(_) = file.read_to_end(&mut file_contents) {
        return HttpResponse::InternalServerError().finish();
    }

    // Return the file as an HTTP response
    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(file_contents)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello_world))
            .route("/message", web::post().to(process_message))
            .route("/download/{filename}", web::get().to(serve_pdf))
    })
        .bind("127.0.0.1:8080")?
        .workers(4)
        .run()
        .await
}



