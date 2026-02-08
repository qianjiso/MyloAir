//! 密码数据模型

use serde::{Deserialize, Serialize};

/// 密码条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Password {
    pub id: Option<i64>,
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    #[serde(rename = "group_id")]
    pub group_id: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub last_used_at: Option<String>,
    pub use_count: Option<i32>,
    pub favorite: Option<bool>,
    pub tags: Option<String>,
}

/// 密码历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordHistory {
    pub id: Option<i64>,
    pub password_id: i64,
    pub old_password: String,
    pub changed_at: String,
    pub change_reason: Option<String>,
}

/// 密码搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordSearchResult {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
}
