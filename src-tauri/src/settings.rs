use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub nginx_path: String,
    pub config_path: String,
    pub theme: String,
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            nginx_path: String::new(),
            config_path: String::new(),
            theme: "dark".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

/// 获取配置文件路径
fn get_settings_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "无法获取配置目录".to_string())?;
    
    let app_config_dir = config_dir.join("nginx-config-manager");
    
    // 确保目录存在
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    Ok(app_config_dir.join("settings.json"))
}

/// 保存应用设置
#[tauri::command]
pub fn save_app_settings(settings: AppSettings) -> Result<bool, String> {
    let settings_path = get_settings_path()?;
    
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    
    fs::write(&settings_path, json)
        .map_err(|e| format!("写入设置文件失败: {}", e))?;
    
    Ok(true)
}

/// 加载应用设置
#[tauri::command]
pub fn load_app_settings() -> Result<AppSettings, String> {
    let settings_path = get_settings_path()?;
    
    if !settings_path.exists() {
        // 如果配置文件不存在,返回默认设置
        return Ok(AppSettings::default());
    }
    
    let json = fs::read_to_string(&settings_path)
        .map_err(|e| format!("读取设置文件失败: {}", e))?;
    
    let settings: AppSettings = serde_json::from_str(&json)
        .map_err(|e| format!("解析设置文件失败: {}", e))?;
    
    Ok(settings)
}

