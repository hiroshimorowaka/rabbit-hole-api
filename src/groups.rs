use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum UserGroups {
    Admin,
    User,
}

impl UserGroups {
    pub fn to_string(&self) -> String {
        match self {
            UserGroups::Admin => "admin".to_string(),
            UserGroups::User => "user".to_string(),
        }
    }

    pub fn from_str(group: &str) -> Option<UserGroups> {
        match group {
            "admin" => Some(UserGroups::Admin),
            "user" => Some(UserGroups::User),
            _ => None,
        }
    }

    pub fn is_valid(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
}
