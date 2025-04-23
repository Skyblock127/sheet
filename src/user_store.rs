use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq)]
pub enum ShareRole {
    Collaborator,
    Viewer,
}

pub struct UserStore {
    pub users: HashMap<String, String>, // username -> password
    pub shared: HashMap<String, HashMap<String, ShareRole>>,
    pub logged_in_users: HashSet<String>, // <--- new!
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            shared: HashMap::new(),
            logged_in_users: HashSet::new(), // <--- new!
        }
    }

    pub fn add_user(&mut self, username: &str, password: &str) -> Result<(), String> {
        if self.users.contains_key(username) {
            return Err("Username already exists".into());
        }

        self.users
            .insert(username.to_string(), password.to_string());
        Ok(())
    }

    pub fn validate_user(&mut self, username: &str, password: &str) -> Result<(), String> {
        match self.users.get(username) {
            Some(stored_pw) if stored_pw == password => {
                self.logged_in_users.insert(username.to_string());
                Ok(())
            }
            Some(_) => Err("Incorrect password".into()),
            None => Err("Username does not exist".into()),
        }
    }

    pub fn share_sheet(
        &mut self,
        owner: &str,
        target: &str,
        role: ShareRole,
    ) -> Result<(), String> {
        if !self.users.contains_key(target) {
            return Err("Target user does not exist".into());
        }

        self.shared
            .entry(owner.to_string())
            .or_default()
            .insert(target.to_string(), role);

        Ok(())
    }

    pub fn get_user_sheets(&self, username: &str) -> Result<Vec<String>, String> {
        let mut sheets = vec![];

        // Sheets owned by the user
        if self.users.contains_key(username) {
            sheets.push(format!("{} (owner)", username));
        }

        // Sheets shared with the user
        for (owner, access) in &self.shared {
            if let Some(role) = access.get(username) {
                let role_str = match role {
                    ShareRole::Collaborator => "collaborator",
                    ShareRole::Viewer => "viewer",
                };
                sheets.push(format!("{} ({})", owner, role_str));
            }
        }

        Ok(sheets)
    }
}
