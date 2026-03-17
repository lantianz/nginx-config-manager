use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[cfg(target_os = "windows")]
use encoding_rs::GBK;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows 创建进程标志：不创建控制台窗口
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub is_windows: bool,
    pub is_elevated: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortProcessInfo {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub pid: u32,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub command_line: Option<String>,
    pub user: Option<String>,
    pub status: String,
    pub start_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInspectionResult {
    pub port: u16,
    pub is_occupied: bool,
    pub entries: Vec<PortProcessInfo>,
    pub permission_status: PermissionStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessOperationResult {
    pub success: bool,
    pub message: String,
    pub requires_elevation: bool,
}

#[cfg(target_os = "windows")]
fn build_powershell_command(script: &str) -> Command {
    let mut command = Command::new("powershell");
    command
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ]);
    command
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<String, String> {
    let output = build_powershell_command(script)
        .output()
        .map_err(|e| format!("执行 PowerShell 失败: {}", e))?;

    if output.status.success() {
        Ok(decode_output(&output.stdout).trim().to_string())
    } else {
        let stderr = decode_output(&output.stderr);
        let stdout = decode_output(&output.stdout);
        let message = if !stderr.trim().is_empty() {
            stderr
        } else {
            stdout
        };
        Err(message.trim().to_string())
    }
}

fn build_permission_message(is_windows: bool, is_elevated: bool) -> String {
    if !is_windows {
        return "当前平台暂未实现端口占用管理能力".to_string();
    }

    if is_elevated {
        "当前以管理员身份运行，可直接执行端口释放操作".to_string()
    } else {
        "当前未以管理员身份运行，查询端口通常可用，但结束受保护进程时可能失败，请按管理员身份重新启动应用".to_string()
    }
}

#[cfg(target_os = "windows")]
fn current_permission_status() -> PermissionStatus {
    let script = "([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)";
    let is_elevated = run_powershell(script)
        .map(|output| output.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    PermissionStatus {
        is_windows: true,
        is_elevated,
        message: build_permission_message(true, is_elevated),
    }
}

#[cfg(not(target_os = "windows"))]
fn current_permission_status() -> PermissionStatus {
    PermissionStatus {
        is_windows: false,
        is_elevated: true,
        message: build_permission_message(false, true),
    }
}

fn deserialize_json_vec<T>(text: &str) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_str(text)
        .map_err(|e| format!("解析系统返回数据失败: {}", e))?;

    match value {
        Value::Array(items) => items
            .into_iter()
            .map(|item| serde_json::from_value(item).map_err(|e| e.to_string()))
            .collect(),
        Value::Object(_) => Ok(vec![
            serde_json::from_value(value).map_err(|e| e.to_string())?,
        ]),
        _ => Ok(Vec::new()),
    }
}

#[cfg(target_os = "windows")]
fn inspect_ports_windows(ports: &[u16]) -> Result<Vec<PortProcessInfo>, String> {
    let port_csv = ports
        .iter()
        .map(|port| port.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let script = format!(
        r#"
$ports = @({port_csv})
$connections = Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object {{ $ports.Count -eq 0 -or $ports -contains [int]$_.LocalPort }}
$results = foreach ($conn in $connections) {{
    $proc = Get-Process -Id $conn.OwningProcess -ErrorAction SilentlyContinue
    $cim = Get-CimInstance Win32_Process -Filter "ProcessId = $($conn.OwningProcess)" -ErrorAction SilentlyContinue
    $owner = $null
    if ($cim) {{
        try {{
            $ownerInfo = Invoke-CimMethod -InputObject $cim -MethodName GetOwner -ErrorAction Stop
            if ($ownerInfo.ReturnValue -eq 0) {{
                if ($ownerInfo.Domain) {{
                    $owner = "$($ownerInfo.Domain)\$($ownerInfo.User)"
                }} else {{
                    $owner = $ownerInfo.User
                }}
            }}
        }} catch {{}}
    }}

    $path = $null
    if ($proc) {{
        try {{
            $path = $proc.Path
        }} catch {{}}
    }}
    if (-not $path -and $cim) {{
        $path = $cim.ExecutablePath
    }}

    $startTime = $null
    if ($proc) {{
        try {{
            $startTime = $proc.StartTime.ToString('yyyy-MM-dd HH:mm:ss')
        }} catch {{}}
    }}

    [PSCustomObject]@{{
        protocol = 'TCP'
        localAddress = [string]$conn.LocalAddress
        localPort = [int]$conn.LocalPort
        pid = [int]$conn.OwningProcess
        processName = if ($proc) {{ [string]$proc.ProcessName }} elseif ($cim) {{ [string]$cim.Name }} else {{ '' }}
        executablePath = $path
        commandLine = if ($cim) {{ $cim.CommandLine }} else {{ $null }}
        user = $owner
        status = [string]$conn.State
        startTime = $startTime
    }}
}}

@($results) | ConvertTo-Json -Compress -Depth 6
"#
    );

    let output = run_powershell(&script)?;
    let mut entries: Vec<PortProcessInfo> = deserialize_json_vec(&output)?;
    entries.sort_by_key(|item| (item.local_port, item.pid));
    Ok(entries)
}

fn build_port_inspection_results(
    ports: &[u16],
    entries: Vec<PortProcessInfo>,
    permission_status: PermissionStatus,
) -> Vec<PortInspectionResult> {
    let mut entry_map: HashMap<u16, Vec<PortProcessInfo>> = HashMap::new();
    for entry in entries {
        entry_map.entry(entry.local_port).or_default().push(entry);
    }

    ports
        .iter()
        .map(|port| {
            let port_entries = entry_map.remove(port).unwrap_or_default();
            let is_occupied = !port_entries.is_empty();
            let base_message = if is_occupied {
                format!("端口 {} 当前被 {} 个监听进程占用", port, port_entries.len())
            } else {
                format!("端口 {} 当前空闲", port)
            };
            let message = if permission_status.is_elevated {
                base_message
            } else {
                format!("{}；{}", base_message, permission_status.message)
            };

            PortInspectionResult {
                port: *port,
                is_occupied,
                entries: port_entries,
                permission_status: permission_status.clone(),
                message,
            }
        })
        .collect()
}

fn permission_denied_message(action: &str) -> String {
    format!(
        "{}失败：权限不足。请以管理员身份重新启动应用后再试。",
        action
    )
}

fn is_permission_denied_output(text: &str) -> bool {
    let lowered = text.to_lowercase();
    lowered.contains("access is denied")
        || text.contains("拒绝访问")
        || text.contains("权限不足")
}

fn is_process_not_found_output(text: &str) -> bool {
    let lowered = text.to_lowercase();
    lowered.contains("not found")
        || lowered.contains("no running instance")
        || text.contains("没有运行的任务")
        || text.contains("找不到进程")
        || text.contains("找不到")
}

#[cfg(target_os = "windows")]
fn terminate_process_internal(pid: u32) -> ProcessOperationResult {
    let output = Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["/C", &format!("taskkill /F /PID {}", pid)])
        .output();

    match output {
        Ok(output) => {
            let stderr = decode_output(&output.stderr);
            let stdout = decode_output(&output.stdout);
            let raw_message = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else {
                stdout.trim().to_string()
            };

            if output.status.success() {
                ProcessOperationResult {
                    success: true,
                    message: format!("已结束 PID {} 对应的进程", pid),
                    requires_elevation: false,
                }
            } else if is_permission_denied_output(&raw_message) {
                ProcessOperationResult {
                    success: false,
                    message: permission_denied_message(&format!("结束 PID {}", pid)),
                    requires_elevation: true,
                }
            } else if is_process_not_found_output(&raw_message) {
                ProcessOperationResult {
                    success: false,
                    message: format!("PID {} 对应的进程不存在或已退出", pid),
                    requires_elevation: false,
                }
            } else {
                ProcessOperationResult {
                    success: false,
                    message: if raw_message.is_empty() {
                        format!("结束 PID {} 失败", pid)
                    } else {
                        format!("结束 PID {} 失败：{}", pid, raw_message)
                    },
                    requires_elevation: false,
                }
            }
        }
        Err(e) => ProcessOperationResult {
            success: false,
            message: format!("执行结束进程命令失败: {}", e),
            requires_elevation: false,
        },
    }
}

#[cfg(not(target_os = "windows"))]
fn terminate_process_internal(pid: u32) -> ProcessOperationResult {
    ProcessOperationResult {
        success: false,
        message: format!("当前平台暂未实现结束 PID {} 的能力", pid),
        requires_elevation: false,
    }
}

/// 检查 Nginx 是否正在运行
pub fn is_nginx_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
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
            .creation_flags(CREATE_NO_WINDOW)
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
pub async fn check_nginx_status() -> Result<NginxStatus, String> {
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

#[tauri::command]
pub async fn check_process_permission_status() -> Result<PermissionStatus, String> {
    Ok(current_permission_status())
}

#[tauri::command]
pub async fn inspect_ports(ports: Vec<u16>) -> Result<Vec<PortInspectionResult>, String> {
    if ports.is_empty() {
        return Err("请至少提供一个端口".to_string());
    }

    let unique_ports: Vec<u16> = ports.into_iter().collect::<BTreeSet<u16>>().into_iter().collect();
    let permission_status = current_permission_status();

    #[cfg(target_os = "windows")]
    {
        let entries = inspect_ports_windows(&unique_ports)?;
        Ok(build_port_inspection_results(&unique_ports, entries, permission_status))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = permission_status;
        let _ = unique_ports;
        Err("当前平台暂未实现端口占用查询".to_string())
    }
}

#[tauri::command]
pub async fn terminate_process(pid: u32) -> Result<ProcessOperationResult, String> {
    Ok(terminate_process_internal(pid))
}

#[tauri::command]
pub async fn release_port(port: u16) -> Result<ProcessOperationResult, String> {
    let inspections = inspect_ports(vec![port]).await?;
    let port_result = inspections.into_iter().next().ok_or_else(|| "未获取到端口查询结果".to_string())?;

    if !port_result.is_occupied {
        return Ok(ProcessOperationResult {
            success: false,
            message: format!("端口 {} 当前未被占用", port),
            requires_elevation: false,
        });
    }

    let pids: Vec<u32> = port_result
        .entries
        .iter()
        .map(|entry| entry.pid)
        .collect::<BTreeSet<u32>>()
        .into_iter()
        .collect();

    let mut failed_messages = Vec::new();
    let mut requires_elevation = false;

    for pid in &pids {
        let result = terminate_process_internal(*pid);
        if !result.success {
            requires_elevation |= result.requires_elevation;
            failed_messages.push(result.message);
        }
    }

    if failed_messages.is_empty() {
        Ok(ProcessOperationResult {
            success: true,
            message: format!("已释放端口 {}，共结束 {} 个进程", port, pids.len()),
            requires_elevation: false,
        })
    } else {
        Ok(ProcessOperationResult {
            success: false,
            message: failed_messages.join("\n"),
            requires_elevation,
        })
    }
}

/// 启动 Nginx
#[tauri::command]
pub async fn start_nginx(nginx_path: String) -> Result<OperationResult, String> {
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

        // 先校验配置是否正确
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
                        message: format!("✗ 配置校验失败，无法启动:\n{}", stderr),
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
                sleep(Duration::from_secs(2)).await;

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

        // 先校验配置
        let test_output = Command::new(&nginx_exe)
            .args(["-t"])
            .output();

        match test_output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                if !output.status.success() {
                    return Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置校验失败，无法启动:\n{}", stderr),
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
                sleep(Duration::from_secs(2)).await;

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
pub async fn stop_nginx() -> Result<OperationResult, String> {
    if !is_nginx_running() {
        return Ok(OperationResult {
            success: false,
            message: "Nginx 未运行".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/C", "taskkill /F /IM nginx.exe"])
            .output();

        match output {
            Ok(_) => {
                sleep(Duration::from_secs(1)).await;

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
                sleep(Duration::from_secs(1)).await;

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
pub async fn restart_nginx(nginx_path: String) -> Result<OperationResult, String> {
    if nginx_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "请先设置 Nginx 路径".to_string(),
        });
    }

    // 先停止
    if is_nginx_running() {
        let stop_result = stop_nginx().await?;
        if !stop_result.success {
            return Ok(stop_result);
        }
    }

    // 再启动
    start_nginx(nginx_path).await
}

/// 重新加载配置
#[tauri::command]
pub async fn reload_nginx(nginx_path: String) -> Result<OperationResult, String> {
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
pub async fn test_nginx_config(nginx_path: String) -> Result<OperationResult, String> {
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
                        message: format!("✓ 配置校验通过\n\n{}", stderr),
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置校验失败\n\n{}", error_msg),
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
                        message: format!("✓ 配置校验通过\n\n{}", stderr),
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置校验失败\n\n{}", error_msg),
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

/// 测试指定配置文件的 Nginx 配置
#[tauri::command]
pub async fn test_nginx_config_file(nginx_path: String, config_path: String) -> Result<OperationResult, String> {
    if nginx_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "请先设置 Nginx 路径".to_string(),
        });
    }

    if config_path.is_empty() {
        return Ok(OperationResult {
            success: false,
            message: "配置文件路径不能为空".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let (nginx_exe, working_dir) = parse_nginx_path(&nginx_path);
        let output = Command::new(&nginx_exe)
            .creation_flags(CREATE_NO_WINDOW)
            .args(["-t", "-c", &config_path])
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
                        message: format!("✓ 配置校验通过\n\n{}", stderr),
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置校验失败\n\n{}", error_msg),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 无法执行校验: {}\n请检查 Nginx 路径是否正确", e),
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let (nginx_exe, _working_dir) = parse_nginx_path(&nginx_path);
        let output = Command::new(&nginx_exe)
            .args(["-t", "-c", &config_path])
            .output();

        match output {
            Ok(output) => {
                let stderr = decode_output(&output.stderr);
                let stdout = decode_output(&output.stdout);

                if output.status.success() {
                    Ok(OperationResult {
                        success: true,
                        message: format!("✓ 配置校验通过\n\n{}", stderr),
                    })
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr
                    } else {
                        stdout
                    };
                    Ok(OperationResult {
                        success: false,
                        message: format!("✗ 配置校验失败\n\n{}", error_msg),
                    })
                }
            }
            Err(e) => Ok(OperationResult {
                success: false,
                message: format!("✗ 无法执行校验: {}\n请检查 Nginx 路径是否正确", e),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_port_inspection_results_should_group_entries_by_port() {
        let permission = PermissionStatus {
            is_windows: true,
            is_elevated: false,
            message: "need admin".to_string(),
        };
        let entries = vec![
            PortProcessInfo {
                protocol: "TCP".to_string(),
                local_address: "0.0.0.0".to_string(),
                local_port: 80,
                pid: 1000,
                process_name: "nginx".to_string(),
                executable_path: None,
                command_line: None,
                user: None,
                status: "Listen".to_string(),
                start_time: None,
            },
            PortProcessInfo {
                protocol: "TCP".to_string(),
                local_address: "127.0.0.1".to_string(),
                local_port: 8080,
                pid: 2000,
                process_name: "node".to_string(),
                executable_path: None,
                command_line: None,
                user: None,
                status: "Listen".to_string(),
                start_time: None,
            },
        ];

        let results = build_port_inspection_results(&[80, 443, 8080], entries, permission);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_occupied);
        assert!(!results[1].is_occupied);
        assert_eq!(results[2].entries[0].pid, 2000);
    }
}
