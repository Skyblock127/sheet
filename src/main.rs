mod user_store;

use axum::{Router, routing::post, response::Json, extract::Json as ExtractJson};
use tokio::net::TcpListener;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

async fn signup(ExtractJson(payload): ExtractJson<Credentials>) -> Json<String> {
    match user_store::add_user(&payload.username, &payload.password) {
        Ok(_) => Json(format!("User '{}' registered successfully!", payload.username)),
        Err(msg) => Json(format!("Signup failed: {}", msg)),
    }
}

async fn login(ExtractJson(payload): ExtractJson<Credentials>) -> Json<String> {
    match user_store::validate_user(&payload.username, &payload.password) {
        Ok(_) => Json(format!("Login successful for '{}'", payload.username)),
        Err(msg) => Json(format!("Login failed: {}", msg)),
    }
}
