mod user_store;

use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use user_store::{ShareRole, UserStore};

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

#[tokio::main]
async fn main() {
    let store = Arc::new(Mutex::new(UserStore::new()));

    let app = Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/share", post(share_sheet))
        .route("/sheets", get(view_shared_sheets))
        .with_state(store);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn signup(
    State(user_store): State<Arc<Mutex<UserStore>>>,
    Json(payload): Json<Credentials>,
) -> Json<String> {
    match user_store
        .lock()
        .unwrap()
        .add_user(&payload.username, &payload.password)
    {
        Ok(_) => Json(format!(
            "User '{}' registered successfully!",
            payload.username
        )),
        Err(msg) => Json(format!("Signup failed: {}", msg)),
    }
}

async fn login(
    State(user_store): State<Arc<Mutex<UserStore>>>,
    Json(payload): Json<Credentials>,
) -> Json<String> {
    match user_store
        .lock()
        .unwrap()
        .validate_user(&payload.username, &payload.password)
    {
        Ok(_) => Json(format!("Login successful for '{}'", payload.username)),
        Err(msg) => Json(format!("Login failed: {}", msg)),
    }
}

async fn share_sheet(
    State(user_store): State<Arc<Mutex<UserStore>>>,
    Json(payload): Json<ShareRequest>,
) -> Json<String> {
    let store = user_store.lock().unwrap();
    if !store.logged_in_users.contains(&payload.username) {
        return Json("You must be logged in to share a sheet".to_string());
    }

    if !store.users.contains_key(&payload.target_user) {
        return Json("Target user does not exist".to_string());
    }

    let role = match payload.role.as_str() {
        "collaborator" => ShareRole::Collaborator,
        "viewer" => ShareRole::Viewer,
        _ => return Json("Invalid role".to_string()),
    };

    drop(store); // release read lock before write

    match user_store
        .lock()
        .unwrap()
        .share_sheet(&payload.username, &payload.target_user, role)
    {
        Ok(_) => Json(format!("Sheet shared with '{}'", payload.target_user)),
        Err(msg) => Json(format!("Failed to share sheet: {}", msg)),
    }
}

async fn view_shared_sheets(
    State(user_store): State<Arc<Mutex<UserStore>>>,
    Json(payload): Json<Credentials>,
) -> Json<Vec<String>> {
    match user_store
        .lock()
        .unwrap()
        .get_user_sheets(&payload.username)
    {
        Ok(sheets) => Json(sheets),
        Err(_) => Json(vec!["Error: Unable to fetch sheets".to_string()]),
    }
}
