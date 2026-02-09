//! 笔记管理 Commands

use crate::models::{SecureRecord, SecureRecordGroup};
use crate::AppState;
use tauri::State;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct SecureRecordGroupWithChildren {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub children: Vec<SecureRecordGroupWithChildren>,
}

/// 辅助函数：解密笔记内容
fn decrypt_note_content(state: &State<'_, AppState>, note: &mut SecureRecord) {
    log::info!("[decrypt_note_content] 开始解密，note.id={:?}", note.id);
    if let Some(cipher) = &note.content {
        if !cipher.is_empty() {
            log::info!("[decrypt_note_content] 密文长度: {}", cipher.len());
            match state.encryption.decrypt(cipher) {
                Ok(plain) => {
                    log::info!("[decrypt_note_content] 解密成功，明文长度: {}", plain.len());
                    note.content = Some(plain);
                }
                Err(e) => {
                    log::error!("[decrypt_note_content] 解密失败: {}", e);
                    // 解密失败时保留原密文，让前端可以看到原始数据
                }
            }
        } else {
            log::info!("[decrypt_note_content] 密文为空");
        }
    } else {
        log::info!("[decrypt_note_content] content 为 None");
    }
}

/// 辅助函数：加密笔记内容
fn encrypt_note_content(state: &State<'_, AppState>, note: &mut SecureRecord) -> Result<(), String> {
    if let Some(plain) = &note.content {
        if !plain.is_empty() {
            let cipher = state.encryption.encrypt(plain)?;
            note.content = Some(cipher);
        }
    }
    Ok(())
}

// --- Note Groups ---

#[tauri::command]
pub async fn get_note_groups(state: State<'_, AppState>) -> Result<Vec<SecureRecordGroup>, String> {
    state.db.get_note_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_note_group_tree(
    state: State<'_, AppState>,
    parent_id: Option<i64>,
) -> Result<Vec<SecureRecordGroupWithChildren>, String> {
    let groups = state.db.get_note_groups().map_err(|e| e.to_string())?;
    Ok(build_note_group_tree(groups, parent_id))
}

fn build_note_group_tree(
    groups: Vec<SecureRecordGroup>,
    root_parent_id: Option<i64>,
) -> Vec<SecureRecordGroupWithChildren> {
    let mut children_map: HashMap<Option<i64>, Vec<SecureRecordGroup>> = HashMap::new();
    for g in groups {
        children_map
            .entry(g.parent_id)
            .or_insert_with(Vec::new)
            .push(g);
    }
    build_note_tree_recursive(&children_map, root_parent_id)
}

fn build_note_tree_recursive(
    map: &HashMap<Option<i64>, Vec<SecureRecordGroup>>,
    current_parent: Option<i64>,
) -> Vec<SecureRecordGroupWithChildren> {
    let mut result = Vec::new();
    if let Some(siblings) = map.get(&current_parent) {
        for group in siblings {
            let children = build_note_tree_recursive(map, group.id);
            let node = SecureRecordGroupWithChildren {
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

#[tauri::command]
pub async fn get_note_group(state: State<'_, AppState>, id: i64) -> Result<Option<SecureRecordGroup>, String> {
    state.db.get_note_group(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_note_group(state: State<'_, AppState>, group: SecureRecordGroup) -> Result<Value, String> {
    let id = state.db.add_note_group(&group).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "id": id }))
}

#[tauri::command]
pub async fn update_note_group(state: State<'_, AppState>, id: i64, mut group: SecureRecordGroup) -> Result<Value, String> {
    log::info!("[update_note_group] 开始更新分组, id={}, group={:?}", id, group);
    group.id = Some(id);
    match state.db.update_note_group(&group) {
        Ok(_) => {
            log::info!("[update_note_group] 更新成功, id={}", id);
            Ok(json!({ "success": true }))
        }
        Err(e) => {
            log::error!("[update_note_group] 更新失败, id={}, error={}", id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn delete_note_group(state: State<'_, AppState>, id: i64) -> Result<Value, String> {
    state.db.delete_note_group(id).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

// --- Notes ---

#[tauri::command]
pub async fn get_notes(state: State<'_, AppState>, group_id: Option<i64>) -> Result<Vec<SecureRecord>, String> {
    log::info!("[get_notes] 开始获取笔记列表, group_id={:?}", group_id);
    let mut notes = state.db.get_notes(group_id).map_err(|e| e.to_string())?;
    log::info!("[get_notes] 从数据库获取到 {} 条笔记", notes.len());
    for note in &mut notes {
        log::info!("[get_notes] 处理笔记 id={:?}, title={:?}", note.id, note.title);
        decrypt_note_content(&state, note);
    }
    log::info!("[get_notes] 完成，返回 {} 条笔记", notes.len());
    Ok(notes)
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: i64) -> Result<Option<SecureRecord>, String> {
    if let Some(mut note) = state.db.get_note(id).map_err(|e| e.to_string())? {
        decrypt_note_content(&state, &mut note);
        Ok(Some(note))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn add_note(state: State<'_, AppState>, mut note: SecureRecord) -> Result<Value, String> {
    encrypt_note_content(&state, &mut note)?;
    let id = state.db.add_note(&note).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true, "id": id }))
}

#[tauri::command]
pub async fn update_note(state: State<'_, AppState>, id: i64, mut note: SecureRecord) -> Result<Value, String> {
    note.id = Some(id);
    encrypt_note_content(&state, &mut note)?;
    state.db.update_note(&note).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, id: i64) -> Result<Value, String> {
    state.db.delete_note(id).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn search_notes_title(state: State<'_, AppState>, keyword: String) -> Result<Vec<SecureRecord>, String> {
    // Note: This searches database. 
    // If content is encrypted, searching content in DB will not yield correct results for plaintext keywords.
    // Title search works.
    let notes = state.db.search_notes(&keyword).map_err(|e| e.to_string())?;
    // We don't decrypt results for search list usually, or we do?
    // If UI shows snippet, we might need to decrypt.
    // Let's decrypt to be safe/consistent.
    /*
    let mut decrypted_notes = notes;
    for note in &mut decrypted_notes {
        decrypt_note_content(&state, note);
    }
    Ok(decrypted_notes)
    */
    // Wait, searching returns many results. Decrypting all might be slow if list is huge.
    // But for local app it's fine.
    
    // However, since we matched query against *Ciphertext* or *Title*, 
    // matched content notes might be weird (matching base64 string).
    // Better to filter by Title Only in DB if we want to avoid confusion?
    // But `search_notes` in DB does `OR content LIKE`.
    // Let's just return what DB gave, but decrypted.
    
    // Actually, Rust iterators consumption.
    let mut notes = notes;
    for note in &mut notes {
        decrypt_note_content(&state, note);
    }
    
    Ok(notes)
}
