//! MyloAir - 密码管理器 Tauri 后端
//!
//! 模块结构：
//! - commands: Tauri Command 处理器
//! - services: 业务逻辑服务
//! - models: 数据模型定义

pub mod commands;
pub mod models;
pub mod services;

use tauri::Manager;
use services::database::DatabaseService;
use services::encryption::EncryptionService;
use std::sync::Mutex;

/// 应用共享状态
pub struct AppState {
    pub db: DatabaseService,
    pub encryption: EncryptionService,  // 应用级加密服务（始终可用）
    pub ui_locked: Mutex<bool>,         // UI 锁定状态
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // 获取应用数据目录
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // 确保目录存在
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir
                .join("myloair.db")
                .to_string_lossy()
                .to_string();

            log::info!("Database path: {}", db_path);

            // 初始化数据库
            let db_service = DatabaseService::new(&db_path);
            if let Err(e) = db_service.initialize() {
                log::error!("Failed to initialize database: {}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database initialization failed: {}", e),
                )));
            }

            // 初始化应用级加密服务（始终可用，用于数据加密）
            let encryption_service = EncryptionService::new_with_app_key();
            log::info!("Encryption service initialized with app-level key");

            // 管理应用状态
            app.manage(AppState { 
                db: db_service, 
                encryption: encryption_service,
                ui_locked: Mutex::new(true),  // 默认锁定 UI
            });

            // 显示窗口
            if let Some(window) = app.get_webview_window("main") {
                window.show().ok();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 密码管理
            commands::passwords::get_passwords,
            commands::passwords::get_password,
            commands::passwords::add_password,
            commands::passwords::update_password,
            commands::passwords::delete_password,
            commands::passwords::search_passwords,
            commands::passwords::generate_password,
            commands::passwords::get_password_history,
            // 分组管理
            commands::groups::get_groups,
            commands::groups::get_group_tree,
            commands::groups::add_group,
            commands::groups::update_group,
            commands::groups::delete_group,
            // 窗口管理
            commands::window::minimize_window,
            commands::window::toggle_maximize_window,
            commands::window::close_window,
            commands::window::open_external,
            // 安全管理
            commands::security::security_get_state,
            commands::security::security_set_master_password,
            commands::security::security_verify_master_password,
            commands::security::security_update_master_password,
            commands::security::security_clear_master_password,
            commands::security::security_set_require_master_password,
            commands::security::security_lock_ui,
            commands::security::security_get_ui_lock_state,
            // 笔记管理
            commands::notes::get_note_groups,
            commands::notes::get_note_group_tree,
            commands::notes::get_note_group,
            commands::notes::add_note_group,
            commands::notes::update_note_group,
            commands::notes::delete_note_group,
            commands::notes::get_notes,
            commands::notes::get_note,
            commands::notes::add_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::search_notes_title,
            // 设置管理
            commands::settings::get_user_settings,
            commands::settings::get_user_setting,
            commands::settings::set_user_setting,
            commands::settings::update_user_setting,
            commands::settings::delete_user_setting,
            commands::settings::get_user_settings_categories,
            // 备份导出
            commands::backup::export_data,
            commands::backup::export_data_to_file,
            commands::backup::import_data,
            commands::backup::pick_export_path,
            commands::backup::pick_export_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
