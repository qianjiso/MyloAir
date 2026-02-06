use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecureRecordGroup {
    pub id: Option<i64>,
    pub name: String,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecureRecord {
    pub id: Option<i64>,
    pub title: String,
    pub content: Option<String>,
    pub group_id: Option<i64>,
    pub pinned: Option<bool>,
    pub archived: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
