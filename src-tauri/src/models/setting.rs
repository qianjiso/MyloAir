use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSetting {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub r#type: Option<String>, // type is reserved keyword
    pub category: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettingsCategory {
    pub category: String,
    pub description: String,
    pub settings: Vec<String>,
}
