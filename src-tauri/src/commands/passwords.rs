//! 密码管理 Commands

use crate::models::{Password, PasswordSearchResult, PasswordHistory};
use serde_json::Value;
use tauri::State;
use crate::AppState;

/// 辅助函数：解密密码字段
fn decrypt_password_field(state: &State<'_, AppState>, p: &mut Password) {
    if let Some(cipher) = &p.password {
        if !cipher.is_empty() {
            if let Ok(plain) = state.encryption.decrypt(cipher) {
                p.password = Some(plain);
            }
        }
    }
}

/// 辅助函数：加密密码字段
fn encrypt_password_field(state: &State<'_, AppState>, p: &mut Password) -> Result<(), String> {
    if let Some(plain) = &p.password {
        if !plain.is_empty() {
            let cipher = state.encryption.encrypt(plain)?;
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
    
    let id = state.db.add_password(&password).map_err(|e| {
        log::error!("Failed to add password to database: {}", e);
        e.to_string()
    })?;
    
    log::info!("Password added successfully with id: {}", id);
    
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
    
    // 获取旧密码用于历史记录
    let old_password_encrypted = if let Some(old_pwd) = state.db.get_password(id).map_err(|e| e.to_string())? {
        old_pwd.password
    } else {
        return Err("Password not found".to_string());
    };
    
    // 确保 ID 一致
    password.id = Some(id);
    
    // 加密新密码
    encrypt_password_field(&state, &mut password)?;
    
    // 如果密码发生变化，保存历史记录
    if let (Some(old_pwd), Some(new_pwd)) = (&old_password_encrypted, &password.password) {
        if old_pwd != new_pwd {
            state.db.add_password_history(id, old_pwd, Some("密码更新")).map_err(|e| e.to_string())?;
        }
    }
    
    // 更新密码
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
    
    let passwords = state.db.search_passwords(&keyword).map_err(|e| e.to_string())?;
    
    // 获取所有分组用于查找分组名称
    let groups = state.db.get_groups().map_err(|e| e.to_string())?;
    let group_map: std::collections::HashMap<i64, String> = groups
        .into_iter()
        .filter_map(|g| g.id.map(|id| (id, g.name)))
        .collect();
    
    log::info!("Group map: {:?}", group_map);
    
    // 转换为 PasswordSearchResult 并填充 group_name
    let results: Vec<PasswordSearchResult> = passwords.into_iter().map(|p| {
        let group_name = p.group_id.and_then(|gid| group_map.get(&gid).cloned());
        log::info!("Password: id={}, title={}, group_id={:?}, group_name={:?}", 
                   p.id.unwrap_or(0), p.title, p.group_id, group_name);
        PasswordSearchResult {
            id: p.id.unwrap_or(0),
            title: p.title,
            username: p.username,
            url: p.url,
            group_id: p.group_id,
            group_name,
        }
    }).collect();
    
    log::info!("Search results count: {}", results.len());
    
    Ok(results)
}

/// 密码生成器选项
#[derive(serde::Deserialize)]
pub struct PasswordGeneratorOptions {
    length: Option<usize>,
    #[serde(rename = "includeUppercase")]
    include_uppercase: Option<bool>,
    #[serde(rename = "includeLowercase")]
    include_lowercase: Option<bool>,
    #[serde(rename = "includeNumbers")]
    include_numbers: Option<bool>,
    #[serde(rename = "includeSymbols")]
    include_symbols: Option<bool>,
}

/// 生成随机密码
#[tauri::command]
pub async fn generate_password(options: PasswordGeneratorOptions) -> Result<String, String> {
    use rand::Rng;
    
    let length = options.length.unwrap_or(16).max(4).min(128);
    let include_uppercase = options.include_uppercase.unwrap_or(true);
    let include_lowercase = options.include_lowercase.unwrap_or(true);
    let include_numbers = options.include_numbers.unwrap_or(true);
    let include_symbols = options.include_symbols.unwrap_or(true);

    let mut charset = String::new();
    if include_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if include_lowercase {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if include_numbers {
        charset.push_str("0123456789");
    }
    if include_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    if charset.is_empty() {
        return Err("至少需要选择一种字符类型".to_string());
    }

    let charset_chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| charset_chars[rng.gen_range(0..charset_chars.len())])
        .collect();

    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_password_default() {
        let options = PasswordGeneratorOptions {
            length: Some(16),
            include_uppercase: Some(true),
            include_lowercase: Some(true),
            include_numbers: Some(true),
            include_symbols: Some(true),
        };
        
        let result = generate_password(options).await;
        assert!(result.is_ok());
        
        let password = result.unwrap();
        assert_eq!(password.len(), 16);
    }

    #[tokio::test]
    async fn test_generate_password_length_limits() {
        // 测试最小长度
        let options = PasswordGeneratorOptions {
            length: Some(2), // 会被限制为 4
            include_lowercase: Some(true),
            include_uppercase: None,
            include_numbers: None,
            include_symbols: None,
        };
        
        let result = generate_password(options).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);

        // 测试最大长度
        let options = PasswordGeneratorOptions {
            length: Some(200), // 会被限制为 128
            include_lowercase: Some(true),
            include_uppercase: None,
            include_numbers: None,
            include_symbols: None,
        };
        
        let result = generate_password(options).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 128);
    }

    #[tokio::test]
    async fn test_generate_password_no_charset() {
        let options = PasswordGeneratorOptions {
            length: Some(16),
            include_uppercase: Some(false),
            include_lowercase: Some(false),
            include_numbers: Some(false),
            include_symbols: Some(false),
        };
        
        let result = generate_password(options).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "至少需要选择一种字符类型");
    }

    #[tokio::test]
    async fn test_generate_password_only_uppercase() {
        let options = PasswordGeneratorOptions {
            length: Some(10),
            include_uppercase: Some(true),
            include_lowercase: Some(false),
            include_numbers: Some(false),
            include_symbols: Some(false),
        };
        
        let result = generate_password(options).await;
        assert!(result.is_ok());
        
        let password = result.unwrap();
        assert_eq!(password.len(), 10);
        assert!(password.chars().all(|c| c.is_uppercase()));
    }
}

/// 获取密码历史记录
#[tauri::command]
pub async fn get_password_history(
    state: State<'_, AppState>,
    password_id: i64,
) -> Result<Vec<serde_json::Value>, String> {
    let history = state.db.get_password_history(password_id).map_err(|e| e.to_string())?;
    
    // 获取当前密码作为 new_password
    let current_password = state.db.get_password(password_id)
        .map_err(|e| e.to_string())?
        .and_then(|p| p.password)
        .unwrap_or_default();
    
    // 解密当前密码
    let current_password_decrypted = if !current_password.is_empty() {
        state.encryption.decrypt(&current_password).unwrap_or(current_password.clone())
    } else {
        current_password
    };
    
    // 构建返回结果，解密旧密码并添加 new_password 字段
    let results: Vec<serde_json::Value> = history.into_iter().map(|h| {
        // 解密旧密码
        let old_password_decrypted = state.encryption.decrypt(&h.old_password)
            .unwrap_or(h.old_password.clone());
        
        serde_json::json!({
            "id": h.id,
            "password_id": h.password_id,
            "old_password": old_password_decrypted,
            "new_password": current_password_decrypted.clone(),
            "changed_at": h.changed_at,
            "change_reason": h.change_reason,
        })
    }).collect();
    
    Ok(results)
}
