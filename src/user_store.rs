use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Clone)]
pub struct Sheet {
    pub owner: String,
    pub data: String, // This will be the spreadsheet data (for simplicity, just a string for now).
    pub collaborators: HashSet<String>,
    pub viewers: HashSet<String>,
}

pub static USERS: Lazy<Mutex<HashMap<String, Sheet>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn add_user(username: &str, password: &str) -> Result<(), &'static str> {
    let mut users = USERS.lock().unwrap();
    if users.contains_key(username) {
        return Err("Username already exists");
    }

    let hashed = hash(password, DEFAULT_COST).unwrap();
    let new_sheet = Sheet {
        owner: username.to_string(),
        data: String::from("Empty Sheet"), // For simplicity
        collaborators: HashSet::new(),
        viewers: HashSet::new(),
    };
    
    users.insert(username.to_string(), new_sheet);
    Ok(())
}

pub fn validate_user(username: &str, password: &str) -> Result<(), &'static str> {
    let users = USERS.lock().unwrap();
    match users.get(username) {
        Some(stored_sheet) => {
            if verify(password, &stored_sheet.owner).unwrap() {
                Ok(())
            } else {
                Err("Incorrect password")
            }
        }
        None => Err("Username does not exist"),
    }
}

pub fn get_sheet(username: &str) -> Option<Sheet> {
    let users = USERS.lock().unwrap();
    users.get(username).cloned()
}

pub fn share_sheet(owner: &str, target_user: &str, role: ShareRole) -> Result<(), &'static str> {
    let mut users = USERS.lock().unwrap();
    let sheet = users.get_mut(owner);
    match sheet {
        Some(sheet) => {
            if role == ShareRole::Collaborator {
                sheet.collaborators.insert(target_user.to_string());
            } else {
                sheet.viewers.insert(target_user.to_string());
            }
            Ok(())
        }
        None => Err("Owner not found"),
    }
}

pub fn get_user_sheets(username: &str) -> Result<Vec<String>, &'static str> {
    let users = USERS.lock().unwrap();
    let mut sheets = Vec::new();
    for (user, sheet) in users.iter() {
        if user == username {
            sheets.push(format!("Owned by: {}", user));
        } else if sheet.collaborators.contains(username) || sheet.viewers.contains(username) {
            sheets.push(format!("Shared with {}: {}", user, if sheet.collaborators.contains(username) { "Collaborator" } else { "Viewer" }));
        }
    }
    Ok(sheets)
}

#[derive(PartialEq)]
pub enum ShareRole {
    Collaborator,
    Viewer,
}
