
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Role {
    Admin,
    BaseUser,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => {
                f.write_str("Admin")
            }
            Role::BaseUser => {
                f.write_str("User")
            }
        }
    }
}