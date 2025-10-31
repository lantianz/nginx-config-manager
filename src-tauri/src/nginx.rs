use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use encoding_rs::GBK;

/// 在 Windows 下将 GBK 编码的字节转换为 UTF-8 字符串
#[cfg(target_os = "windows")]
fn decode_gbk(bytes: &[u8]) -> String {
    let (cow, _, _) = GBK.decode(bytes);
    cow.into_owned()
}

/// 在非 Windows 系统下直接使用 UTF-8
#[cfg(not(target_os = "windows"))]
fn decode_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// Windows 下使用 GBK 解码
#[cfg(target_os = "windows")]
fn decode_output(bytes: &[u8]) -> String {
    decode_gbk(bytes)
}

/// 智能解析 nginx 路径，支持文件路径和目录路径两种方式
/// 返回 (nginx_exe_path, working_dir)
#[cfg(target_os = "windows")]
fn parse_nginx_path(nginx_path: &str) -> (String, String) {
    let path = Path::new(nginx_path);

    // 如果路径指向一个文件（例如 D:\nginx\nginx.exe）
    if path.is_file() || nginx_path.to_lowercase().ends_with(".exe") {
        let dir = path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| nginx_path.to_string());
        (nginx_path.to_string(), dir)
    } else {
        // 如果路径是目录（例如 D:\nginx）
        let exe_path = path.join("nginx.exe").to_string_lossy().to_string();
        (exe_path, nginx_path.to_string())
    }
}

/// 非 Windows 系统的路径解析
#[cfg(not(target_os = "windows"))]
fn parse_nginx_path(nginx_path: &str) -> (String, String) {
    let path = Path::new(nginx_path);

    // 如果路径指向一个文件
    if path.is_file() || !nginx_path.ends_with('/') {
        let dir = path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| nginx_path.to_string());
        (nginx_path.to_string(), dir)
    } else {
        // 如果路径是目录
        let exe_path = path.join("nginx").to_string_lossy().to_string();
        (exe_path, nginx_path.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NginxStatus {
    pub is_running: bool,
    pub process_count: u32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
}

/// 检查 Nginx 是否正在运行
pub fn is_nginx_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .args(["/C", "tasklist | findstr /i nginx.exe"])
            .output();

        if let Ok(output) = output {
            output.status.success()
        } else {
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("sh")
            .args(["-c", "pgrep -x nginx"])
            .output();

        if let Ok(output) = output {
            output.status.success()
        } else {
            false
        }
    }
}

/// 获取 Nginx 进程数量
pub fn get_nginx_process_count() -> u32 {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .args(["/C", "tasklist | findstr /i nginx.exe"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = decode_output(&output.stdout);
                return stdout.lines().count() as u32;
            }
        }
        0
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("sh")
            .args(["-c", "pgrep -x nginx | wc -l"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = decode_output(&output.stdout);
                return stdout.trim().parse().unwrap_or(0);
            }
        }
        0
    }
}

/// 检查 Nginx 状态
#[tauri::command]
pub fn check_nginx_status() -> Result<NginxStatus, String> {
    let is_running = is_nginx_running();
    let process_count = if is_running {
        get_nginx_process_count()
    } else {
        0
    };

    let message = if is_running {
        format!("Nginx 正在运行 ({} 个进程)", process_count)
    } else {
        "Nginx 未运行".to_string()
    };

    Ok(NginxStatus {
        is_running,
        process_count,
        message,
    })
}

/// 启动 Nginx
#[tauri::command]
pub fn start_nginx(nginx_path: String) -> Result<OperationResult, String> {
    if nginx_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "请先设置 Nginx 路径".to_string(),
        });
    }

    if is_nginx_running() {
        return Ok(OperationResult {
            success: false,
            message: "Nginx 已经在运行中".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let (nginx_exe, working_dir) = parse_nginx_path(&nginx_path);

        // 先测试配置是否正确
        let test_output = Command::new(&nginx_exe)
            .args(["-t"])
            .current_dir(&working_dir)
            .output();

        match test_output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                if !output.status.success() {
                    return Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置测试失败，无法启动:\n{}", stderr),
                    });
                }
            }
            Err(e) => {
                return Ok(OperationResult {
                    success: false,
                    message: format!("✗ 无法执行 nginx.exe: {}\n请检查路径是否正确", e),
                });
            }
        }

        // 启动 Nginx
        let start_result = Command::new(&nginx_exe)
            .current_dir(&working_dir)
            .spawn();

        match start_result {
            Ok(_) => {
                // 等待 2 秒让 Nginx 启动
                thread::sleep(Duration::from_secs(2));

                if is_nginx_running() {
                    Ok(OperationResult {
                        success: true,
                        message: "✓ Nginx 启动成功".to_string(),
                    })
                } else {
                    Ok(OperationResult {
                        success: false,
                        message: "✗ Nginx 启动失败，进程未检测到\n请检查 Nginx 错误日志".to_string(),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 启动失败: {}", e),
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let (nginx_exe, _working_dir) = parse_nginx_path(&nginx_path);

        // 先测试配置
        let test_output = Command::new(&nginx_exe)
            .args(["-t"])
            .output();

        match test_output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                if !output.status.success() {
                    return Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置测试失败，无法启动:\n{}", stderr),
                    });
                }
            }
            Err(e) => {
                return Ok(OperationResult {
                    success: false,
                    message: format!("✗ 无法执行 nginx: {}\n请检查路径是否正确", e),
                });
            }
        }

        let output = Command::new(&nginx_exe).spawn();

        match output {
            Ok(_) => {
                thread::sleep(Duration::from_secs(2));

                if is_nginx_running() {
                    Ok(OperationResult {
                        success: true,
                        message: "✓ Nginx 启动成功".to_string(),
                    })
                } else {
                    Ok(OperationResult {
                        success: false,
                        message: "✗ Nginx 启动失败，进程未检测到\n请检查 Nginx 错误日志".to_string(),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 启动失败: {}", e),
            }),
        }
    }
}

/// 停止 Nginx
#[tauri::command]
pub fn stop_nginx() -> Result<OperationResult, String> {
    if !is_nginx_running() {
        return Ok(OperationResult {
            success: false,
            message: "Nginx 未运行".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .args(["/C", "taskkill /F /IM nginx.exe"])
            .output();

        match output {
            Ok(_) => {
                thread::sleep(Duration::from_secs(1));

                if !is_nginx_running() {
                    Ok(OperationResult {
                        success: true,
                        message: "✓ Nginx 停止成功".to_string(),
                    })
                } else {
                    Ok(OperationResult {
                        success: false,
                        message: "✗ Nginx 停止失败".to_string(),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("停止失败: {}", e),
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("sh")
            .args(["-c", "pkill -9 nginx"])
            .output();

        match output {
            Ok(_) => {
                thread::sleep(Duration::from_secs(1));

                if !is_nginx_running() {
                    Ok(OperationResult {
                        success: true,
                        message: "✓ Nginx 停止成功".to_string(),
                    })
                } else {
                    Ok(OperationResult {
                        success: false,
                        message: "✗ Nginx 停止失败".to_string(),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("停止失败: {}", e),
            }),
        }
    }
}

/// 重启 Nginx
#[tauri::command]
pub fn restart_nginx(nginx_path: String) -> Result<OperationResult, String> {
    if nginx_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "请先设置 Nginx 路径".to_string(),
        });
    }

    // 先停止
    if is_nginx_running() {
        let stop_result = stop_nginx()?;
        if !stop_result.success {
            return Ok(stop_result);
        }
    }

    // 再启动
    start_nginx(nginx_path)
}

/// 重新加载配置
#[tauri::command]
pub fn reload_nginx(nginx_path: String) -> Result<OperationResult, String> {
    if nginx_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "请先设置 Nginx 路径".to_string(),
        });
    }

    if !is_nginx_running() {
        return Ok(OperationResult {
            success: false,
            message: "Nginx 未运行，无法重新加载配置".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let (nginx_exe, working_dir) = parse_nginx_path(&nginx_path);
        let output = Command::new(&nginx_exe)
            .args(["-s", "reload"])
            .current_dir(&working_dir)
            .output();

        match output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                let stdout = decode_output(&output.stdout);

                if output.status.success() {
                    let msg = if !stderr.is_empty() {
                        format!("✓ 配置重新加载成功\n{}", stderr)
                    } else {
                        "✓ 配置重新加载成功".to_string()
                    };
                    Ok(OperationResult {
                        success: true,
                        message: msg,
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置重新加载失败:\n{}", error_msg),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 重新加载失败: {}", e),
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let (nginx_exe, _working_dir) = parse_nginx_path(&nginx_path);
        let output = Command::new(&nginx_exe)
            .args(["-s", "reload"])
            .output();

        match output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                let stdout = decode_output(&output.stdout);

                if output.status.success() {
                    let msg = if !stderr.is_empty() {
                        format!("✓ 配置重新加载成功\n{}", stderr)
                    } else {
                        "✓ 配置重新加载成功".to_string()
                    };
                    Ok(OperationResult {
                        success: true,
                        message: msg,
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置重新加载失败:\n{}", error_msg),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 重新加载失败: {}", e),
            }),
        }
    }
}

/// 测试 Nginx 配置
#[tauri::command]
pub fn test_nginx_config(nginx_path: String) -> Result<OperationResult, String> {
    if nginx_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "请先设置 Nginx 路径".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let (nginx_exe, working_dir) = parse_nginx_path(&nginx_path);
        let output = Command::new(&nginx_exe)
            .args(["-t"])
            .current_dir(&working_dir)
            .output();

        match output {
            Ok(output) => {
                // nginx -t 的输出在 stderr 中
                let stderr = decode_output(&output.stderr);
                let stdout = decode_output(&output.stdout);

                if output.status.success() {
                    Ok(OperationResult {
                        success: true,
                        message: format!("✓ 配置测试通过\n\n{}", stderr),
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置测试失败\n\n{}", error_msg),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 无法执行测试: {}\n请检查 Nginx 路径是否正确", e),
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let (nginx_exe, _working_dir) = parse_nginx_path(&nginx_path);
        let output = Command::new(&nginx_exe)
            .args(["-t"])
            .output();

        match output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                let stdout = decode_output(&output.stdout);

                if output.status.success() {
                    Ok(OperationResult {
                        success: true,
                        message: format!("✓ 配置测试通过\n\n{}", stderr),
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置测试失败\n\n{}", error_msg),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 无法执行测试: {}\n请检查 Nginx 路径是否正确", e),
            }),
        }
    }
}

