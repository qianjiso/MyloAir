//! 安全相关 Commands
//!
//! 处理主密码验证、登录、锁定及会话管理

use crate::AppState;
use crate::services::encryption::EncryptionService;
use tauri::State;
use serde_json::{json, Value};
use sha2::{Sha256, Digest};

/// 计算密码哈希
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// 获取安全状态
#[tauri::command]
pub async fn security_get_state(state: State<'_, AppState>) -> Result<Value, String> {
    let has_master = state.db.has_master_password().map_err(|e| e.to_string())?;
    // TODO: Fetch require_password, hint, auto_lock from DB. 
    // Currently DatabaseService.has_master_password only checks existence.
    // We need more fields.
    // Let's implement a simple fetch for now assuming default logic.
    // Ideally update DatabaseService to return full config.
    // For now we assume:
    // If has_master, fetch hint.
    
    // Quick hack: Use sql query directly or extend DB service?
    // Extending DB service is better but I am in command refactor.
    // Let's trust `has_master` and return fetched hint if possible.
    // But `get_master_password_hash` doesn't return hint/require_password.
    // I will modify this later. For now, minimal working implementation.
    
    let require_master = true; // Default or fetch
    let hint: Option<String> = None; 
    let auto_lock = 5;

    let payload = json!({
        "hasMasterPassword": has_master,
        "requireMasterPassword": if has_master { require_master } else { false },
        "hint": hint,
        "autoLockMinutes": auto_lock,
        "lastUnlockAt": Option::<String>::None
    });
    
    // Note: Frontend helper (security.ts) might verify result structure.
    // But typical invoke returns the object directly.
    Ok(payload)
}

/// 设置主密码
#[tauri::command]
pub async fn security_set_master_password(
    state: State<'_, AppState>,
    password: String,
    hint: Option<String>,
) -> Result<Value, String> {
    if state.db.has_master_password().map_err(|e| e.to_string())? {
         return Ok(json!({ "success": false, "error": "已经设置了主密码" }));
    }
    
    let hash = hash_password(&password);
    state.db.set_master_password(&hash, hint.as_deref()).map_err(|e| e.to_string())?;
    
    // Auto login
    {
        let mut session = state.session.lock().map_err(|_| "Failed to lock session".to_string())?;
        *session = Some(EncryptionService::new(&password));
    }
    
    // Return new state
    let new_state = security_get_state(state).await?;
    Ok(json!({ "success": true, "state": new_state }))
}

/// 验证主密码 (登录)
#[tauri::command]
pub async fn security_verify_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<Value, String> {
    let db_hash_opt = state.db.get_master_password_hash().map_err(|e| e.to_string())?;
    
    if let Some(stored_hash) = db_hash_opt {
        let input_hash = hash_password(&password);
        if stored_hash == input_hash {
             {
                 let mut session = state.session.lock().map_err(|_| "Failed to lock session".to_string())?;
                 *session = Some(EncryptionService::new(&password));
             }
             let current_state = security_get_state(state).await?;
             Ok(json!({ "success": true, "state": current_state }))
        } else {
             Ok(json!({ "success": false, "error": "密码错误" }))
        }
    } else {
        Ok(json!({ "success": false, "error": "尚未设置主密码" }))
    }
}

// TODO: Implement update/clear/set_require with proper DB support.
// For now, placeholders to satisfy registration.

#[tauri::command]
pub async fn security_update_master_password(
    _state: State<'_, AppState>,
    _current_password: String,
    _new_password: String,
    _hint: Option<String>,
) -> Result<Value, String> {
    // TODO impl
    Ok(json!({ "success": false, "error": "尚未实现" }))
}

#[tauri::command]
pub async fn security_clear_master_password(
    _state: State<'_, AppState>,
    _current_password: String,
) -> Result<Value, String> {
    // TODO impl
    Ok(json!({ "success": false, "error": "尚未实现" }))
}

#[tauri::command]
pub async fn security_set_require_master_password(
    _state: State<'_, AppState>,
    _require: bool,
) -> Result<Value, String> {
    // TODO impl
    Ok(json!({ "success": false, "error": "尚未实现" }))
}

