use bcrypt::{DEFAULT_COST, hash, verify};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub enum ShareRole {
    Collaborator,
    Viewer,
}

pub struct UserStore {
    pub users: HashMap<String, String>,   // username → hashed password
    pub logged_in_users: HashSet<String>, // who's currently logged in
    pub shared_sheets: HashMap<String, Vec<Option<String>>>,
    // owner → list where
    //   [0]         = collaborator (None if none)
    //   [1..]       = viewers (each Some(username))
}

pub static STORE: Lazy<Mutex<UserStore>> = Lazy::new(|| {
    Mutex::new(UserStore {
        users: HashMap::new(),
        logged_in_users: HashSet::new(),
        shared_sheets: HashMap::new(),
    })
});

/// Add a new user to the system
pub fn add_user(username: &str, password: &str) -> Result<(), String> {
    let mut store = STORE.lock().unwrap();

    // Check if user already exists
    if store.users.contains_key(username) {
        return Err("User already exists".into());
    }

    // Hash the password
    let hashed_password =
        hash(password, DEFAULT_COST).map_err(|_| "Password hashing failed".to_string())?;

    // Add user to the system
    store.users.insert(username.to_string(), hashed_password);

    // Auto-login after signup
    store.logged_in_users.insert(username.to_string());

    Ok(())
}

/// Validate user credentials and log them in
pub fn validate_user(username: &str, password: &str) -> Result<(), String> {
    let mut store = STORE.lock().unwrap();

    // Check if user exists
    let hashed_password = store
        .users
        .get(username)
        .ok_or_else(|| "Username does not exist".to_string())?;

    // Check if already logged in
    if store.logged_in_users.contains(username) {
        return Err("User already logged in".into());
    }

    // Verify password
    let is_valid = verify(password, hashed_password)
        .map_err(|_| "Password verification failed".to_string())?;

    if is_valid {
        store.logged_in_users.insert(username.to_string());
        Ok(())
    } else {
        Err("Invalid password".into())
    }
}

/// Log out a user
pub fn logout_user(username: &str) -> Result<(), String> {
    let mut store = STORE.lock().unwrap();

    // Check if user exists
    if !store.users.contains_key(username) {
        return Err("User does not exist".into());
    }

    // Check if user is logged in
    if !store.logged_in_users.contains(username) {
        return Err("User is not logged in".into());
    }

    // Log user out
    store.logged_in_users.remove(username);

    Ok(())
}

/// Get list of sheets shared with the user
pub fn get_user_sheets(username: &str) -> Result<Vec<String>, String> {
    let store = STORE.lock().unwrap();

    // Check if user exists
    if !store.users.contains_key(username) {
        return Err("User does not exist".into());
    }

    // Get sheets where user is owner
    let mut sheets = Vec::new();
    sheets.push(format!("{}'s own sheet", username));

    // Get sheets shared with user (as collaborator or viewer)
    for (owner, shares) in &store.shared_sheets {
        // Check if user is collaborator (index 0)
        if shares.first() == Some(&Some(username.to_string())) {
            sheets.push(format!("{}'s sheet (collaborator)", owner));
        }

        // Check if user is viewer (index 1+)
        if shares
            .iter()
            .skip(1)
            .any(|u| u.as_ref() == Some(&username.to_string()))
        {
            sheets.push(format!("{}'s sheet (viewer)", owner));
        }
    }

    Ok(sheets)
}

/// Share a sheet with another user
pub fn share_sheet(owner: &str, target: &str, role: ShareRole) -> Result<(), String> {
    let mut store = STORE.lock().unwrap();

    // 🔒 validations
    if !store.users.contains_key(owner) {
        return Err("Sender does not exist".into());
    }
    if !store.users.contains_key(target) {
        return Err("Receiver does not exist".into());
    }
    if !store.logged_in_users.contains(owner) {
        return Err("Sender is not logged in".into());
    }

    // get or initialize the share-list
    let list = store
        .shared_sheets
        .entry(owner.to_string())
        .or_insert_with(|| vec![None]); // start with just [None]

    match role {
        ShareRole::Collaborator => {
            // 1) if there's an existing collaborator, demote them to viewer
            if let Some(old_collab) = list[0].take() {
                // new viewer at the end
                if !list[1..].iter().any(|u| u.as_ref() == Some(&old_collab)) {
                    list.push(Some(old_collab));
                }
            }
            // 2) remove target if already a viewer
            list.retain(|u| u.as_ref() != Some(&target.to_string()) || u.is_none());
            // 3) set new collaborator
            list[0] = Some(target.to_string());
        }
        ShareRole::Viewer => {
            // if target is current collaborator, demote them
            if list[0].as_ref() == Some(&target.to_string()) {
                list[0] = None;
            }
            // if not already in viewers, add them
            if !list[1..]
                .iter()
                .any(|u| u.as_ref() == Some(&target.to_string()))
            {
                list.push(Some(target.to_string()));
            }
        }
    }

    Ok(())
}

/// Remove user access to a sheet
pub fn remove_user_access(owner: &str, target: &str) -> Result<(), String> {
    let mut store = STORE.lock().unwrap();

    if !store.users.contains_key(owner) {
        return Err("Owner does not exist".into());
    }
    if !store.shared_sheets.contains_key(owner) {
        return Err("Owner has no shared sheet entries".into());
    }
    let list = store.shared_sheets.get_mut(owner).unwrap();

    // if they're the collaborator
    if list[0].as_ref() == Some(&target.to_string()) {
        list[0] = None;
        return Ok(());
    }
    // otherwise try to remove from the viewer slots
    let before = list.len();
    list.retain(|u| u.as_ref() != Some(&target.to_string()));
    if list.len() < before {
        return Ok(());
    }

    Err("Target user did not have access".into())
}
