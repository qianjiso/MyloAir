//! 分组数据模型

use serde::{Deserialize, Serialize};

/// 分组
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: Option<i64>,
    pub name: String,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 带子分组的分组树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupWithChildren {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub children: Vec<GroupWithChildren>,
}
