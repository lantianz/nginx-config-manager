use std::path::Path;

/// 使用系统默认程序打开文件
#[tauri::command]
pub async fn open_file_in_system(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    
    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }
    
    // 使用系统默认程序打开文件
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        match Command::new("cmd")
            .args(&["/C", "start", "", &file_path])
            .spawn()
        {
            Ok(_) => Ok("文件已打开".to_string()),
            Err(e) => Err(format!("打开文件失败: {}", e)),
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        match Command::new("open")
            .arg(&file_path)
            .spawn()
        {
            Ok(_) => Ok("文件已打开".to_string()),
            Err(e) => Err(format!("打开文件失败: {}", e)),
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        match Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
        {
            Ok(_) => Ok("文件已打开".to_string()),
            Err(e) => Err(format!("打开文件失败: {}", e)),
        }
    }
}

