//! 备份与云备份 Commands
//!
//! 数据流：
//!   导入: 备份JSON(明文) -> encrypt -> DB(密文)
//!   导出: DB(密文) -> decrypt -> 备份JSON(明文)
//!   云端: DB(密文) -> decrypt -> encrypted_zip -> COS

use crate::services::encryption::EncryptionService;
use crate::AppState;
use chrono::{Datelike, Local, Timelike, Utc};
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, HOST},
    Client, Method, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

type HmacSha256 = Hmac<Sha256>;

const BACKUP_FILENAME_PREFIX: &str = "myloair-backup-";
const CLOUD_PROVIDER: &str = "cos";
const AWS_SERVICE_NAME: &str = "s3";
const BACKUP_FAILURE_NOTIFY_COOLDOWN_SECS: u64 = 300;
const URI_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b'/');
const QUERY_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

#[derive(Debug, Clone)]
struct BackupConfig {
    target_mode: String,
    auto_export_enabled: bool,
    auto_export_frequency: String,
    auto_export_directory: String,
    auto_export_format: String,
    auto_export_password: Option<String>,
    auto_export_time_of_day: String,
    auto_export_day_of_week: i64,
    auto_export_day_of_month: i64,
    auto_export_interval_minutes: i64,
    retention_count: usize,
    cloud_provider: String,
    cloud_endpoint: String,
    cloud_bucket: String,
    cloud_region: String,
    cloud_path_prefix: String,
    cloud_secret_id: Option<String>,
    cloud_secret_key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRunStatus {
    at: Option<String>,
    result: Option<String>,
    target: Option<String>,
    file: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfigResponse {
    target_mode: String,
    auto_export_enabled: bool,
    auto_export_frequency: String,
    auto_export_directory: String,
    export_format: String,
    auto_export_time_of_day: String,
    auto_export_day_of_week: i64,
    auto_export_day_of_month: i64,
    auto_export_interval_minutes: i64,
    retention_count: usize,
    cloud_provider: String,
    endpoint: String,
    bucket: String,
    region: String,
    path_prefix: String,
    secret_id_masked: Option<String>,
    has_secret_key: bool,
    has_archive_password: bool,
    failure_notification_cooldown_minutes: u64,
    last_manual_run: BackupRunStatus,
    last_auto_run: BackupRunStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBackupConfigInput {
    target_mode: Option<String>,
    retention_count: Option<usize>,
    endpoint: Option<String>,
    bucket: Option<String>,
    region: Option<String>,
    path_prefix: Option<String>,
    secret_id: Option<String>,
    secret_key: Option<String>,
    export_default_password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestBackupCloudInput {
    endpoint: String,
    bucket: String,
    region: String,
    path_prefix: Option<String>,
    secret_id: Option<String>,
    secret_key: Option<String>,
    export_default_password: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudTestResponse {
    success: bool,
    category: Option<String>,
    message: String,
    warning: Option<String>,
}

#[derive(Debug, Clone)]
struct CloudConfig {
    endpoint: String,
    bucket: String,
    region: String,
    path_prefix: String,
    secret_id: String,
    secret_key: String,
}

#[derive(Debug, Clone)]
struct CloudObject {
    key: String,
}

#[derive(Debug)]
struct CloudError {
    category: &'static str,
    message: String,
}

impl CloudError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

/// 导出所有数据为字节数组，支持 json 和 encrypted_zip 两种格式
#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    log::info!("export_data called");

    let format = options
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("json");
    let archive_password = options.get("archivePassword").and_then(|v| v.as_str());

    let json_bytes = build_backup_json_bytes(&state)?;

    let output_bytes = if format == "encrypted_zip" {
        let password = archive_password.ok_or("加密ZIP格式需要提供 archivePassword")?;
        create_encrypted_zip(&json_bytes, password)?
    } else {
        json_bytes
    };

    let data: Vec<i32> = output_bytes.iter().map(|b| *b as i32).collect();
    Ok(json!({ "success": true, "data": data }))
}

/// 导出数据到指定文件
#[tauri::command]
pub async fn export_data_to_file(
    state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    log::info!("export_data_to_file called");

    let file_path = options
        .get("filePath")
        .and_then(|v| v.as_str())
        .ok_or("缺少 filePath 参数")?
        .to_string();

    let export_result = export_data(state, options).await?;

    if let Some(data) = export_result.get("data") {
        let bytes: Vec<u8> = data
            .as_array()
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
    options: Value,
) -> Result<Value, String> {
    log::info!("import_data called, data length: {}", data.len());

    let is_zip = data.len() >= 2 && data[0] == 0x50 && data[1] == 0x4B;

    let json_bytes = if is_zip {
        let password = options
            .get("archivePassword")
            .and_then(|v| v.as_str())
            .ok_or("导入加密ZIP需要提供密码")?;
        read_encrypted_zip(&data, password)?
    } else {
        data
    };

    let backup: Value =
        serde_json::from_slice(&json_bytes).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let db = &state.db;
    let encryption = &state.encryption;
    let conn = db
        .get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| e.to_string())?;

    let result = do_import(&conn, &backup, encryption);

    match result {
        Ok(stats) => {
            conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?;
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
            Err(format!("导入失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_backup_config(state: State<'_, AppState>) -> Result<BackupConfigResponse, String> {
    let config = load_backup_config(&state)?;
    Ok(BackupConfigResponse {
        target_mode: config.target_mode,
        auto_export_enabled: config.auto_export_enabled,
        auto_export_frequency: config.auto_export_frequency,
        auto_export_directory: config.auto_export_directory,
        export_format: config.auto_export_format,
        auto_export_time_of_day: config.auto_export_time_of_day,
        auto_export_day_of_week: config.auto_export_day_of_week,
        auto_export_day_of_month: config.auto_export_day_of_month,
        auto_export_interval_minutes: config.auto_export_interval_minutes,
        retention_count: config.retention_count,
        cloud_provider: config.cloud_provider,
        endpoint: config.cloud_endpoint,
        bucket: config.cloud_bucket,
        region: config.cloud_region,
        path_prefix: config.cloud_path_prefix,
        secret_id_masked: config.cloud_secret_id.as_deref().map(mask_secret_id),
        has_secret_key: config
            .cloud_secret_key
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        has_archive_password: config
            .auto_export_password
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        failure_notification_cooldown_minutes: BACKUP_FAILURE_NOTIFY_COOLDOWN_SECS / 60,
        last_manual_run: get_backup_run_status(&state, "last_manual"),
        last_auto_run: get_backup_run_status(&state, "last_auto"),
    })
}

#[tauri::command]
pub async fn save_backup_config(
    state: State<'_, AppState>,
    input: SaveBackupConfigInput,
) -> Result<Value, String> {
    if let Some(target_mode) = input.target_mode {
        if target_mode != "local" && target_mode != "cos" {
            return Err("targetMode 仅支持 local 或 cos".to_string());
        }
        save_plain_setting(
            &state,
            "backup.target_mode",
            target_mode,
            "string",
            "backup",
            "备份目标模式",
        )?;
    }

    if let Some(retention_count) = input.retention_count {
        let value = retention_count.max(1).min(365).to_string();
        save_plain_setting(
            &state,
            "backup.retention_count",
            value,
            "number",
            "backup",
            "备份保留份数",
        )?;
    }

    if let Some(endpoint) = input.endpoint {
        save_plain_setting(
            &state,
            "backup.cloud.endpoint",
            endpoint.trim().to_string(),
            "string",
            "backup",
            "对象存储 Endpoint",
        )?;
    }
    if let Some(bucket) = input.bucket {
        save_plain_setting(
            &state,
            "backup.cloud.bucket",
            bucket.trim().to_string(),
            "string",
            "backup",
            "对象存储 Bucket",
        )?;
    }
    if let Some(region) = input.region {
        save_plain_setting(
            &state,
            "backup.cloud.region",
            region.trim().to_string(),
            "string",
            "backup",
            "对象存储 Region",
        )?;
    }
    if let Some(path_prefix) = input.path_prefix {
        save_plain_setting(
            &state,
            "backup.cloud.path_prefix",
            normalize_path_prefix(&path_prefix),
            "string",
            "backup",
            "对象存储前缀",
        )?;
    }

    save_plain_setting(
        &state,
        "backup.cloud.provider",
        CLOUD_PROVIDER.to_string(),
        "string",
        "backup",
        "云备份提供商",
    )?;

    if let Some(secret_id) = input.secret_id {
        let trimmed = secret_id.trim();
        if !trimmed.is_empty() {
            save_sensitive_setting(
                &state,
                "backup.cloud.secret_id",
                trimmed,
                "string",
                "backup",
                "对象存储 SecretId",
            )?;
        }
    }

    if let Some(secret_key) = input.secret_key {
        let trimmed = secret_key.trim();
        if !trimmed.is_empty() {
            save_sensitive_setting(
                &state,
                "backup.cloud.secret_key",
                trimmed,
                "string",
                "backup",
                "对象存储 SecretKey",
            )?;
        }
    }

    if let Some(password) = input.export_default_password {
        let trimmed = password.trim();
        if !trimmed.is_empty() {
            save_sensitive_setting(
                &state,
                "backup.auto_export_password",
                trimmed,
                "string",
                "backup",
                "自动导出加密密码",
            )?;
        }
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn test_backup_cloud_connection(
    state: State<'_, AppState>,
    input: TestBackupCloudInput,
) -> Result<CloudTestResponse, String> {
    let client = build_http_client()?;
    let config = resolve_test_cloud_config(&state, &input).await?;
    let archive_password = resolve_test_archive_password(&state, &input)?;
    let test_payload = json!({
        "kind": "myloair-cloud-backup-test",
        "generatedAt": Local::now().to_rfc3339(),
    })
    .to_string();
    let test_zip = create_encrypted_zip(test_payload.as_bytes(), &archive_password)
        .map_err(|e| format!("生成加密测试备份失败: {}", e))?;

    let test_key = format!(
        "{}.__myloair_test_upload__{}.zip",
        config.path_prefix,
        Local::now().timestamp_millis()
    );

    let warning = match put_object(&client, &config, &test_key, &test_zip).await {
        Ok(()) => match delete_object(&client, &config, &test_key).await {
            Ok(()) => None,
            Err(err) => Some(format!("上传成功，但清理测试对象失败: {}", err.message)),
        },
        Err(err) => {
            return Ok(CloudTestResponse {
                success: false,
                category: Some(err.category.to_string()),
                message: err.message,
                warning: None,
            });
        }
    };

    Ok(CloudTestResponse {
        success: true,
        category: None,
        message: "连接成功，已验证加密备份上传权限".to_string(),
        warning,
    })
}

#[tauri::command]
pub async fn trigger_manual_cloud_backup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let result = execute_backup_run(&app, &state, "manual", "cos").await;

    match result {
        Ok(outcome) => {
            app.emit(
                "backup-manual-done",
                json!({
                    "success": true,
                    "target": outcome.target,
                    "file": outcome.file_name,
                    "filePath": outcome.file_path
                }),
            )
            .ok();
            Ok(json!({ "success": true, "file": outcome.file_name }))
        }
        Err(err) => {
            let error_message = err.message.clone();
            let category = err.category;
            let failed_file = load_backup_config(&state)
                .map(|config| build_backup_filename_for_format(&config.auto_export_format))
                .unwrap_or_else(|_| build_backup_filename_for_format("encrypted_zip"));
            let _ = record_backup_run(
                &state,
                "last_manual",
                "cos",
                &failed_file,
                "failed",
                Some(&error_message),
            );
            app.emit(
                "backup-manual-done",
                json!({
                    "success": false,
                    "target": "cos",
                    "error": error_message.clone(),
                    "category": category
                }),
            )
            .ok();
            Ok(json!({
                "success": false,
                "target": "cos",
                "error": error_message,
                "category": category
            }))
        }
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
    app: AppHandle,
    _state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    let default_path = options
        .get("defaultPath")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from);

    let (tx, rx) = std::sync::mpsc::channel();
    let mut builder = app.dialog().file();
    if let Some(path) = default_path {
        builder = builder.set_directory(path);
    }
    builder.pick_folder(move |folder| {
        let _ = tx.send(folder);
    });

    let result = rx.recv().map_err(|e| format!("选择目录失败: {}", e))?;
    Ok(json!({
        "success": true,
        "directory": result.map(|path| path.to_string())
    }))
}

#[derive(Debug)]
struct ImportStats {
    total_imported: usize,
    total_skipped: usize,
    errors: Vec<String>,
}

#[derive(Debug)]
struct BackupExecutionOutcome {
    target: String,
    file_name: String,
    file_path: Option<String>,
}

fn build_backup_json_bytes(state: &State<'_, AppState>) -> Result<Vec<u8>, String> {
    let db = &state.db;
    let encryption = &state.encryption;
    let conn = db
        .get_connection()
        .map_err(|e| format!("数据库连接失败: {}", e))?;

    let mut groups_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at FROM groups ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            groups_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    let mut passwords_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, username, password, url, notes, group_id, created_at, updated_at FROM passwords ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| e.to_string())?;
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

    let mut note_groups_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, parent_id, color, sort_order, created_at, updated_at FROM secure_record_groups ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, Option<i64>>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "parent_id": row.get::<_, Option<i64>>(2)?,
                    "color": row.get::<_, Option<String>>(3)?,
                    "sort_order": row.get::<_, Option<i32>>(4)?,
                    "created_at": row.get::<_, Option<String>>(5)?,
                    "updated_at": row.get::<_, Option<String>>(6)?
                }))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            note_groups_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    let mut notes_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, group_id, pinned, archived, created_at, updated_at FROM secure_records ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| e.to_string())?;
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

    let mut settings_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, key, value, type, category, description, created_at, updated_at FROM user_settings ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            settings_arr.push(row.map_err(|e| e.to_string())?);
        }
    }

    let mut history_arr: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, password_id, old_password, changed_at, change_reason FROM password_history ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, Option<i64>>(0)?,
                    "password_id": row.get::<_, i64>(1)?,
                    "old_password": row.get::<_, String>(2)?,
                    "changed_at": row.get::<_, Option<String>>(3)?,
                    "changed_reason": row.get::<_, Option<String>>(4)?
                }))
            })
            .map_err(|e| e.to_string())?;
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

    serde_json::to_vec_pretty(&backup).map_err(|e| e.to_string())
}

fn build_encrypted_backup_bytes(
    state: &State<'_, AppState>,
    archive_password: &str,
) -> Result<Vec<u8>, String> {
    let json_bytes = build_backup_json_bytes(state)?;
    create_encrypted_zip(&json_bytes, archive_password)
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

fn read_encrypted_zip(zip_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let reader = Cursor::new(zip_bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("读取ZIP文件失败: {}", e))?;

    let mut file = archive
        .by_name_decrypt("backup.json", password.as_bytes())
        .map_err(|e| format!("ZIP解密失败（密码错误或文件损坏）: {}", e))?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| format!("读取ZIP内容失败: {}", e))?;

    Ok(contents)
}

fn do_import(
    conn: &rusqlite::Connection,
    backup: &Value,
    encryption: &EncryptionService,
) -> Result<ImportStats, String> {
    let mut stats = ImportStats {
        total_imported: 0,
        total_skipped: 0,
        errors: Vec::new(),
    };

    let mut group_id_map: HashMap<i64, i64> = HashMap::new();
    if let Some(groups) = backup.get("groups").and_then(|v| v.as_array()) {
        let (top_groups, child_groups): (Vec<&Value>, Vec<&Value>) = groups
            .iter()
            .partition(|g| g.get("parent_id").map_or(true, |v| v.is_null()));

        for group in top_groups.iter().chain(child_groups.iter()) {
            let old_id = group.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = group.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let color = group.get("color").and_then(|v| v.as_str());
            let sort_order = group
                .get("sort")
                .and_then(|v| v.as_i64())
                .or_else(|| group.get("sort_order").and_then(|v| v.as_i64()));
            let parent_id = group.get("parent_id").and_then(|v| v.as_i64());
            let mapped_parent_id = parent_id.and_then(|pid| group_id_map.get(&pid).copied());

            let existing: Option<i64> = conn
                .query_row(
                    "SELECT id FROM groups WHERE name = ?1 AND (parent_id IS ?2)",
                    rusqlite::params![name, mapped_parent_id],
                    |row| row.get(0),
                )
                .ok();

            let new_id = if let Some(eid) = existing {
                conn.execute(
                    "UPDATE groups SET color = ?1, sort_order = ?2, updated_at = datetime('now') WHERE id = ?3",
                    rusqlite::params![color, sort_order, eid],
                )
                .map_err(|e| e.to_string())?;
                eid
            } else {
                conn.execute(
                    "INSERT INTO groups (name, parent_id, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    rusqlite::params![name, mapped_parent_id, color, sort_order],
                )
                .map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            };

            group_id_map.insert(old_id, new_id);
            stats.total_imported += 1;
        }
    }

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

            let existing: Option<i64> = conn
                .query_row(
                    "SELECT id FROM passwords WHERE title = ?1 AND (username IS ?2)",
                    rusqlite::params![title, username],
                    |row| row.get(0),
                )
                .ok();

            if let Some(eid) = existing {
                conn.execute(
                    "UPDATE passwords SET password = ?1, url = ?2, notes = ?3, group_id = ?4, updated_at = datetime('now') WHERE id = ?5",
                    rusqlite::params![encrypted_pwd, url, notes, mapped_group_id, eid],
                )
                .map_err(|e| e.to_string())?;
            } else {
                conn.execute(
                    "INSERT INTO passwords (title, username, password, url, notes, group_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
                    rusqlite::params![title, username, encrypted_pwd, url, notes, mapped_group_id],
                )
                .map_err(|e| e.to_string())?;
            }
            stats.total_imported += 1;
        }
    }

    let mut note_group_id_map: HashMap<i64, i64> = HashMap::new();
    if let Some(note_groups) = backup.get("note_groups").and_then(|v| v.as_array()) {
        let (top_groups, child_groups): (Vec<&Value>, Vec<&Value>) = note_groups
            .iter()
            .partition(|g| g.get("parent_id").map_or(true, |v| v.is_null()));

        for group in top_groups.iter().chain(child_groups.iter()) {
            let old_id = group.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = group.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let color = group.get("color").and_then(|v| v.as_str());
            let sort_order = group
                .get("sort_order")
                .and_then(|v| v.as_i64())
                .or_else(|| group.get("sort").and_then(|v| v.as_i64()));
            let parent_id = group.get("parent_id").and_then(|v| v.as_i64());
            let mapped_parent_id = parent_id.and_then(|pid| note_group_id_map.get(&pid).copied());

            let existing: Option<i64> = conn
                .query_row(
                    "SELECT id FROM secure_record_groups WHERE name = ?1 AND (parent_id IS ?2)",
                    rusqlite::params![name, mapped_parent_id],
                    |row| row.get(0),
                )
                .ok();

            let new_id = if let Some(eid) = existing {
                conn.execute(
                    "UPDATE secure_record_groups SET color = ?1, sort_order = ?2, updated_at = datetime('now') WHERE id = ?3",
                    rusqlite::params![color, sort_order, eid],
                )
                .map_err(|e| e.to_string())?;
                eid
            } else {
                conn.execute(
                    "INSERT INTO secure_record_groups (name, parent_id, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    rusqlite::params![name, mapped_parent_id, color, sort_order],
                )
                .map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            };

            note_group_id_map.insert(old_id, new_id);
            stats.total_imported += 1;
        }
    }

    if let Some(notes) = backup.get("notes").and_then(|v| v.as_array()) {
        for note in notes {
            let title = note.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let plain_content = note
                .get("content_ciphertext")
                .and_then(|v| v.as_str())
                .or_else(|| note.get("content").and_then(|v| v.as_str()));
            let old_group_id = note.get("group_id").and_then(|v| v.as_i64());
            let mapped_group_id = old_group_id.and_then(|gid| note_group_id_map.get(&gid).copied());
            let pinned = note.get("pinned").and_then(|v| v.as_i64()).unwrap_or(0);
            let archived = note.get("archived").and_then(|v| v.as_i64()).unwrap_or(0);
            let encrypted_content = encrypt_field(encryption, plain_content);

            let existing: Option<i64> = conn
                .query_row(
                    "SELECT id FROM secure_records WHERE title = ?1 AND (group_id IS ?2)",
                    rusqlite::params![title, mapped_group_id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(eid) = existing {
                conn.execute(
                    "UPDATE secure_records SET content = ?1, pinned = ?2, archived = ?3, updated_at = datetime('now') WHERE id = ?4",
                    rusqlite::params![encrypted_content, pinned, archived, eid],
                )
                .map_err(|e| e.to_string())?;
            } else {
                conn.execute(
                    "INSERT INTO secure_records (title, content, group_id, pinned, archived, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
                    rusqlite::params![title, encrypted_content, mapped_group_id, pinned, archived],
                )
                .map_err(|e| e.to_string())?;
            }
            stats.total_imported += 1;
        }
    }

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
                Ok(_) => stats.total_imported += 1,
                Err(e) => {
                    stats
                        .errors
                        .push(format!("导入设置 '{}' 失败: {}", key, e));
                    stats.total_skipped += 1;
                }
            }
        }
    }

    if let Some(history) = backup.get("password_history").and_then(|v| v.as_array()) {
        if !history.is_empty() {
            stats.total_skipped += history.len();
        }
    }

    Ok(stats)
}

fn decrypt_field(encryption: &EncryptionService, cipher: &Option<String>) -> Option<String> {
    match cipher {
        Some(text) if !text.is_empty() => match encryption.decrypt(text) {
            Ok(plain) => Some(plain),
            Err(_) => Some(text.clone()),
        },
        other => other.clone(),
    }
}

fn encrypt_field(encryption: &EncryptionService, plain: Option<&str>) -> Option<String> {
    match plain {
        Some(text) if !text.is_empty() => match encryption.encrypt(text) {
            Ok(cipher) => Some(cipher),
            Err(e) => {
                log::warn!("加密失败: {}，将以明文存储", e);
                Some(text.to_string())
            }
        },
        _ => None,
    }
}

fn build_backup_filename_for_format(format: &str) -> String {
    let now = Local::now();
    let ext = if format == "json" { "json" } else { "zip" };
    format!(
        "{BACKUP_FILENAME_PREFIX}{:04}-{:02}-{:02}-{:02}-{:02}-{:02}.{}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        ext
    )
}

fn chrono_now_iso() -> String {
    Local::now().to_rfc3339()
}

fn load_backup_config(state: &State<'_, AppState>) -> Result<BackupConfig, String> {
    let db = &state.db;
    Ok(BackupConfig {
        target_mode: get_plain_setting(db, "backup.target_mode")?.unwrap_or_else(|| "local".to_string()),
        auto_export_enabled: get_plain_setting(db, "backup.auto_export_enabled")?
            .map(|v| v == "true")
            .unwrap_or(false),
        auto_export_frequency: get_plain_setting(db, "backup.auto_export_frequency")?
            .unwrap_or_else(|| "daily".to_string()),
        auto_export_directory: get_plain_setting(db, "backup.auto_export_directory")?
            .unwrap_or_default(),
        auto_export_format: get_plain_setting(db, "backup.auto_export_format")?
            .unwrap_or_else(|| "json".to_string()),
        auto_export_password: get_sensitive_setting(state, "backup.auto_export_password")?,
        auto_export_time_of_day: get_plain_setting(db, "backup.auto_export_time_of_day")?
            .unwrap_or_else(|| "02:00".to_string()),
        auto_export_day_of_week: get_plain_setting(db, "backup.auto_export_day_of_week")?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(1),
        auto_export_day_of_month: get_plain_setting(db, "backup.auto_export_day_of_month")?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(1),
        auto_export_interval_minutes: get_plain_setting(db, "backup.auto_export_interval_minutes")?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(60),
        retention_count: get_plain_setting(db, "backup.retention_count")?
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(30)
            .max(1),
        cloud_provider: get_plain_setting(db, "backup.cloud.provider")?
            .unwrap_or_else(|| CLOUD_PROVIDER.to_string()),
        cloud_endpoint: get_plain_setting(db, "backup.cloud.endpoint")?.unwrap_or_default(),
        cloud_bucket: get_plain_setting(db, "backup.cloud.bucket")?.unwrap_or_default(),
        cloud_region: get_plain_setting(db, "backup.cloud.region")?.unwrap_or_default(),
        cloud_path_prefix: get_plain_setting(db, "backup.cloud.path_prefix")?.unwrap_or_default(),
        cloud_secret_id: get_sensitive_setting(state, "backup.cloud.secret_id")?,
        cloud_secret_key: get_sensitive_setting(state, "backup.cloud.secret_key")?,
    })
}

fn get_plain_setting(
    db: &crate::services::database::DatabaseService,
    key: &str,
) -> Result<Option<String>, String> {
    db.get_user_setting(key)
        .map_err(|e| e.to_string())
        .map(|opt| opt.map(|s| s.value))
}

fn get_sensitive_setting(state: &State<'_, AppState>, key: &str) -> Result<Option<String>, String> {
    let value = get_plain_setting(&state.db, key)?;
    match value {
        Some(cipher) if !cipher.trim().is_empty() => state
            .encryption
            .decrypt(&cipher)
            .map(Some)
            .map_err(|e| format!("解密配置失败({}): {}", key, e)),
        _ => Ok(None),
    }
}

fn save_plain_setting(
    state: &State<'_, AppState>,
    key: &str,
    value: String,
    type_: &str,
    category: &str,
    description: &str,
) -> Result<(), String> {
    state
        .db
        .set_user_setting(&crate::models::setting::UserSetting {
            id: None,
            key: key.to_string(),
            value,
            r#type: Some(type_.to_string()),
            category: Some(category.to_string()),
            description: Some(description.to_string()),
            created_at: None,
            updated_at: None,
        })
        .map_err(|e| e.to_string())
}

fn save_sensitive_setting(
    state: &State<'_, AppState>,
    key: &str,
    value: &str,
    type_: &str,
    category: &str,
    description: &str,
) -> Result<(), String> {
    let cipher = state.encryption.encrypt(value)?;
    save_plain_setting(state, key, cipher, type_, category, description)
}

fn get_backup_run_status(state: &State<'_, AppState>, prefix: &str) -> BackupRunStatus {
    let db = &state.db;
    BackupRunStatus {
        at: get_plain_setting(db, &format!("backup.status.{}_at", prefix)).ok().flatten(),
        result: get_plain_setting(db, &format!("backup.status.{}_result", prefix)).ok().flatten(),
        target: get_plain_setting(db, &format!("backup.status.{}_target", prefix)).ok().flatten(),
        file: get_plain_setting(db, &format!("backup.status.{}_file", prefix)).ok().flatten(),
        error: get_plain_setting(db, &format!("backup.status.{}_error", prefix)).ok().flatten(),
    }
}

fn record_backup_run(
    state: &State<'_, AppState>,
    prefix: &str,
    target: &str,
    file: &str,
    result: &str,
    error: Option<&str>,
) -> Result<(), String> {
    save_plain_setting(
        state,
        &format!("backup.status.{}_at", prefix),
        chrono_now_iso(),
        "string",
        "backup",
        "最近备份时间",
    )?;
    save_plain_setting(
        state,
        &format!("backup.status.{}_result", prefix),
        result.to_string(),
        "string",
        "backup",
        "最近备份结果",
    )?;
    save_plain_setting(
        state,
        &format!("backup.status.{}_target", prefix),
        target.to_string(),
        "string",
        "backup",
        "最近备份目标",
    )?;
    save_plain_setting(
        state,
        &format!("backup.status.{}_file", prefix),
        file.to_string(),
        "string",
        "backup",
        "最近备份文件名",
    )?;
    save_plain_setting(
        state,
        &format!("backup.status.{}_error", prefix),
        error.unwrap_or("").to_string(),
        "string",
        "backup",
        "最近备份错误",
    )?;
    Ok(())
}

fn mask_secret_id(secret_id: &str) -> String {
    let chars: Vec<char> = secret_id.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }
    let start: String = chars.iter().take(4).collect();
    let end: String = chars.iter().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{}****{}", start, end)
}

fn normalize_path_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}/", trimmed)
    }
}

fn normalize_endpoint(endpoint: &str) -> Result<String, String> {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("请输入 Endpoint".to_string());
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("Endpoint 必须以 http:// 或 https:// 开头".to_string());
    }
    Ok(trimmed.to_string())
}

fn cloud_config_from_backup(config: &BackupConfig) -> Result<CloudConfig, String> {
    let endpoint = normalize_endpoint(&config.cloud_endpoint)?;
    let cloud = CloudConfig {
        endpoint,
        bucket: config.cloud_bucket.clone(),
        region: config.cloud_region.clone(),
        path_prefix: normalize_path_prefix(&config.cloud_path_prefix),
        secret_id: config
            .cloud_secret_id
            .clone()
            .ok_or("请先配置 SecretId")?,
        secret_key: config
            .cloud_secret_key
            .clone()
            .ok_or("请先配置 SecretKey")?,
    };
    validate_cloud_config(&cloud)?;
    Ok(cloud)
}

fn validate_cloud_config(config: &CloudConfig) -> Result<(), String> {
    if config.bucket.trim().is_empty() {
        return Err("请先配置 Bucket".to_string());
    }
    if config.region.trim().is_empty() {
        return Err("请先配置 Region".to_string());
    }
    if config.secret_id.trim().is_empty() {
        return Err("请先配置 SecretId".to_string());
    }
    if config.secret_key.trim().is_empty() {
        return Err("请先配置 SecretKey".to_string());
    }
    Ok(())
}

async fn resolve_test_cloud_config(
    state: &State<'_, AppState>,
    input: &TestBackupCloudInput,
) -> Result<CloudConfig, String> {
    let base = load_backup_config(state)?;
    let endpoint = if input.endpoint.trim().is_empty() {
        base.cloud_endpoint.clone()
    } else {
        normalize_endpoint(&input.endpoint)?
    };
    let bucket = if input.bucket.trim().is_empty() {
        base.cloud_bucket.clone()
    } else {
        input.bucket.trim().to_string()
    };
    let region = if input.region.trim().is_empty() {
        base.cloud_region.clone()
    } else {
        input.region.trim().to_string()
    };
    let path_prefix = input
        .path_prefix
        .as_deref()
        .map(normalize_path_prefix)
        .unwrap_or_else(|| base.cloud_path_prefix.clone());
    let secret_id = input
        .secret_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| base.cloud_secret_id.clone())
        .ok_or("请先配置 SecretId 或在输入框中填写")?;
    let secret_key = input
        .secret_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| base.cloud_secret_key.clone())
        .ok_or("请先配置 SecretKey 或在输入框中填写")?;

    let config = CloudConfig {
        endpoint,
        bucket,
        region,
        path_prefix,
        secret_id,
        secret_key,
    };

    validate_cloud_config(&config)?;
    Ok(config)
}

fn resolve_test_archive_password(
    state: &State<'_, AppState>,
    input: &TestBackupCloudInput,
) -> Result<String, String> {
    let base = load_backup_config(state)?;
    let password = input
        .export_default_password
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or(base.auto_export_password)
        .ok_or("请先设置加密ZIP默认密码（至少4位）")?;
    if password.trim().len() < 4 {
        return Err("加密ZIP默认密码至少需要 4 位".to_string());
    }
    Ok(password)
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

async fn upload_backup_bytes(
    client: &Client,
    config: &CloudConfig,
    filename: &str,
    bytes: &[u8],
    retention_count: usize,
) -> Result<(), CloudError> {
    let key = format!("{}{}", config.path_prefix, filename);
    put_object(client, config, &key, bytes).await?;
    cleanup_cloud_backups(client, config, retention_count).await?;
    Ok(())
}

async fn cleanup_cloud_backups(
    client: &Client,
    config: &CloudConfig,
    retention_count: usize,
) -> Result<(), CloudError> {
    let objects = list_backup_objects(client, config).await?;
    if objects.len() <= retention_count {
        return Ok(());
    }

    for object in objects.iter().take(objects.len() - retention_count) {
        delete_object(client, config, &object.key).await?;
    }
    Ok(())
}

async fn list_backup_objects(client: &Client, config: &CloudConfig) -> Result<Vec<CloudObject>, CloudError> {
    let query = vec![
        ("list-type".to_string(), "2".to_string()),
        ("prefix".to_string(), config.path_prefix.clone()),
    ];
    let url = bucket_base_url(config);
    let response = signed_request(
        client,
        Method::GET,
        &url,
        query,
        None,
        config,
        Some("application/xml"),
    )
    .await?;
    let body = response
        .text()
        .await
        .map_err(|e| CloudError::new("network_failure", format!("读取对象列表失败: {}", e)))?;

    let mut objects = extract_xml_tags(&body, "Key")
        .into_iter()
        .filter(|key| is_managed_backup_key(key))
        .map(|key| CloudObject { key })
        .collect::<Vec<_>>();
    objects.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(objects)
}

async fn put_object(client: &Client, config: &CloudConfig, key: &str, body: &[u8]) -> Result<(), CloudError> {
    let url = object_url(config, key);
    let response = signed_request(
        client,
        Method::PUT,
        &url,
        Vec::new(),
        Some(body),
        config,
        Some("application/octet-stream"),
    )
    .await?;
    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(cloud_error_from_response(status, response.text().await.unwrap_or_default()))
    }
}

async fn delete_object(client: &Client, config: &CloudConfig, key: &str) -> Result<(), CloudError> {
    let url = object_url(config, key);
    let response = signed_request(client, Method::DELETE, &url, Vec::new(), None, config, None).await?;
    let status = response.status();
    if status.is_success() || status == StatusCode::NO_CONTENT {
        Ok(())
    } else {
        Err(cloud_error_from_response(status, response.text().await.unwrap_or_default()))
    }
}

async fn signed_request(
    client: &Client,
    method: Method,
    base_url: &str,
    query_pairs: Vec<(String, String)>,
    body: Option<&[u8]>,
    config: &CloudConfig,
    content_type: Option<&str>,
) -> Result<reqwest::Response, CloudError> {
    let endpoint_url = reqwest::Url::parse(base_url)
        .map_err(|e| CloudError::new("invalid_endpoint", format!("无效 Endpoint: {}", e)))?;
    let host = endpoint_url
        .host_str()
        .ok_or_else(|| CloudError::new("invalid_endpoint", "Endpoint 缺少 host"))?;
    let canonical_uri = if endpoint_url.path().is_empty() {
        "/".to_string()
    } else {
        utf8_percent_encode(endpoint_url.path(), URI_ENCODE_SET).to_string()
    };

    let query_string = canonical_query_string(&query_pairs);
    let amz_date = aws_amz_datetime();
    let short_date = &amz_date[..8];
    let payload = body.unwrap_or(&[]);
    let payload_hash = sha256_hex(payload);

    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_str(host).map_err(|e| CloudError::new("invalid_endpoint", e.to_string()))?);
    headers.insert(
        HeaderName::from_static("x-amz-content-sha256"),
        HeaderValue::from_str(&payload_hash).map_err(|e| CloudError::new("unknown_cloud_error", e.to_string()))?,
    );
    headers.insert(
        HeaderName::from_static("x-amz-date"),
        HeaderValue::from_str(&amz_date).map_err(|e| CloudError::new("unknown_cloud_error", e.to_string()))?,
    );
    if let Some(content_type) = content_type {
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type).map_err(|e| CloudError::new("unknown_cloud_error", e.to_string()))?,
        );
    }

    let canonical_headers = canonical_headers(&headers)?;
    let signed_headers = signed_headers(&headers);
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method.as_str(),
        canonical_uri,
        query_string,
        canonical_headers,
        signed_headers,
        payload_hash
    );
    let credential_scope = format!("{}/{}/{}/aws4_request", short_date, config.region, AWS_SERVICE_NAME);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        sha256_hex(canonical_request.as_bytes())
    );
    let signing_key = build_signing_key(&config.secret_key, short_date, &config.region, AWS_SERVICE_NAME);
    let signature = hex::encode(hmac_sign(&signing_key, &string_to_sign));
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        config.secret_id, credential_scope, signed_headers, signature
    );
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&authorization).map_err(|e| CloudError::new("unknown_cloud_error", e.to_string()))?,
    );

    let mut request = client.request(method, endpoint_url);
    if !query_pairs.is_empty() {
        request = request.query(&query_pairs);
    }
    request = request.headers(headers);
    if let Some(body) = body {
        request = request.body(body.to_vec());
    }
    request
        .send()
        .await
        .map_err(|e| CloudError::new("network_failure", format!("请求对象存储失败: {}", e)))
}

fn canonical_query_string(query_pairs: &[(String, String)]) -> String {
    let mut pairs = query_pairs
        .iter()
        .map(|(k, v)| {
            (
                utf8_percent_encode(k, QUERY_ENCODE_SET).to_string(),
                utf8_percent_encode(v, QUERY_ENCODE_SET).to_string(),
            )
        })
        .collect::<Vec<_>>();
    pairs.sort();
    pairs
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

fn canonical_headers(headers: &HeaderMap) -> Result<String, CloudError> {
    let mut pairs = headers
        .iter()
        .map(|(name, value)| {
            Ok((
                name.as_str().to_ascii_lowercase(),
                value
                    .to_str()
                    .map_err(|e| CloudError::new("unknown_cloud_error", e.to_string()))?
                    .trim()
                    .to_string(),
            ))
        })
        .collect::<Result<Vec<_>, CloudError>>()?;
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(pairs
        .into_iter()
        .map(|(k, v)| format!("{}:{}\n", k, v))
        .collect::<String>())
}

fn signed_headers(headers: &HeaderMap) -> String {
    let mut names = headers
        .keys()
        .map(|name| name.as_str().to_ascii_lowercase())
        .collect::<Vec<_>>();
    names.sort();
    names.join(";")
}

fn build_signing_key(secret: &str, short_date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sign(format!("AWS4{}", secret).as_bytes(), short_date);
    let k_region = hmac_sign(&k_date, region);
    let k_service = hmac_sign(&k_region, service);
    hmac_sign(&k_service, "aws4_request")
}

fn hmac_sign(key: &[u8], message: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length is valid");
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hex::encode(hasher.finalize())
}

fn aws_amz_datetime() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn bucket_base_url(config: &CloudConfig) -> String {
    if endpoint_contains_bucket(config) {
        config.endpoint.clone()
    } else {
        format!("{}/{}", config.endpoint, config.bucket)
    }
}

fn object_url(config: &CloudConfig, key: &str) -> String {
    let encoded_key = utf8_percent_encode(key, URI_ENCODE_SET).to_string();
    if endpoint_contains_bucket(config) {
        format!("{}/{}", config.endpoint, encoded_key)
    } else {
        format!("{}/{}/{}", config.endpoint, config.bucket, encoded_key)
    }
}

fn endpoint_contains_bucket(config: &CloudConfig) -> bool {
    let bucket = config.bucket.trim();
    if bucket.is_empty() {
        return false;
    }

    let Ok(url) = reqwest::Url::parse(&config.endpoint) else {
        return false;
    };

    let bucket_lower = bucket.to_ascii_lowercase();
    if let Some(host) = url.host_str() {
        let host_lower = host.to_ascii_lowercase();
        if host_lower == bucket_lower || host_lower.starts_with(&format!("{}.", bucket_lower)) {
            return true;
        }
    }

    if let Some(first_segment) = url
        .path_segments()
        .and_then(|mut segments| segments.find(|s| !s.is_empty()))
    {
        return first_segment.eq_ignore_ascii_case(bucket);
    }

    false
}

fn is_managed_backup_key(key: &str) -> bool {
    let file_name = key.rsplit('/').next().unwrap_or(key);
    file_name.starts_with(BACKUP_FILENAME_PREFIX)
        && (file_name.ends_with(".zip") || file_name.ends_with(".json"))
}

fn extract_xml_tags(xml: &str, tag: &str) -> Vec<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);
    let mut values = Vec::new();
    let mut start = 0;
    while let Some(open_idx) = xml[start..].find(&open_tag) {
        let content_start = start + open_idx + open_tag.len();
        if let Some(close_idx) = xml[content_start..].find(&close_tag) {
            let content_end = content_start + close_idx;
            values.push(xml[content_start..content_end].to_string());
            start = content_end + close_tag.len();
        } else {
            break;
        }
    }
    values
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    extract_xml_tags(xml, tag).into_iter().next()
}

fn cloud_error_from_response(status: StatusCode, body: String) -> CloudError {
    let code = extract_xml_tag(&body, "Code").unwrap_or_default();
    let message = extract_xml_tag(&body, "Message").unwrap_or_default();

    match code.as_str() {
        "InvalidAccessKeyId" => CloudError::new("invalid_ak", "AK 无效，请检查 SecretId"),
        "SignatureDoesNotMatch" => CloudError::new("invalid_sk", "SK 无效，请检查 SecretKey"),
        "NoSuchBucket" => CloudError::new("invalid_bucket", "Bucket 不存在或不可访问"),
        "AuthorizationHeaderMalformed" | "InvalidRegionName" => {
            CloudError::new("invalid_region_or_endpoint", "Region 或 Endpoint 配置错误")
        }
        "AccessDenied" => CloudError::new("permission_denied", "当前凭证没有写入权限"),
        _ if status == StatusCode::FORBIDDEN => {
            CloudError::new("permission_denied", "当前凭证没有写入权限")
        }
        _ if status == StatusCode::NOT_FOUND => {
            CloudError::new("invalid_bucket", "Bucket 不存在或 Endpoint 不可访问")
        }
        _ if status == StatusCode::UNAUTHORIZED => {
            CloudError::new("invalid_ak", "认证失败，请检查 AK/SK")
        }
        _ => {
            let fallback = if message.is_empty() {
                format!("对象存储请求失败: HTTP {}", status)
            } else {
                format!("对象存储请求失败: {}", message)
            };
            CloudError::new("unknown_cloud_error", fallback)
        }
    }
}

async fn execute_backup_run(
    app: &AppHandle,
    state: &State<'_, AppState>,
    run_kind: &str,
    target: &str,
) -> Result<BackupExecutionOutcome, CloudError> {
    let config = load_backup_config(state).map_err(|e| CloudError::new("config_error", e))?;
    let target = if target == "auto" {
        config.target_mode.as_str()
    } else {
        target
    };
    let file_name = build_backup_filename_for_format(&config.auto_export_format);
    let prefix = if run_kind == "manual" { "last_manual" } else { "last_auto" };

    let outcome = match target {
        "local" => execute_local_backup(state, &config, &file_name)
            .map_err(|e| CloudError::new("local_backup_failed", e))?,
        "cos" => execute_cloud_backup(state, &config, &file_name).await?,
        _ => return Err(CloudError::new("config_error", "不支持的备份目标")),
    };

    record_backup_run(state, prefix, &outcome.target, &outcome.file_name, "success", None)
        .map_err(|e| CloudError::new("status_error", e))?;

    if run_kind == "auto" {
        app.emit(
            "auto-export-done",
            json!({
                "success": true,
                "target": outcome.target,
                "filePath": outcome.file_path,
                "file": outcome.file_name
            }),
        )
        .ok();
    }

    Ok(outcome)
}

fn execute_local_backup(
    state: &State<'_, AppState>,
    config: &BackupConfig,
    file_name: &str,
) -> Result<BackupExecutionOutcome, String> {
    let directory = config.auto_export_directory.trim();
    if directory.is_empty() {
        return Err("请先配置自动导出目录".to_string());
    }

    let path = PathBuf::from(directory);
    std::fs::create_dir_all(&path).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let bytes = if config.auto_export_format == "json" {
        build_backup_json_bytes(state)?
    } else {
        let password = config
            .auto_export_password
            .as_ref()
            .ok_or("请先设置加密ZIP默认密码，再执行本地备份")?;
        if password.trim().len() < 4 {
            return Err("加密ZIP默认密码至少需要 4 位".to_string());
        }
        build_encrypted_backup_bytes(state, password)?
    };

    let full_path = path.join(file_name);
    std::fs::write(&full_path, bytes).map_err(|e| format!("写入本地备份失败: {}", e))?;
    cleanup_local_backups(&path, config.retention_count)?;

    Ok(BackupExecutionOutcome {
        target: "local".to_string(),
        file_name: file_name.to_string(),
        file_path: Some(full_path.to_string_lossy().to_string()),
    })
}

async fn execute_cloud_backup(
    state: &State<'_, AppState>,
    config: &BackupConfig,
    file_name: &str,
) -> Result<BackupExecutionOutcome, CloudError> {
    let cloud = cloud_config_from_backup(config).map_err(|e| CloudError::new("config_error", e))?;
    let password = config
        .auto_export_password
        .clone()
        .ok_or_else(|| CloudError::new("config_error", "请先设置加密ZIP默认密码，再执行云备份"))?;
    if password.trim().len() < 4 {
        return Err(CloudError::new(
            "config_error",
            "加密ZIP默认密码至少需要 4 位",
        ));
    }

    let bytes = build_encrypted_backup_bytes(state, &password)
        .map_err(|e| CloudError::new("backup_generation_failed", e))?;
    let client = build_http_client().map_err(|e| CloudError::new("network_failure", e))?;
    upload_backup_bytes(&client, &cloud, file_name, &bytes, config.retention_count).await?;

    Ok(BackupExecutionOutcome {
        target: "cos".to_string(),
        file_name: file_name.to_string(),
        file_path: Some(format!("{}{}", cloud.path_prefix, file_name)),
    })
}

fn cleanup_local_backups(directory: &PathBuf, retention_count: usize) -> Result<(), String> {
    let mut files = std::fs::read_dir(directory)
        .map_err(|e| format!("读取备份目录失败: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(is_managed_backup_key)
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    files.sort();
    if files.len() <= retention_count {
        return Ok(());
    }

    let delete_count = files.len() - retention_count;
    for path in files.into_iter().take(delete_count) {
        std::fs::remove_file(&path).map_err(|e| format!("清理旧备份失败: {}", e))?;
    }
    Ok(())
}

pub fn start_backup_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(err) = maybe_run_scheduled_backup(&app).await {
                if should_notify_backup_failure(&app, &err.category, &err.message) {
                    app.emit(
                        "auto-export-done",
                        json!({
                            "success": false,
                            "error": err.message,
                            "category": err.category
                        }),
                    )
                    .ok();
                }
            }
        }
    });
}

async fn maybe_run_scheduled_backup(app: &AppHandle) -> Result<(), CloudError> {
    let state = app.state::<AppState>();
    let config = load_backup_config(&state).map_err(|e| CloudError::new("config_error", e))?;
    if !config.auto_export_enabled {
        return Ok(());
    }
    let last_auto = get_backup_run_status(&state, "last_auto");
    if !is_schedule_due(&config, last_auto.at.as_deref()) {
        return Ok(());
    }

    match execute_backup_run(app, &state, "auto", "auto").await {
        Ok(_) => Ok(()),
        Err(err) => {
            let filename = build_backup_filename_for_format(&config.auto_export_format);
            record_backup_run(
                &state,
                "last_auto",
                &config.target_mode,
                &filename,
                "failed",
                Some(&err.message),
            )
            .map_err(|e| CloudError::new("status_error", e))?;
            Err(err)
        }
    }
}

fn should_notify_backup_failure(app: &AppHandle, category: &str, message: &str) -> bool {
    let cooldown = Duration::from_secs(BACKUP_FAILURE_NOTIFY_COOLDOWN_SECS);
    let state = app.state::<AppState>();
    let mut guard = state.backup_notification.lock().map_err(|_| ()).unwrap();
    let now = Instant::now();
    if !should_emit_failure_notification(guard.as_ref(), category, message, now, cooldown) {
        return false;
    }
    *guard = Some((category.to_string(), message.to_string(), now));
    true
}

fn is_schedule_due(config: &BackupConfig, last_auto_at: Option<&str>) -> bool {
    is_schedule_due_at(config, last_auto_at, Local::now())
}

fn is_schedule_due_at(
    config: &BackupConfig,
    last_auto_at: Option<&str>,
    now: chrono::DateTime<Local>,
) -> bool {
    let last = last_auto_at.and_then(parse_rfc3339_to_local);

    match config.auto_export_frequency.as_str() {
        "every_minute" => {
            let interval = config.auto_export_interval_minutes.max(1);
            match last {
                Some(last) => now.signed_duration_since(last).num_minutes() >= interval,
                None => true,
            }
        }
        "daily" => is_time_slot_due(
            now,
            last,
            config.auto_export_time_of_day.as_str(),
            |_| true,
        ),
        "weekly" => {
            let target_weekday = config.auto_export_day_of_week.clamp(1, 7) as u32;
            is_time_slot_due(now, last, config.auto_export_time_of_day.as_str(), |dt| {
                dt.weekday().number_from_monday() == target_weekday
            })
        }
        "monthly" => {
            let target_day = config.auto_export_day_of_month.clamp(1, 31) as u32;
            let last_day = last_day_of_month(now.year(), now.month());
            let effective_day = target_day.min(last_day);
            is_time_slot_due(now, last, config.auto_export_time_of_day.as_str(), |dt| {
                dt.day() == effective_day
            })
        }
        _ => false,
    }
}

fn should_emit_failure_notification(
    previous: Option<&(String, String, Instant)>,
    category: &str,
    message: &str,
    now: Instant,
    cooldown: Duration,
) -> bool {
    if let Some((prev_category, prev_message, prev_time)) = previous {
        if prev_category == category && prev_message == message && now.duration_since(*prev_time) < cooldown {
            return false;
        }
    }
    true
}

fn is_time_slot_due<F>(
    now: chrono::DateTime<Local>,
    last: Option<chrono::DateTime<Local>>,
    hhmm: &str,
    matches_day: F,
) -> bool
where
    F: Fn(chrono::DateTime<Local>) -> bool,
{
    let Some((hour, minute)) = parse_hhmm(hhmm) else {
        return false;
    };
    if !matches_day(now) || now.hour() != hour || now.minute() != minute {
        return false;
    }
    match last {
        Some(last) => last.date_naive() != now.date_naive() || last.hour() != hour || last.minute() != minute,
        None => true,
    }
}

fn parse_hhmm(value: &str) -> Option<(u32, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse::<u32>().ok()?;
    let minute = parts.next()?.parse::<u32>().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some((hour, minute))
}

fn parse_rfc3339_to_local(value: &str) -> Option<chrono::DateTime<Local>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|dt| dt.with_timezone(&Local).into())
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_of_next = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("valid month");
    let last_of_this = first_of_next - chrono::Duration::days(1);
    last_of_this.day()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::tempdir;

    #[test]
    fn test_build_backup_filename_format() {
        let filename = build_backup_filename_for_format("encrypted_zip");
        assert!(filename.starts_with(BACKUP_FILENAME_PREFIX));
        assert!(filename.ends_with(".zip"));
        assert_eq!(filename.len(), "myloair-backup-2026-03-24-10-30-00.zip".len());
    }

    #[test]
    fn test_mask_secret_id() {
        assert_eq!(mask_secret_id("AKID12345678ABCD"), "AKID****ABCD");
        assert_eq!(mask_secret_id("short"), "****");
    }

    #[test]
    fn test_normalize_path_prefix() {
        assert_eq!(normalize_path_prefix(""), "");
        assert_eq!(normalize_path_prefix(" backups/test "), "backups/test/");
        assert_eq!(normalize_path_prefix("/backups/test/"), "backups/test/");
    }

    #[test]
    fn test_managed_backup_key_filter() {
        assert!(is_managed_backup_key("backups/myloair-backup-2026-03-24-10-30-00.zip"));
        assert!(!is_managed_backup_key("backups/random-file.zip"));
    }

    #[test]
    fn test_object_url_path_style_endpoint() {
        let cloud = CloudConfig {
            endpoint: "https://cos.ap-shanghai.myqcloud.com".to_string(),
            bucket: "myloair-1318175726".to_string(),
            region: "ap-shanghai".to_string(),
            path_prefix: "backups/qjs/".to_string(),
            secret_id: "ak".to_string(),
            secret_key: "sk".to_string(),
        };
        let key = "backups/qjs/myloair-backup-2026-03-24-10-30-00.zip";
        assert_eq!(
            bucket_base_url(&cloud),
            "https://cos.ap-shanghai.myqcloud.com/myloair-1318175726"
        );
        assert_eq!(
            object_url(&cloud, key),
            "https://cos.ap-shanghai.myqcloud.com/myloair-1318175726/backups/qjs/myloair-backup-2026-03-24-10-30-00.zip"
        );
    }

    #[test]
    fn test_object_url_virtual_host_style_endpoint() {
        let cloud = CloudConfig {
            endpoint: "https://myloair-1318175726.cos.ap-shanghai.myqcloud.com".to_string(),
            bucket: "myloair-1318175726".to_string(),
            region: "ap-shanghai".to_string(),
            path_prefix: "backups/qjs/".to_string(),
            secret_id: "ak".to_string(),
            secret_key: "sk".to_string(),
        };
        let key = "backups/qjs/myloair-backup-2026-03-24-10-30-00.zip";
        assert_eq!(
            bucket_base_url(&cloud),
            "https://myloair-1318175726.cos.ap-shanghai.myqcloud.com"
        );
        assert_eq!(
            object_url(&cloud, key),
            "https://myloair-1318175726.cos.ap-shanghai.myqcloud.com/backups/qjs/myloair-backup-2026-03-24-10-30-00.zip"
        );
    }

    #[test]
    fn test_should_emit_failure_notification_with_cooldown() {
        let now = Instant::now();
        let previous = ("permission_denied".to_string(), "AK 无效".to_string(), now);
        let cooldown = Duration::from_secs(300);
        assert!(!should_emit_failure_notification(
            Some(&previous),
            "permission_denied",
            "AK 无效",
            now + Duration::from_secs(120),
            cooldown
        ));
        assert!(should_emit_failure_notification(
            Some(&previous),
            "permission_denied",
            "AK 无效",
            now + Duration::from_secs(301),
            cooldown
        ));
        assert!(should_emit_failure_notification(
            Some(&previous),
            "network_failure",
            "网络失败",
            now + Duration::from_secs(120),
            cooldown
        ));
    }

    #[test]
    fn test_schedule_due_every_minute() {
        let config = test_backup_config("every_minute");
        let now = local_dt(2026, 5, 10, 10, 30, 0);
        assert!(is_schedule_due_at(&config, None, now));

        let last = local_dt(2026, 5, 10, 10, 29, 30).to_rfc3339();
        assert!(!is_schedule_due_at(&config, Some(&last), now));

        let last_due = local_dt(2026, 5, 10, 9, 30, 0).to_rfc3339();
        assert!(is_schedule_due_at(&config, Some(&last_due), now));
    }

    #[test]
    fn test_schedule_due_daily_and_restart_scenario() {
        let mut config = test_backup_config("daily");
        config.auto_export_time_of_day = "10:30".to_string();
        let now = local_dt(2026, 5, 10, 10, 30, 0);
        assert!(is_schedule_due_at(&config, None, now));

        // Simulate app restart with persisted last run at the same daily slot.
        let last_same_slot = local_dt(2026, 5, 10, 10, 30, 0).to_rfc3339();
        assert!(!is_schedule_due_at(&config, Some(&last_same_slot), now));
    }

    #[test]
    fn test_schedule_due_weekly_and_monthly() {
        let mut weekly = test_backup_config("weekly");
        weekly.auto_export_day_of_week = 7; // Sunday
        weekly.auto_export_time_of_day = "09:00".to_string();
        let sunday = local_dt(2026, 5, 10, 9, 0, 0);
        let monday = local_dt(2026, 5, 11, 9, 0, 0);
        assert!(is_schedule_due_at(&weekly, None, sunday));
        assert!(!is_schedule_due_at(&weekly, None, monday));

        let mut monthly = test_backup_config("monthly");
        monthly.auto_export_day_of_month = 31;
        monthly.auto_export_time_of_day = "18:00".to_string();
        // April has 30 days, should run on the last day.
        let april_30 = local_dt(2026, 4, 30, 18, 0, 0);
        assert!(is_schedule_due_at(&monthly, None, april_30));
    }

    #[test]
    fn test_cleanup_local_backups_keeps_unmanaged_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        let managed_old = path.join("myloair-backup-2026-03-24-10-00-00.zip");
        let managed_mid = path.join("myloair-backup-2026-03-24-10-10-00.zip");
        let managed_new = path.join("myloair-backup-2026-03-24-10-20-00.zip");
        let unmanaged = path.join("notes.txt");

        std::fs::write(&managed_old, b"old").unwrap();
        std::fs::write(&managed_mid, b"mid").unwrap();
        std::fs::write(&managed_new, b"new").unwrap();
        std::fs::write(&unmanaged, b"keep").unwrap();

        cleanup_local_backups(&path, 2).unwrap();

        assert!(!managed_old.exists());
        assert!(managed_mid.exists());
        assert!(managed_new.exists());
        assert!(unmanaged.exists());
    }

    fn test_backup_config(frequency: &str) -> BackupConfig {
        BackupConfig {
            target_mode: "local".to_string(),
            auto_export_enabled: true,
            auto_export_frequency: frequency.to_string(),
            auto_export_directory: String::new(),
            auto_export_format: "encrypted_zip".to_string(),
            auto_export_password: Some("1234".to_string()),
            auto_export_time_of_day: "00:00".to_string(),
            auto_export_day_of_week: 1,
            auto_export_day_of_month: 1,
            auto_export_interval_minutes: 60,
            retention_count: 30,
            cloud_provider: CLOUD_PROVIDER.to_string(),
            cloud_endpoint: String::new(),
            cloud_bucket: String::new(),
            cloud_region: String::new(),
            cloud_path_prefix: String::new(),
            cloud_secret_id: None,
            cloud_secret_key: None,
        }
    }

    fn local_dt(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> chrono::DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
    }
}
