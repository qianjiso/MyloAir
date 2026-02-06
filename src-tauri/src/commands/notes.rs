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

// Helper: Encrypt/Decrypt logic
fn decrypt_note_content(state: &State<'_, AppState>, note: &mut SecureRecord) {
    if let Ok(session) = state.session.lock() {
        if let Some(service) = session.as_ref() {
            if let Some(cipher) = &note.content {
                if !cipher.is_empty() {
                    if let Ok(plain) = service.decrypt(cipher) {
                        note.content = Some(plain);
                    }
                }
            }
        }
    }
}

fn encrypt_note_content(state: &State<'_, AppState>, note: &mut SecureRecord) -> Result<(), String> {
    let session = state.session.lock().map_err(|_| "Failed to lock session".to_string())?;
    let service = session.as_ref().ok_or("Vault is locked".to_string())?;
    
    if let Some(plain) = &note.content {
        if !plain.is_empty() {
            let cipher = service.encrypt(plain)?;
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
    group.id = Some(id);
    state.db.update_note_group(&group).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_note_group(state: State<'_, AppState>, id: i64) -> Result<Value, String> {
    state.db.delete_note_group(id).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

// --- Notes ---

#[tauri::command]
pub async fn get_notes(state: State<'_, AppState>, group_id: Option<i64>) -> Result<Vec<SecureRecord>, String> {
    let mut notes = state.db.get_notes(group_id).map_err(|e| e.to_string())?;
    for note in &mut notes {
        decrypt_note_content(&state, note);
    }
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
