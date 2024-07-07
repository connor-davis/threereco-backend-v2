use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub enum Role {
    Collector,
    Staff,
    Business,
    SystemAdmin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Collector => write!(f, "Collector"),
            Role::Business => write!(f, "Business"),
            Role::Staff => write!(f, "Staff"),
            Role::SystemAdmin => write!(f, "System Admin"),
        }
    }
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::Collector => "Collector".to_string(),
            Role::Business => "Business".to_string(),
            Role::Staff => "Staff".to_string(),
            Role::SystemAdmin => "System Admin".to_string(),
        }
    }
}
