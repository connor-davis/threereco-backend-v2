use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Business {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_name: String,
    pub business_type: String,
    pub business_description: String,
    pub phone_number: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}