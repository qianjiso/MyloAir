//! 密码管理 Commands

use crate::models::{Password, PasswordSearchResult};
use serde_json::Value;
use tauri::State;
use crate::AppState;

/// 辅助函数：解密密码字段
fn decrypt_password_field(state: &State<'_, AppState>, p: &mut Password) {
    if let Ok(session) = state.session.lock() {
        if let Some(service) = session.as_ref() {
             if let Some(cipher) = &p.password {
                 if !cipher.is_empty() {
                     if let Ok(plain) = service.decrypt(cipher) {
                         p.password = Some(plain);
                     }
                 }
             }
        }
    }
}

/// 辅助函数：加密密码字段
fn encrypt_password_field(state: &State<'_, AppState>, p: &mut Password) -> Result<(), String> {
    let session = state.session.lock().map_err(|_| "Failed to lock session".to_string())?;
    let service = session.as_ref().ok_or("Vault is locked".to_string())?;
    
    if let Some(plain) = &p.password {
        if !plain.is_empty() {
            let cipher = service.encrypt(plain)?;
            p.password = Some(cipher);
        }
    }
    Ok(())
}

/// 获取密码列表
#[tauri::command]
pub async fn get_passwords(
    state: State<'_, AppState>,
    group_id: Option<i64>,
) -> Result<Vec<Password>, String> {
    log::info!("get_passwords called with group_id: {:?}", group_id);
    let mut passwords = state.db.get_passwords(group_id).map_err(|e| e.to_string())?;
    
    // Decrypt passwords
    for p in &mut passwords {
        decrypt_password_field(&state, p);
    }
    
    Ok(passwords)
}

/// 获取单个密码
#[tauri::command]
pub async fn get_password(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Password>, String> {
    log::info!("get_password called with id: {}", id);
    if let Some(mut p) = state.db.get_password(id).map_err(|e| e.to_string())? {
        decrypt_password_field(&state, &mut p);
        Ok(Some(p))
    } else {
        Ok(None)
    }
}

/// 添加密码
#[tauri::command]
pub async fn add_password(
    state: State<'_, AppState>,
    mut password: Password,
) -> Result<Value, String> {
    log::info!("add_password called: {:?}", password.title);
    
    encrypt_password_field(&state, &mut password)?;
    
    let id = state.db.add_password(&password).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true,
        "id": id
    }))
}

/// 更新密码
#[tauri::command]
pub async fn update_password(
    state: State<'_, AppState>,
    id: i64,
    mut password: Password,
) -> Result<Value, String> {
    log::info!("update_password called: id={}", id);
    
    // 确保 ID 一致
    password.id = Some(id);
    
    // 加密
    encrypt_password_field(&state, &mut password)?;
    
    state.db.update_password(&password).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// 删除密码
#[tauri::command]
pub async fn delete_password(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Value, String> {
    log::info!("delete_password called: id={}", id);
    let _ = state.session.lock().map_err(|_| "Failed to lock session".to_string())?
        .as_ref().ok_or("Vault is locked".to_string())?;

    state.db.delete_password(id).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// 搜索密码
#[tauri::command]
pub async fn search_passwords(
    state: State<'_, AppState>,
    keyword: String,
) -> Result<Vec<PasswordSearchResult>, String> {
    log::info!("search_passwords called: keyword={}", keyword);
    
    // Search works on plaintext fields (title, username, notes)
    // Results usually don't contain password field or it's not shown in search list
    let passwords = state.db.search_passwords(&keyword).map_err(|e| e.to_string())?;
    
    // 转换为 PasswordSearchResult
    let results = passwords.into_iter().map(|p| PasswordSearchResult {
        id: p.id.unwrap_or(0),
        title: p.title,
        username: p.username,
        url: p.url,
        group_id: p.group_id,
        group_name: None, 
    }).collect();
    
    Ok(results)
}


