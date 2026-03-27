//! 安全相关 Commands
//!
//! 处理主密码验证、登录、锁定及会话管理

use crate::models::UserSetting;
use crate::{AppState, UnlockThrottleState};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use tauri::State;

const UNLOCK_FAILURE_LIMIT: u32 = 5;
const UNLOCK_COOLDOWN_SECONDS: u64 = 30;
const DEFAULT_AUTO_LOCK_MINUTES: u64 = 5;
const MIN_MASTER_PASSWORD_LEN: usize = 6;

/// 计算密码哈希
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn validate_master_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_MASTER_PASSWORD_LEN {
        return Err(format!("主密码长度至少为{}位", MIN_MASTER_PASSWORD_LEN));
    }
    Ok(())
}

fn read_auto_lock_minutes(state: &State<'_, AppState>) -> Result<u64, String> {
    let timeout_setting = state
        .db
        .get_user_setting("security.auto_lock_timeout")
        .map_err(|e| e.to_string())?
        .or_else(|| state.db.get_user_setting("autoLockTime").ok().flatten());

    let maybe_seconds = timeout_setting.and_then(|setting| setting.value.parse::<u64>().ok());
    let minutes = maybe_seconds
        .map(|seconds| ((seconds + 59) / 60).max(1))
        .unwrap_or(DEFAULT_AUTO_LOCK_MINUTES);
    Ok(minutes)
}

fn read_last_unlock_at(state: &State<'_, AppState>) -> Result<Option<String>, String> {
    let setting = state
        .db
        .get_user_setting("security.last_unlock_at")
        .map_err(|e| e.to_string())?;
    Ok(setting.and_then(|s| {
        let value = s.value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }))
}

fn touch_last_unlock_at(state: &State<'_, AppState>) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let setting = UserSetting {
        id: None,
        key: "security.last_unlock_at".to_string(),
        value: now,
        r#type: Some("string".to_string()),
        category: Some("security".to_string()),
        description: Some("最近一次解锁时间".to_string()),
        created_at: None,
        updated_at: None,
    };
    state.db.set_user_setting(&setting).map_err(|e| e.to_string())
}

fn active_cooldown_seconds(throttle: &mut UnlockThrottleState, now: Instant) -> Option<u64> {
    let until = throttle.cooldown_until?;
    if now >= until {
        throttle.failed_attempts = 0;
        throttle.cooldown_until = None;
        return None;
    }
    Some(until.duration_since(now).as_secs().max(1))
}

fn register_unlock_failure(throttle: &mut UnlockThrottleState, now: Instant) -> Option<u64> {
    throttle.failed_attempts += 1;
    if throttle.failed_attempts < UNLOCK_FAILURE_LIMIT {
        return None;
    }

    let cooldown = Duration::from_secs(UNLOCK_COOLDOWN_SECONDS);
    throttle.cooldown_until = Some(now + cooldown);
    Some(cooldown.as_secs())
}

fn reset_unlock_throttle(throttle: &mut UnlockThrottleState) {
    throttle.failed_attempts = 0;
    throttle.cooldown_until = None;
}

fn verify_current_password(stored_hash_opt: Option<String>, input_password: &str) -> Result<(), &'static str> {
    if let Some(stored_hash) = stored_hash_opt {
        let input_hash = hash_password(input_password);
        if stored_hash == input_hash {
            Ok(())
        } else {
            Err("wrong_password")
        }
    } else {
        Err("not_set")
    }
}

fn lock_state_after_require_toggle(require: bool) -> bool {
    require
}

/// 获取安全状态
#[tauri::command]
pub async fn security_get_state(state: State<'_, AppState>) -> Result<Value, String> {
    // 从数据库获取主密码配置
    let (has_master, hint, require_password) = state.db.get_master_password_config()
        .map_err(|e| e.to_string())?;

    let auto_lock = read_auto_lock_minutes(&state)?;
    let last_unlock_at = read_last_unlock_at(&state)?;

    let payload = json!({
        "hasMasterPassword": has_master,
        "requireMasterPassword": require_password,
        "hint": hint,
        "autoLockMinutes": auto_lock,
        "lastUnlockAt": last_unlock_at
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

    validate_master_password(&password)?;
    let hash = hash_password(&password);
    state.db.set_master_password(&hash, hint.as_deref()).map_err(|e| e.to_string())?;

    // 自动解锁 UI（不再创建 session）
    {
        let mut ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
        *ui_locked = false;
    }
    touch_last_unlock_at(&state)?;

    {
        let mut throttle = state
            .unlock_throttle
            .lock()
            .map_err(|_| "Failed to lock throttle state".to_string())?;
        reset_unlock_throttle(&mut throttle);
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
    let now = Instant::now();
    {
        let mut throttle = state
            .unlock_throttle
            .lock()
            .map_err(|_| "Failed to lock throttle state".to_string())?;
        if let Some(remaining) = active_cooldown_seconds(&mut throttle, now) {
            return Ok(json!({
                "success": false,
                "error": format!("尝试次数过多，请 {} 秒后重试", remaining),
                "errorCode": "UNLOCK_COOLDOWN",
                "cooldownSeconds": remaining
            }));
        }
    }

    let db_hash_opt = state.db.get_master_password_hash().map_err(|e| e.to_string())?;

    if let Some(stored_hash) = db_hash_opt {
        let input_hash = hash_password(&password);
        if stored_hash == input_hash {
            // 只更新 UI 锁定状态，不再创建 session
            {
                let mut ui_locked = state.ui_locked.lock().map_err(|_| "Failed to lock state".to_string())?;
                *ui_locked = false;
            }
            touch_last_unlock_at(&state)?;
            {
                let mut throttle = state
                    .unlock_throttle
                    .lock()
                    .map_err(|_| "Failed to lock throttle state".to_string())?;
                reset_unlock_throttle(&mut throttle);
            }
            let current_state = security_get_state(state).await?;
            Ok(json!({ "success": true, "state": current_state }))
        } else {
            let mut throttle = state
                .unlock_throttle
                .lock()
                .map_err(|_| "Failed to lock throttle state".to_string())?;
            let cooldown_seconds = register_unlock_failure(&mut throttle, now);
            if let Some(remaining) = cooldown_seconds {
                Ok(json!({
                    "success": false,
                    "error": format!("密码错误，已冷却 {} 秒", remaining),
                    "errorCode": "UNLOCK_COOLDOWN",
                    "cooldownSeconds": remaining
                }))
            } else {
                Ok(json!({ "success": false, "error": "密码错误" }))
            }
        }
    } else {
        Ok(json!({ "success": false, "error": "尚未设置主密码" }))
    }
}

#[tauri::command]
pub async fn security_update_master_password(
    state: State<'_, AppState>,
    current_password: String,
    new_password: String,
    hint: Option<String>,
) -> Result<Value, String> {
    // 1. 验证当前密码
    let db_hash_opt = state.db.get_master_password_hash().map_err(|e| e.to_string())?;
    match verify_current_password(db_hash_opt, &current_password) {
        Ok(()) => {}
        Err("wrong_password") => {
            return Ok(json!({
                "success": false,
                "error": "当前密码错误"
            }))
        }
        Err(_) => {
            return Ok(json!({
                "success": false,
                "error": "主密码未设置"
            }))
        }
    }

    validate_master_password(&new_password)?;

    let (_has_master, _old_hint, require_password) = state
        .db
        .get_master_password_config()
        .map_err(|e| e.to_string())?;

    // 2. 更新密码
    let new_hash = hash_password(&new_password);
    state
        .db
        .set_master_password_with_require(&new_hash, hint.as_deref(), require_password)
        .map_err(|e| e.to_string())?;

    log::info!("Master password updated successfully");

    // 3. 返回新状态(不修改 ui_locked)
    let new_state = security_get_state(state).await?;

    Ok(json!({
        "success": true,
        "state": new_state
    }))
}

#[tauri::command]
pub async fn security_clear_master_password(
    state: State<'_, AppState>,
    current_password: String,
) -> Result<Value, String> {
    // 1. 验证当前密码
    let db_hash_opt = state.db.get_master_password_hash().map_err(|e| e.to_string())?;

    match verify_current_password(db_hash_opt, &current_password) {
        Ok(()) => {}
        Err("wrong_password") => {
            return Ok(json!({
                "success": false,
                "error": "当前主密码错误"
            }))
        }
        Err(_) => {
            return Ok(json!({
                "success": false,
                "error": "尚未设置主密码"
            }))
        }
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
    state: State<'_, AppState>,
    require: bool,
    password: Option<String>,
    hint: Option<String>,
    current_password: Option<String>,
) -> Result<Value, String> {
    if require {
        // 已设置主密码时，仅切换“是否要求解锁”
        if state.db.has_master_password().map_err(|e| e.to_string())? {
            state
                .db
                .set_require_master_password(true)
                .map_err(|e| e.to_string())?;
        } else {
            let pwd = password.ok_or("开启主密码需要提供密码".to_string())?;
            validate_master_password(&pwd)?;
            let hash = hash_password(&pwd);
            state
                .db
                .set_master_password_with_require(&hash, hint.as_deref(), true)
                .map_err(|e| e.to_string())?;
        }

        // 立即锁定 UI
        {
            let mut ui_locked = state.ui_locked.lock()
                .map_err(|_| "Failed to lock state".to_string())?;
            *ui_locked = lock_state_after_require_toggle(true);
        }

        log::info!("Master password enabled and UI locked");

    } else {
        // 关闭“要求解锁”，保留主密码
        let current_pwd = current_password.ok_or("关闭主密码需要验证当前密码".to_string())?;

        // 验证密码
        let db_hash_opt = state.db.get_master_password_hash()
            .map_err(|e| e.to_string())?;

        match verify_current_password(db_hash_opt, &current_pwd) {
            Ok(()) => {}
            Err("wrong_password") => {
                return Ok(json!({
                    "success": false,
                    "error": "密码错误"
                }))
            }
            Err(_) => {
                return Ok(json!({
                    "success": false,
                    "error": "主密码未设置"
                }))
            }
        }
        
        // 仅关闭 require_password，不清除主密码哈希
        state
            .db
            .set_require_master_password(false)
            .map_err(|e| e.to_string())?;

        // 立即解锁 UI
        {
            let mut ui_locked = state.ui_locked.lock()
                .map_err(|_| "Failed to lock state".to_string())?;
            *ui_locked = lock_state_after_require_toggle(false);
        }

        log::info!("Master password disabled and UI unlocked");
    }

    // 返回新状态
    let new_state = security_get_state(state).await?;

    Ok(json!({
        "success": true,
        "state": new_state
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throttle_activates_after_five_failures() {
        let mut throttle = UnlockThrottleState::default();
        let now = Instant::now();

        for _ in 0..(UNLOCK_FAILURE_LIMIT - 1) {
            assert!(register_unlock_failure(&mut throttle, now).is_none());
        }

        let cooldown = register_unlock_failure(&mut throttle, now);
        assert_eq!(cooldown, Some(UNLOCK_COOLDOWN_SECONDS));
        assert!(active_cooldown_seconds(&mut throttle, now).is_some());
    }

    #[test]
    fn test_throttle_resets_after_expiry_or_success() {
        let mut throttle = UnlockThrottleState::default();
        let now = Instant::now();

        for _ in 0..UNLOCK_FAILURE_LIMIT {
            let _ = register_unlock_failure(&mut throttle, now);
        }
        assert!(active_cooldown_seconds(&mut throttle, now).is_some());

        let after_expiry = now + Duration::from_secs(UNLOCK_COOLDOWN_SECONDS + 1);
        assert_eq!(active_cooldown_seconds(&mut throttle, after_expiry), None);
        assert_eq!(throttle.failed_attempts, 0);

        register_unlock_failure(&mut throttle, now);
        assert_eq!(throttle.failed_attempts, 1);
        reset_unlock_throttle(&mut throttle);
        assert_eq!(throttle.failed_attempts, 0);
        assert!(throttle.cooldown_until.is_none());
    }

    #[test]
    fn test_verify_current_password_with_wrong_input() {
        let stored = hash_password("correct-password");
        assert_eq!(
            verify_current_password(Some(stored), "bad-password"),
            Err("wrong_password")
        );
    }

    #[test]
    fn test_verify_current_password_when_not_set() {
        assert_eq!(verify_current_password(None, "any"), Err("not_set"));
    }

    #[test]
    fn test_lock_state_after_require_toggle() {
        assert!(lock_state_after_require_toggle(true));
        assert!(!lock_state_after_require_toggle(false));
    }
}
