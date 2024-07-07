use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collector {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub id_number: String,
    pub phone_number: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub bank_name: String,
    pub bank_account_holder: String,
    pub bank_account_number: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
