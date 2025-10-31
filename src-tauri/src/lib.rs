mod nginx;
mod settings;
mod config;
mod file_ops;

use nginx::{check_nginx_status, reload_nginx, restart_nginx, start_nginx, stop_nginx, test_nginx_config};
use settings::{load_app_settings, save_app_settings};
use config::{
    read_config_file,
    add_server_block,
    update_server_block,
    delete_server_block,
    add_location_to_server,
    add_server_block_text,
    update_server_block_text,
};
use file_ops::open_file_in_system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            start_nginx,
            stop_nginx,
            restart_nginx,
            reload_nginx,
            check_nginx_status,
            test_nginx_config,
            save_app_settings,
            load_app_settings,
            read_config_file,
            add_server_block,
            update_server_block,
            delete_server_block,
            add_location_to_server,
            add_server_block_text,
            update_server_block_text,
            open_file_in_system,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
