mod user_store;

use axum::{Router, routing::{post, get}, response::{Json, Html}};
use tokio::net::TcpListener;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/share", post(share_sheet))
        .route("/sheets", get(view_shared_sheets));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ShareRequest {
    username: String,
    target_user: String,
    role: String, // "collaborator" or "viewer"
}

async fn signup(axum::Json(payload): axum::Json<Credentials>) -> Json<String> {
    match user_store::add_user(&payload.username, &payload.password) {
        Ok(_) => Json(format!("User '{}' registered successfully!", payload.username)),
        Err(msg) => Json(format!("Signup failed: {}", msg)),
    }
}

async fn login(axum::Json(payload): axum::Json<Credentials>) -> Json<String> {
    match user_store::validate_user(&payload.username, &payload.password) {
        Ok(_) => Json(format!("Login successful for '{}'", payload.username)),
        Err(msg) => Json(format!("Login failed: {}", msg)),
    }
}

async fn share_sheet(axum::Json(payload): axum::Json<ShareRequest>) -> Json<String> {
    let role = if payload.role == "collaborator" {
        user_store::ShareRole::Collaborator
    } else {
        user_store::ShareRole::Viewer
    };

    match user_store::share_sheet(&payload.username, &payload.target_user, role) {
        Ok(_) => Json(format!("Sheet shared with '{}'", payload.target_user)),
        Err(msg) => Json(format!("Failed to share sheet: {}", msg)),
    }
}

async fn view_shared_sheets(axum::Json(payload): axum::Json<Credentials>) -> Json<Vec<String>> {
    match user_store::get_user_sheets(&payload.username) {
        Ok(sheets) => Json(sheets),
        Err(_) => Json(vec!["Error: Unable to fetch sheets".to_string()]),
    }
}
