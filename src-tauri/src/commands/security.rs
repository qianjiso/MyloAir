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
    // 从数据库获取主密码配置
    let (has_master, hint, require_password) = state.db.get_master_password_config()
        .map_err(|e| e.to_string())?;
    
    // TODO: 从数据库读取 auto_lock_minutes，目前使用默认值
    let auto_lock = 5;

    let payload = json!({
        "hasMasterPassword": has_master,
        "requireMasterPassword": require_password,
        "hint": hint,
        "autoLockMinutes": auto_lock,
        "lastUnlockAt": Option::<String>::None
    });
    
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
    
    // 自动解锁 UI（不再创建 session）
    {
        let mut ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
        *ui_locked = false;
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
            // 只更新 UI 锁定状态，不再创建 session
            {
                let mut ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
                *ui_locked = false;
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
    state: State<'_, AppState>,
    current_password: String,
) -> Result<Value, String> {
    // 1. 验证当前密码
    let db_hash_opt = state.db.get_master_password_hash().map_err(|e| e.to_string())?;
    
    if let Some(stored_hash) = db_hash_opt {
        let input_hash = hash_password(&current_password);
        if stored_hash != input_hash {
            return Ok(json!({
                "success": false,
                "error": "当前主密码错误"
            }));
        }
    } else {
        return Ok(json!({
            "success": false,
            "error": "尚未设置主密码"
        }));
    }
    
    // 2. 清除主密码
    state.db.clear_master_password().map_err(|e| e.to_string())?;
    
    // 3. 解锁 UI
    {
        let mut ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
        *ui_locked = false;
    }
    
    // 4. 返回新状态
    let new_state = security_get_state(state).await?;
    
    log::info!("Master password cleared successfully");
    
    Ok(json!({
        "success": true,
        "state": new_state
    }))
}

#[tauri::command]
pub async fn security_set_require_master_password(
    _state: State<'_, AppState>,
    _require: bool,
) -> Result<Value, String> {
    // TODO impl
    Ok(json!({ "success": false, "error": "尚未实现" }))
}


/// 锁定 UI
#[tauri::command]
pub async fn security_lock_ui(state: State<'_, AppState>) -> Result<Value, String> {
    let mut ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
    *ui_locked = true;
    Ok(json!({ "success": true }))
}

/// 获取 UI 锁定状态
#[tauri::command]
pub async fn security_get_ui_lock_state(state: State<'_, AppState>) -> Result<Value, String> {
    let ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
    Ok(json!({ "locked": *ui_locked }))
}
