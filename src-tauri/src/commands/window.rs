//! 窗口管理 Commands

use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

/// 最小化窗口
#[tauri::command]
pub async fn minimize_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 切换最大化状态
#[tauri::command]
pub async fn toggle_maximize_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            window.unmaximize().map_err(|e| e.to_string())?;
        } else {
            window.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 关闭窗口
/// macOS 上隐藏而非关闭，其他平台直接关闭
#[tauri::command]
pub async fn close_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "macos")]
        {
            window.hide().map_err(|e| e.to_string())?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            window.close().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 打开外部链接
#[tauri::command]
pub async fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    if url.is_empty() {
        return Err("URL 不能为空".to_string());
    }
    
    // 使用 shell plugin 打开外部链接
    app.shell()
        .open(&url, None)
        .map_err(|e| e.to_string())
}
