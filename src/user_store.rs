use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use bcrypt::{hash, verify, DEFAULT_COST};

pub static USERS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn add_user(username: &str, password: &str) -> Result<(), &'static str> {
    let mut users = USERS.lock().unwrap();
    if users.contains_key(username) {
        return Err("Username already exists");
    }

    let hashed = hash(password, DEFAULT_COST).unwrap();
    users.insert(username.to_string(), hashed);
    Ok(())
}

pub fn validate_user(username: &str, password: &str) -> Result<(), &'static str> {
    let users = USERS.lock().unwrap();
    match users.get(username) {
        Some(stored_hash) => {
            if verify(password, stored_hash).unwrap() {
                Ok(())
            } else {
                Err("Incorrect password")
            }
        }
        None => Err("Username does not exist"),
    }
}
