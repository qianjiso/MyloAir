//! 备份与导入导出 Commands
//!
//! 数据流：
//!   导入: 备份JSON(明文) → encrypt → DB(密文)
//!   导出: DB(密文) → decrypt → 备份JSON(明文)
//! 支持格式: JSON / 加密ZIP(AES-256)

use crate::AppState;
use crate::services::encryption::EncryptionService;
use tauri::State;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

/// 导出所有数据为 JSON 字节数组，支持 json 和 encrypted_zip 两种格式
#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    _options: Value,
) -> Result<Value, String> {
    log::info!("export_data called with options: {:?}", _options);

    let format = _options.get("format").and_then(|v| v.as_str()).unwrap_or("json");
    let archive_password = _options.get("archivePassword").and_then(|v| v.as_str());

    let db = &state.db;
    let encryption = &state.encryption;
    let conn = db.get_connection().map_err(|e| format!("数据库连接失败: {}", e))?;

    // ─── 构建 JSON 备份数据 ───

    // 1. 导出 groups
    let mut groups_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at FROM groups ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, Option<i64>>(0)?,
                "name": row.get::<_, String>(1)?,
                "parent_id": row.get::<_, Option<i64>>(2)?,
                "color": row.get::<_, Option<String>>(4)?,
                "order_index": 0,
                "sort": row.get::<_, Option<i32>>(5)?,
                "created_at": row.get::<_, Option<String>>(6)?,
                "updated_at": row.get::<_, Option<String>>(7)?
            }))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            groups_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    // 2. 导出 passwords（解密 password 字段）
    let mut passwords_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, title, username, password, url, notes, group_id, created_at, updated_at FROM passwords ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let (id, title, username, cipher_pwd, url, notes, group_id, created_at, updated_at) =
                row.map_err(|e| e.to_string())?;
            let plain_pwd = decrypt_field(encryption, &cipher_pwd);
            passwords_arr.push(json!({
                "id": id,
                "title": title,
                "username": username,
                "password": plain_pwd,
                "url": url,
                "notes": notes,
                "multi_accounts": Value::Null,
                "group_id": group_id,
                "created_at": created_at,
                "updated_at": updated_at
            }));
        }
    }

    // 3. 导出 note_groups
    let mut note_groups_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, color, sort_order, created_at, updated_at FROM secure_record_groups ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, Option<i64>>(0)?,
                "name": row.get::<_, String>(1)?,
                "parent_id": row.get::<_, Option<i64>>(2)?,
                "color": row.get::<_, Option<String>>(3)?,
                "sort_order": row.get::<_, Option<i32>>(4)?,
                "created_at": row.get::<_, Option<String>>(5)?,
                "updated_at": row.get::<_, Option<String>>(6)?
            }))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            note_groups_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    // 4. 导出 notes（解密 content 字段）
    let mut notes_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, title, content, group_id, pinned, archived, created_at, updated_at FROM secure_records ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<i32>>(4)?,
                row.get::<_, Option<i32>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            let (id, title, cipher_content, group_id, pinned, archived, created_at, updated_at) =
                row.map_err(|e| e.to_string())?;
            let plain_content = decrypt_field(encryption, &cipher_content);
            notes_arr.push(json!({
                "id": id,
                "title": title,
                "content_ciphertext": plain_content,
                "group_id": group_id,
                "pinned": pinned,
                "archived": archived,
                "created_at": created_at,
                "updated_at": updated_at
            }));
        }
    }

    // 5. 导出 user_settings
    let mut settings_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, key, value, type, category, description, created_at, updated_at FROM user_settings ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, Option<i64>>(0)?,
                "key": row.get::<_, String>(1)?,
                "value": row.get::<_, Option<String>>(2)?,
                "type": row.get::<_, Option<String>>(3)?,
                "category": row.get::<_, Option<String>>(4)?,
                "description": row.get::<_, Option<String>>(5)?,
                "created_at": row.get::<_, Option<String>>(6)?,
                "updated_at": row.get::<_, Option<String>>(7)?
            }))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            settings_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    // 6. 导出 password_history
    let mut history_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, password_id, old_password, changed_at, change_reason FROM password_history ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, Option<i64>>(0)?,
                "password_id": row.get::<_, i64>(1)?,
                "old_password": row.get::<_, String>(2)?,
                "changed_at": row.get::<_, Option<String>>(3)?,
                "changed_reason": row.get::<_, Option<String>>(4)?
            }))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            history_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    let backup = json!({
        "version": "1.0",
        "exported_at": chrono_now_iso(),
        "app_name": "Password Manager",
        "passwords": passwords_arr,
        "groups": groups_arr,
        "note_groups": note_groups_arr,
        "notes": notes_arr,
        "user_settings": settings_arr,
        "password_history": history_arr
    });

    let json_bytes = serde_json::to_vec_pretty(&backup).map_err(|e| e.to_string())?;

    // ─── 根据格式输出 ───
    let output_bytes = if format == "encrypted_zip" {
        let password = archive_password.ok_or("加密ZIP格式需要提供 archivePassword")?;
        create_encrypted_zip(&json_bytes, password)?
    } else {
        json_bytes
    };

    let data: Vec<i32> = output_bytes.iter().map(|b| *b as i32).collect();
    Ok(json!({ "success": true, "data": data }))
}

/// 创建 AES-256 加密的 ZIP 文件，内含 backup.json
fn create_encrypted_zip(json_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .with_aes_encryption(zip::AesMode::Aes256, password);

    zip.start_file("backup.json", options)
        .map_err(|e| format!("创建ZIP条目失败: {}", e))?;
    zip.write_all(json_bytes)
        .map_err(|e| format!("写入ZIP数据失败: {}", e))?;

    let result = zip.finish().map_err(|e| format!("完成ZIP文件失败: {}", e))?;
    Ok(result.into_inner())
}

/// 从加密 ZIP 中读取 backup.json
fn read_encrypted_zip(zip_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let reader = Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("读取ZIP文件失败: {}", e))?;

    let mut file = archive.by_name_decrypt("backup.json", password.as_bytes())
        .map_err(|e| format!("ZIP解密失败（密码错误或文件损坏）: {}", e))?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| format!("读取ZIP内容失败: {}", e))?;

    Ok(contents)
}

/// 导出数据到指定文件
#[tauri::command]
pub async fn export_data_to_file(
    state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    log::info!("export_data_to_file called with options: {:?}", options);

    let file_path = options.get("filePath")
        .and_then(|v| v.as_str())
        .ok_or("缺少 filePath 参数")?
        .to_string();

    let export_result = export_data(state, options).await?;

    if let Some(data) = export_result.get("data") {
        let bytes: Vec<u8> = data.as_array()
            .ok_or("data 不是数组")?
            .iter()
            .map(|v| v.as_i64().unwrap_or(0) as u8)
            .collect();

        std::fs::write(&file_path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;
        Ok(json!({ "success": true, "filePath": file_path }))
    } else {
        Err("导出数据失败".to_string())
    }
}

/// 导入备份数据（自动检测 JSON / 加密ZIP）
#[tauri::command]
pub async fn import_data(
    state: State<'_, AppState>,
    data: Vec<u8>,
    _options: Value,
) -> Result<Value, String> {
    log::info!("import_data called, data length: {}", data.len());

    // 自动检测格式：ZIP 文件以 "PK" (0x50 0x4B) 开头
    let is_zip = data.len() >= 2 && data[0] == 0x50 && data[1] == 0x4B;

    let json_bytes = if is_zip {
        let password = _options.get("archivePassword")
            .and_then(|v| v.as_str())
            .ok_or("导入加密ZIP需要提供密码")?;
        log::info!("检测到ZIP格式，尝试解密...");
        read_encrypted_zip(&data, password)?
    } else {
        data
    };

    let backup: Value = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    let db = &state.db;
    let encryption = &state.encryption;
    let conn = db.get_connection().map_err(|e| format!("数据库连接失败: {}", e))?;

    conn.execute_batch("BEGIN TRANSACTION;").map_err(|e| e.to_string())?;

    let result = do_import(&conn, &backup, encryption);

    match result {
        Ok(stats) => {
            conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?;
            log::info!("导入完成: {:?}", stats);
            Ok(json!({
                "success": true,
                "data": {
                    "imported": stats.total_imported,
                    "skipped": stats.total_skipped,
                    "errors": stats.errors
                }
            }))
        }
        Err(e) => {
            conn.execute_batch("ROLLBACK;").ok();
            log::error!("导入失败，已回滚: {}", e);
            Err(format!("导入失败: {}", e))
        }
    }
}

#[derive(Debug)]
struct ImportStats {
    total_imported: usize,
    total_skipped: usize,
    errors: Vec<String>,
}

/// 在事务内执行实际导入逻辑（UPSERT 模式）
fn do_import(conn: &rusqlite::Connection, backup: &Value, encryption: &EncryptionService) -> Result<ImportStats, String> {
    let mut stats = ImportStats {
        total_imported: 0,
        total_skipped: 0,
        errors: Vec::new(),
    };

    // --- 1. 导入 groups（按 name + parent_id 查重） ---
    let mut group_id_map: HashMap<i64, i64> = HashMap::new();
    if let Some(groups) = backup.get("groups").and_then(|v| v.as_array()) {
        let (top_groups, child_groups): (Vec<&Value>, Vec<&Value>) = groups.iter().partition(|g| {
            g.get("parent_id").map_or(true, |v| v.is_null())
        });

        for group in top_groups.iter().chain(child_groups.iter()) {
            let old_id = group.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = group.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let color = group.get("color").and_then(|v| v.as_str());
            let sort_order = group.get("sort").and_then(|v| v.as_i64())
                .or_else(|| group.get("sort_order").and_then(|v| v.as_i64()));
            let parent_id = group.get("parent_id").and_then(|v| v.as_i64());
            let mapped_parent_id = parent_id.and_then(|pid| group_id_map.get(&pid).copied());

            let existing: Option<i64> = conn.query_row(
                "SELECT id FROM groups WHERE name = ?1 AND (parent_id IS ?2)",
                rusqlite::params![name, mapped_parent_id],
                |row| row.get(0),
            ).ok();

            let new_id = if let Some(eid) = existing {
                conn.execute(
                    "UPDATE groups SET color = ?1, sort_order = ?2, updated_at = datetime('now') WHERE id = ?3",
                    rusqlite::params![color, sort_order, eid],
                ).map_err(|e| e.to_string())?;
                log::info!("更新分组: {} (id={})", name, eid);
                eid
            } else {
                conn.execute(
                    "INSERT INTO groups (name, parent_id, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    rusqlite::params![name, mapped_parent_id, color, sort_order],
                ).map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            };

            group_id_map.insert(old_id, new_id);
            stats.total_imported += 1;
            log::info!("导入分组: {} (old_id={} -> new_id={})", name, old_id, new_id);
        }
    }

    // --- 2. 导入 passwords（明文→加密存储，按 title+username 查重） ---
    if let Some(passwords) = backup.get("passwords").and_then(|v| v.as_array()) {
        for pwd in passwords {
            let title = pwd.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let username = pwd.get("username").and_then(|v| v.as_str());
            let plain_password = pwd.get("password").and_then(|v| v.as_str());
            let url = pwd.get("url").and_then(|v| v.as_str());
            let notes = pwd.get("notes").and_then(|v| v.as_str());
            let old_group_id = pwd.get("group_id").and_then(|v| v.as_i64());
            let mapped_group_id = old_group_id.and_then(|gid| group_id_map.get(&gid).copied());

            let encrypted_pwd = encrypt_field(encryption, plain_password);

            let existing: Option<i64> = conn.query_row(
                "SELECT id FROM passwords WHERE title = ?1 AND (username IS ?2)",
                rusqlite::params![title, username],
                |row| row.get(0),
            ).ok();

            if let Some(eid) = existing {
                conn.execute(
                    "UPDATE passwords SET password = ?1, url = ?2, notes = ?3, group_id = ?4, updated_at = datetime('now') WHERE id = ?5",
                    rusqlite::params![encrypted_pwd, url, notes, mapped_group_id, eid],
                ).map_err(|e| e.to_string())?;
                log::info!("更新密码: {} (id={})", title, eid);
            } else {
                conn.execute(
                    "INSERT INTO passwords (title, username, password, url, notes, group_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
                    rusqlite::params![title, username, encrypted_pwd, url, notes, mapped_group_id],
                ).map_err(|e| e.to_string())?;
            }
            stats.total_imported += 1;
        }
    }

    // --- 3. 导入 note_groups（按 name + parent_id 查重） ---
    let mut note_group_id_map: HashMap<i64, i64> = HashMap::new();
    if let Some(note_groups) = backup.get("note_groups").and_then(|v| v.as_array()) {
        let (top_groups, child_groups): (Vec<&Value>, Vec<&Value>) = note_groups.iter().partition(|g| {
            g.get("parent_id").map_or(true, |v| v.is_null())
        });

        for group in top_groups.iter().chain(child_groups.iter()) {
            let old_id = group.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = group.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let color = group.get("color").and_then(|v| v.as_str());
            let sort_order = group.get("sort_order").and_then(|v| v.as_i64())
                .or_else(|| group.get("sort").and_then(|v| v.as_i64()));
            let parent_id = group.get("parent_id").and_then(|v| v.as_i64());
            let mapped_parent_id = parent_id.and_then(|pid| note_group_id_map.get(&pid).copied());

            let existing: Option<i64> = conn.query_row(
                "SELECT id FROM secure_record_groups WHERE name = ?1 AND (parent_id IS ?2)",
                rusqlite::params![name, mapped_parent_id],
                |row| row.get(0),
            ).ok();

            let new_id = if let Some(eid) = existing {
                conn.execute(
                    "UPDATE secure_record_groups SET color = ?1, sort_order = ?2, updated_at = datetime('now') WHERE id = ?3",
                    rusqlite::params![color, sort_order, eid],
                ).map_err(|e| e.to_string())?;
                log::info!("更新笔记分组: {} (id={})", name, eid);
                eid
            } else {
                conn.execute(
                    "INSERT INTO secure_record_groups (name, parent_id, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    rusqlite::params![name, mapped_parent_id, color, sort_order],
                ).map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            };

            note_group_id_map.insert(old_id, new_id);
            stats.total_imported += 1;
            log::info!("导入笔记分组: {} (old_id={} -> new_id={})", name, old_id, new_id);
        }
    }

    // --- 4. 导入 notes（明文→加密存储，按 title + group_id 查重） ---
    if let Some(notes) = backup.get("notes").and_then(|v| v.as_array()) {
        for note in notes {
            let title = note.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let plain_content = note.get("content_ciphertext").and_then(|v| v.as_str())
                .or_else(|| note.get("content").and_then(|v| v.as_str()));
            let old_group_id = note.get("group_id").and_then(|v| v.as_i64());
            let mapped_group_id = old_group_id.and_then(|gid| note_group_id_map.get(&gid).copied());
            let pinned = note.get("pinned").and_then(|v| v.as_i64()).unwrap_or(0);
            let archived = note.get("archived").and_then(|v| v.as_i64()).unwrap_or(0);

            let encrypted_content = encrypt_field(encryption, plain_content);

            let existing: Option<i64> = conn.query_row(
                "SELECT id FROM secure_records WHERE title = ?1 AND (group_id IS ?2)",
                rusqlite::params![title, mapped_group_id],
                |row| row.get(0),
            ).ok();

            if let Some(eid) = existing {
                conn.execute(
                    "UPDATE secure_records SET content = ?1, pinned = ?2, archived = ?3, updated_at = datetime('now') WHERE id = ?4",
                    rusqlite::params![encrypted_content, pinned, archived, eid],
                ).map_err(|e| e.to_string())?;
                log::info!("更新笔记: {} (id={})", title, eid);
            } else {
                conn.execute(
                    "INSERT INTO secure_records (title, content, group_id, pinned, archived, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
                    rusqlite::params![title, encrypted_content, mapped_group_id, pinned, archived],
                ).map_err(|e| e.to_string())?;
            }
            stats.total_imported += 1;
        }
    }

    // --- 5. 导入 user_settings（UPSERT on key） ---
    if let Some(settings) = backup.get("user_settings").and_then(|v| v.as_array()) {
        for setting in settings {
            let key = setting.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let value = setting.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let stype = setting.get("type").and_then(|v| v.as_str());
            let category = setting.get("category").and_then(|v| v.as_str());
            let description = setting.get("description").and_then(|v| v.as_str());

            let result = conn.execute(
                "INSERT INTO user_settings (key, value, type, category, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value, type=excluded.type, category=excluded.category, description=excluded.description, updated_at=datetime('now')",
                rusqlite::params![key, value, stype, category, description],
            );

            match result {
                Ok(_) => { stats.total_imported += 1; }
                Err(e) => {
                    let msg = format!("导入设置 '{}' 失败: {}", key, e);
                    log::warn!("{}", msg);
                    stats.errors.push(msg);
                    stats.total_skipped += 1;
                }
            }
        }
    }

    // --- 6. 跳过 password_history ---
    if let Some(history) = backup.get("password_history").and_then(|v| v.as_array()) {
        if !history.is_empty() {
            log::info!("跳过 {} 条密码历史记录（password_id 映射不可用）", history.len());
            stats.total_skipped += history.len();
        }
    }

    Ok(stats)
}

/// 解密字段：密文→明文，解密失败则原样返回
fn decrypt_field(encryption: &EncryptionService, cipher: &Option<String>) -> Option<String> {
    match cipher {
        Some(text) if !text.is_empty() => {
            match encryption.decrypt(text) {
                Ok(plain) => Some(plain),
                Err(_) => Some(text.clone()),
            }
        }
        other => other.clone(),
    }
}

/// 加密字段：明文→密文
fn encrypt_field(encryption: &EncryptionService, plain: Option<&str>) -> Option<String> {
    match plain {
        Some(text) if !text.is_empty() => {
            match encryption.encrypt(text) {
                Ok(cipher) => Some(cipher),
                Err(e) => {
                    log::warn!("加密失败: {}，将以明文存储", e);
                    Some(text.to_string())
                }
            }
        }
        _ => None,
    }
}

/// 选择导出路径
#[tauri::command]
pub async fn pick_export_path(
    _state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    log::info!("pick_export_path: {:?}", options);
    Ok(json!({ "success": true, "filePath": null }))
}

/// 选择导出目录
#[tauri::command]
pub async fn pick_export_directory(
    _state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    log::info!("pick_export_directory: {:?}", options);
    Ok(json!({ "success": true, "directory": null }))
}

/// 生成当前 ISO 时间戳
fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}Z", secs)
}
