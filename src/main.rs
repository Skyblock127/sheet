mod user_store;

use axum::{Json, Router, routing::post};
use serde::Deserialize;
use tokio::net::TcpListener;
use user_store::{
    ShareRole, add_user, get_user_sheets, logout_user, remove_user_access, share_sheet,
    validate_user,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/share", post(share_handler))
        .route("/remove_access", post(remove_access_handler))
        .route("/sheets", post(list_sheets));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LogoutRequest {
    username: String,
}

#[derive(Deserialize)]
struct ShareRequest {
    username: String,
    target_user: String,
    role: String, // "collaborator" or "viewer", ignored for remove_access
}

#[derive(Deserialize)]
struct RemoveAccessRequest {
    username: String,    // owner
    target_user: String, // user to remove access for
}

async fn signup(Json(payload): Json<Credentials>) -> Json<String> {
    match add_user(&payload.username, &payload.password) {
        Ok(_) => Json(format!(
            "User '{}' registered and logged in!",
            payload.username
        )),
        Err(e) => Json(format!("Signup failed: {}", e)),
    }
}

async fn login(Json(payload): Json<Credentials>) -> Json<String> {
    match validate_user(&payload.username, &payload.password) {
        Ok(_) => Json(format!("Login successful for '{}'", payload.username)),
        Err(e) => Json(format!("Login failed: {}", e)),
    }
}

async fn logout(Json(payload): Json<LogoutRequest>) -> Json<String> {
    // Modified to only require username for logout
    match logout_user(&payload.username) {
        Ok(_) => Json(format!("User '{}' logged out", payload.username)),
        Err(e) => Json(format!("Logout failed: {}", e)),
    }
}

async fn share_handler(Json(payload): Json<ShareRequest>) -> Json<String> {
    // parse role
    let role = match payload.role.as_str() {
        "collaborator" => ShareRole::Collaborator,
        "viewer" => ShareRole::Viewer,
        _ => return Json("Invalid role, must be 'collaborator' or 'viewer'".into()),
    };

    match share_sheet(&payload.username, &payload.target_user, role) {
        Ok(_) => Json(format!(
            "User '{}' shared with '{}' as {}",
            payload.username, payload.target_user, payload.role
        )),
        Err(e) => Json(format!("Share failed: {}", e)),
    }
}

async fn remove_access_handler(Json(payload): Json<RemoveAccessRequest>) -> Json<String> {
    match remove_user_access(&payload.username, &payload.target_user) {
        Ok(_) => Json(format!(
            "Access for '{}' removed from '{}'",
            payload.target_user, payload.username
        )),
        Err(e) => Json(format!("Remove-access failed: {}", e)),
    }
}

async fn list_sheets(Json(payload): Json<LogoutRequest>) -> Json<Vec<String>> {
    // Modified to only require username for listing sheets
    match get_user_sheets(&payload.username) {
        Ok(list) => Json(list),
        Err(e) => Json(vec![format!("Error fetching sheets: {}", e)]),
    }
}
