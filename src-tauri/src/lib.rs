mod commands;
mod ssh;
mod types;

use std::sync::Arc;
use commands::mod_commands::AppState;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, MouseButton, MouseButtonState},
    Manager,
};

/// 获取配置文件目录路径
#[tauri::command]
fn get_config_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::mod_commands::test_connection,
            commands::mod_commands::start_forward_cmd,
            commands::mod_commands::stop_forward_cmd,
            commands::mod_commands::get_forward_status,
            commands::mod_commands::get_all_statuses,
            commands::mod_commands::import_ssh_config,
            get_config_dir,
        ])
        .setup(|app| {
            // 获取应用句柄
            let app_handle = app.handle().clone();
            
            // 创建菜单项
            let show_window_i = MenuItem::with_id(
                &app_handle,
                "show_window",
                "显示主窗口",
                true,
                None::<&str>,
            )?;
            
            let quit_i = MenuItem::with_id(
                &app_handle,
                "quit",
                "退出",
                true,
                None::<&str>,
            )?;
            
            let separator = PredefinedMenuItem::separator(&app_handle)?;
            
            // 创建托盘菜单
            let menu = Menu::with_items(
                &app_handle,
                &[&show_window_i, &separator, &quit_i],
            )?;
            
            // 构建托盘图标
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false) // 左键点击不显示菜单，而是切换窗口显示
                .on_tray_icon_event(|tray, event| {
                    let app_handle = tray.app_handle();
                    
                    match event {
                        // 左键点击 - 切换窗口显示/隐藏
                        tauri::tray::TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .on_menu_event(|app_handle, event| {
                    match event.id().as_ref() {
                        "show_window" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(&app_handle)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            // 拦截窗口关闭事件，改为隐藏窗口
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
