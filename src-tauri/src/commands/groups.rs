//! 分组管理 Commands

use crate::models::{Group, GroupWithChildren};
use crate::AppState;
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

/// 获取所有分组
#[tauri::command]
pub async fn get_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    log::info!("get_groups called");
    state.db.get_groups().map_err(|e| e.to_string())
}

/// 获取分组树
#[tauri::command]
pub async fn get_group_tree(
    state: State<'_, AppState>,
    parent_id: Option<i64>,
) -> Result<Vec<GroupWithChildren>, String> {
    log::info!("get_group_tree called with parent_id: {:?}", parent_id);

    // 1. Get all groups
    let groups = state.db.get_groups().map_err(|e| e.to_string())?;

    // 2. Build tree
    let tree = build_group_tree(groups, parent_id);
    Ok(tree)
}

/// 构建分组树的辅助函数
fn build_group_tree(groups: Vec<Group>, root_parent_id: Option<i64>) -> Vec<GroupWithChildren> {
    let mut children_map: HashMap<Option<i64>, Vec<Group>> = HashMap::new();
    for g in groups {
        children_map
            .entry(g.parent_id)
            .or_insert_with(Vec::new)
            .push(g);
    }

    build_tree_recursive(&children_map, root_parent_id)
}

fn build_tree_recursive(
    map: &HashMap<Option<i64>, Vec<Group>>,
    current_parent: Option<i64>,
) -> Vec<GroupWithChildren> {
    let mut result = Vec::new();
    if let Some(siblings) = map.get(&current_parent) {
        for group in siblings {
            let children = build_tree_recursive(map, group.id);
            
            let node = GroupWithChildren {
                id: group.id.unwrap_or(0), 
                name: group.name.clone(),
                parent_id: group.parent_id,
                icon: group.icon.clone(),
                color: group.color.clone(),
                sort_order: group.sort_order,
                children,
            };
            result.push(node);
        }
    }
    result
}

/// 添加分组
#[tauri::command]
pub async fn add_group(state: State<'_, AppState>, group: Group) -> Result<Value, String> {
    log::info!("add_group called: {:?}", group.name);
    let id = state.db.add_group(&group).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "id": id
    }))
}

/// 更新分组
#[tauri::command]
pub async fn update_group(
    state: State<'_, AppState>,
    id: i64,
    mut group: Group,
) -> Result<Value, String> {
    log::info!("update_group called: id={}", id);
    group.id = Some(id); // Ensure ID matches
    state.db.update_group(&group).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true
    }))
}

/// 删除分组
#[tauri::command]
pub async fn delete_group(state: State<'_, AppState>, id: i64) -> Result<Value, String> {
    log::info!("delete_group called: id={}", id);
    state.db.delete_group(id).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true
    }))
}
