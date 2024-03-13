use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Deserialize)]
struct ChatMessage {
    message: String,
}

#[derive(Serialize)]
struct OpenAIResponse {
    data: Vec<OpenAIImage>,
}

#[derive(Serialize)]
struct OpenAIImage {
    url: String,
}

async fn chat(msg: web::Json<ChatMessage>) -> impl Responder {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "dall-e-3",
            "prompt": msg.message,
            "n": 1,
            "size": "1024x1024"
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    HttpResponse::Ok().json(OpenAIResponse {
        data: res["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| OpenAIImage {
                url: v["url"].as_str().unwrap().to_string(),
            })
            .collect(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(web::resource("/chat").route(web::post().to(chat)))
            .service(fs::Files::new("/", "./static/").index_file("index.html")) // 这里添加了静态文件服务
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
