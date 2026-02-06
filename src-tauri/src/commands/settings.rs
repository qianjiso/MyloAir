//! 用户设置 Commands

use crate::models::{UserSetting, UserSettingsCategory};
use crate::AppState;
use tauri::State;
use serde_json::{json, Value};

#[tauri::command]
pub async fn get_user_settings(state: State<'_, AppState>, category: Option<String>) -> Result<Vec<UserSetting>, String> {
    state.db.get_user_settings(category.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_setting(state: State<'_, AppState>, key: String) -> Result<Option<UserSetting>, String> {
    state.db.get_user_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_user_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
    type_: Option<String>,
    category: Option<String>,
    description: Option<String>,
) -> Result<Value, String> {
    let setting = UserSetting {
        id: None,
        key: key.clone(),
        value,
        r#type: type_,
        category,
        description,
        created_at: None,
        updated_at: None,
    };
    state.db.set_user_setting(&setting).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn update_user_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<Value, String> {
    // Fetch existing matches
    if let Some(mut setting) = state.db.get_user_setting(&key).map_err(|e| e.to_string())? {
        setting.value = value;
        state.db.set_user_setting(&setting).map_err(|e| e.to_string())?;
        Ok(json!({ "success": true }))
    } else {
        // Create new if not exists with default type/category?
        // Let's assume defaults.
        let setting = UserSetting {
            id: None,
            key: key.clone(),
            value,
            r#type: Some("string".to_string()),
            category: Some("general".to_string()),
            description: None,
            created_at: None,
            updated_at: None,
        };
        state.db.set_user_setting(&setting).map_err(|e| e.to_string())?;
        Ok(json!({ "success": true }))
    }
}

#[tauri::command]
pub async fn delete_user_setting(state: State<'_, AppState>, key: String) -> Result<Value, String> {
    state.db.delete_user_setting(&key).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn get_user_settings_categories(_state: State<'_, AppState>) -> Result<Vec<UserSettingsCategory>, String> {
    // Mock implementation as we don't have categories table.
    // Return hardcoded list used in app?
    // Or select distinct categories from settings?
    Ok(vec![
        UserSettingsCategory { category: "general".to_string(), description: "通用".to_string(), settings: vec![] },
        UserSettingsCategory { category: "security".to_string(), description: "安全".to_string(), settings: vec![] },
        UserSettingsCategory { category: "appearance".to_string(), description: "外观".to_string(), settings: vec![] },
    ])
}
