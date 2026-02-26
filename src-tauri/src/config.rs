use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 配置指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub value: String,
    pub line: usize,
}

/// Location 配置块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationBlock {
    pub id: String,
    pub path: String,
    pub modifier: Option<String>, // =, ~, ~*, ^~
    pub directives: Vec<Directive>,
    pub raw_content: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Server 配置块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBlock {
    pub id: String,
    pub listen: Vec<String>,
    pub server_name: Vec<String>,
    pub locations: Vec<LocationBlock>,
    pub directives: Vec<Directive>,
    pub raw_content: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Nginx 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NginxConfig {
    pub servers: Vec<ServerBlock>,
    pub global_directives: Vec<Directive>,
    pub file_path: String,
    pub raw_content: String,
}

/// 配置解析结果
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub success: bool,
    pub message: String,
    pub config: Option<NginxConfig>,
}

/// 读取配置文件
#[tauri::command]
pub async fn read_config_file(config_path: String) -> Result<ParseResult, String> {
    if config_path.is_empty() {
        return Ok(ParseResult {
            success: false,
            message: "配置文件路径不能为空".to_string(),
            config: None,
        });
    }

    let path = Path::new(&config_path);
    if !path.exists() {
        return Ok(ParseResult {
            success: false,
            message: format!("配置文件不存在: {}", config_path),
            config: None,
        });
    }

    match fs::read_to_string(path) {
        Ok(content) => {
            // 解析配置文件
            match parse_nginx_config(&content, &config_path) {
                Ok(config) => Ok(ParseResult {
                    success: true,
                    message: "配置文件读取成功".to_string(),
                    config: Some(config),
                }),
                Err(e) => Ok(ParseResult {
                    success: false,
                    message: format!("配置文件解析失败: {}", e),
                    config: None,
                }),
            }
        }
        Err(e) => Ok(ParseResult {
            success: false,
            message: format!("读取配置文件失败: {}", e),
            config: None,
        }),
    }
}

/// 解析 Nginx 配置文件
fn parse_nginx_config(content: &str, file_path: &str) -> Result<NginxConfig, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut servers = Vec::new();
    let mut global_directives = Vec::new();
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        
        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }
        
        // 检测 server 块
        if line.starts_with("server") && line.contains('{') {
            match parse_server_block(&lines, i) {
                Ok((server, end_line)) => {
                    servers.push(server);
                    i = end_line + 1;
                }
                Err(e) => {
                    return Err(format!("解析 server 块失败 (行 {}): {}", i + 1, e));
                }
            }
        } else {
            // 全局指令
            if let Some(directive) = parse_directive(line, i + 1) {
                global_directives.push(directive);
            }
            i += 1;
        }
    }
    
    Ok(NginxConfig {
        servers,
        global_directives,
        file_path: file_path.to_string(),
        raw_content: content.to_string(),
    })
}

/// 解析 server 块
fn parse_server_block(lines: &[&str], start: usize) -> Result<(ServerBlock, usize), String> {
    let mut listen = Vec::new();
    let mut server_name = Vec::new();
    let mut locations = Vec::new();
    let mut directives = Vec::new();
    
    // 查找 server 块的结束位置
    let end = find_block_end(lines, start)?;
    
    // 提取原始内容
    let raw_content = lines[start..=end].join("\n");
    
    // 解析 server 块内容
    let mut i = start + 1;
    while i < end {
        let line = lines[i].trim();
        
        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }
        
        // 检测 location 块
        if line.starts_with("location") && line.contains('{') {
            match parse_location_block(lines, i) {
                Ok((location, end_line)) => {
                    locations.push(location);
                    i = end_line + 1;
                }
                Err(e) => {
                    return Err(format!("解析 location 块失败 (行 {}): {}", i + 1, e));
                }
            }
        } else {
            // 解析指令
            if let Some(directive) = parse_directive(line, i + 1) {
                // 特殊处理 listen 和 server_name
                if directive.name == "listen" {
                    listen.push(directive.value.clone());
                } else if directive.name == "server_name" {
                    // server_name 可能有多个值
                    server_name.extend(
                        directive.value
                            .split_whitespace()
                            .map(|s| s.to_string())
                    );
                }
                directives.push(directive);
            }
            i += 1;
        }
    }
    
    // 生成唯一 ID
    let id = format!("server_{}", start);

    Ok((
        ServerBlock {
            id,
            listen,
            server_name,
            locations,
            directives,
            raw_content,
            start_line: start + 1,
            end_line: end + 1,
        },
        end,
    ))
}

/// 解析 location 块
fn parse_location_block(lines: &[&str], start: usize) -> Result<(LocationBlock, usize), String> {
    let line = lines[start].trim();
    
    // 解析 location 路径和修饰符
    // 格式: location [modifier] path {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let (modifier, path) = if parts.len() >= 3 {
        // 有修饰符: location ~ /api {
        let mod_str = parts[1];
        let path_str = parts[2].trim_end_matches('{').trim();
        (Some(mod_str.to_string()), path_str.to_string())
    } else if parts.len() >= 2 {
        // 无修饰符: location /api {
        let path_str = parts[1].trim_end_matches('{').trim();
        (None, path_str.to_string())
    } else {
        return Err("无效的 location 语法".to_string());
    };
    
    // 查找 location 块的结束位置
    let end = find_block_end(lines, start)?;
    
    // 提取原始内容
    let raw_content = lines[start..=end].join("\n");
    
    // 解析 location 块内的指令
    let mut directives = Vec::new();
    for i in (start + 1)..end {
        let line = lines[i].trim();
        if !line.is_empty() && !line.starts_with('#') {
            if let Some(directive) = parse_directive(line, i + 1) {
                directives.push(directive);
            }
        }
    }
    
    // 生成唯一 ID
    let id = format!("location_{}_{}", start, path.replace('/', "_"));
    
    Ok((
        LocationBlock {
            id,
            path,
            modifier,
            directives,
            raw_content,
            start_line: start + 1,
            end_line: end + 1,
        },
        end,
    ))
}

/// 解析单个指令
fn parse_directive(line: &str, line_number: usize) -> Option<Directive> {
    // 先去除注释（# 后面的内容）
    let line_without_comment = if let Some(pos) = line.find('#') {
        &line[..pos]
    } else {
        line
    };

    // 去除分号和空格
    let line = line_without_comment.trim_end_matches(';').trim();

    if line.is_empty() || line.starts_with('#') || line.ends_with('{') || line == "}" {
        return None;
    }

    // 分割指令名和值
    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    if parts.len() >= 2 {
        Some(Directive {
            name: parts[0].to_string(),
            value: parts[1].trim().to_string(),
            line: line_number,
        })
    } else if parts.len() == 1 {
        Some(Directive {
            name: parts[0].to_string(),
            value: String::new(),
            line: line_number,
        })
    } else {
        None
    }
}

/// 查找块的结束位置（匹配的右花括号）
fn find_block_end(lines: &[&str], start: usize) -> Result<usize, String> {
    let mut depth = 0;
    let mut found_start = false;
    
    for i in start..lines.len() {
        let line = lines[i].trim();
        
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                found_start = true;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 && found_start {
                    return Ok(i);
                }
            }
        }
    }
    
    Err("未找到匹配的右花括号".to_string())
}

// ==================== 配置编辑功能 ====================

/// 新增/编辑 Server 块的输入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBlockInput {
    pub listen: Vec<String>,
    pub server_name: Vec<String>,
    pub directives: Vec<DirectiveInput>,
}

/// 新增/编辑 Location 块的输入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationBlockInput {
    pub path: String,
    pub modifier: Option<String>,
    pub directives: Vec<DirectiveInput>,
}

/// 指令输入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveInput {
    pub name: String,
    pub value: String,
}

/// 编辑操作结果
#[derive(Debug, Serialize, Deserialize)]
pub struct EditResult {
    pub success: bool,
    pub message: String,
}

/// 新增 Server 块
#[tauri::command]
pub async fn add_server_block(
    config_path: String,
    server_input: ServerBlockInput,
) -> Result<EditResult, String> {
    // 读取配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 生成新的 server 块内容
    let mut server_content = String::from("\nserver {\n");

    // 添加 listen 指令
    for listen in &server_input.listen {
        server_content.push_str(&format!("    listen {};\n", listen));
    }

    // 添加 server_name 指令
    if !server_input.server_name.is_empty() {
        server_content.push_str(&format!("    server_name {};\n", server_input.server_name.join(" ")));
    }

    // 添加其他指令
    for directive in &server_input.directives {
        if !directive.value.is_empty() {
            server_content.push_str(&format!("    {} {};\n", directive.name, directive.value));
        } else {
            server_content.push_str(&format!("    {};\n", directive.name));
        }
    }

    server_content.push_str("}\n");

    // 将新的 server 块添加到配置文件末尾
    let new_content = content + &server_content;

    // 写入配置文件
    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "Server 块添加成功".to_string(),
    })
}

/// 更新 Server 块
#[tauri::command]
pub async fn update_server_block(
    config_path: String,
    server_id: String,
    server_input: ServerBlockInput,
) -> Result<EditResult, String> {
    // 读取并解析配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config = parse_nginx_config(&content, &config_path)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 找到要更新的 server 块
    let server = config.servers.iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("未找到 ID 为 {} 的 Server 块", server_id))?;

    // 生成新的 server 块内容
    let mut new_server_content = String::from("server {\n");

    // 添加 listen 指令
    for listen in &server_input.listen {
        new_server_content.push_str(&format!("    listen {};\n", listen));
    }

    // 添加 server_name 指令
    if !server_input.server_name.is_empty() {
        new_server_content.push_str(&format!("    server_name {};\n", server_input.server_name.join(" ")));
    }

    // 添加其他指令
    for directive in &server_input.directives {
        if !directive.value.is_empty() {
            new_server_content.push_str(&format!("    {} {};\n", directive.name, directive.value));
        } else {
            new_server_content.push_str(&format!("    {};\n", directive.name));
        }
    }

    // 保留原有的 location 块
    for location in &server.locations {
        new_server_content.push_str("\n");
        new_server_content.push_str(&location.raw_content);
    }

    new_server_content.push_str("}\n");

    // 替换原有的 server 块
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();

    // 复制 server 块之前的内容
    for i in 0..(server.start_line - 1) {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    // 添加新的 server 块内容
    for line in new_server_content.lines() {
        new_lines.push(line.to_string());
    }

    // 复制 server 块之后的内容
    for i in server.end_line..lines.len() {
        new_lines.push(lines[i].to_string());
    }

    let new_content = new_lines.join("\n");

    // 写入配置文件
    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "Server 块更新成功".to_string(),
    })
}

/// 删除 Server 块
#[tauri::command]
pub async fn delete_server_block(
    config_path: String,
    server_id: String,
) -> Result<EditResult, String> {
    // 读取并解析配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config = parse_nginx_config(&content, &config_path)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 找到要删除的 server 块
    let server = config.servers.iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("未找到 ID 为 {} 的 Server 块", server_id))?;

    // 删除 server 块
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();

    // 复制 server 块之前的内容
    for i in 0..(server.start_line - 1) {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    // 跳过 server 块

    // 复制 server 块之后的内容
    for i in server.end_line..lines.len() {
        new_lines.push(lines[i].to_string());
    }

    let new_content = new_lines.join("\n");

    // 写入配置文件
    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "Server 块删除成功".to_string(),
    })
}

/// 在 Server 块中添加 Location 块
#[tauri::command]
pub async fn add_location_to_server(
    config_path: String,
    server_id: String,
    location_input: LocationBlockInput,
) -> Result<EditResult, String> {
    // 读取并解析配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config = parse_nginx_config(&content, &config_path)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 找到目标 server 块
    let server = config.servers.iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("未找到 ID 为 {} 的 Server 块", server_id))?;

    // 生成新的 location 块内容
    let mut location_content = String::from("\n    location ");
    if let Some(modifier) = &location_input.modifier {
        location_content.push_str(&format!("{} ", modifier));
    }
    location_content.push_str(&format!("{} {{\n", location_input.path));

    // 添加指令
    for directive in &location_input.directives {
        if !directive.value.is_empty() {
            location_content.push_str(&format!("        {} {};\n", directive.name, directive.value));
        } else {
            location_content.push_str(&format!("        {};\n", directive.name));
        }
    }

    location_content.push_str("    }\n");

    // 在 server 块的结束花括号之前插入 location 块
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();

    // 复制到 server 块结束之前的内容
    for i in 0..(server.end_line - 1) {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    // 添加新的 location 块
    for line in location_content.lines() {
        new_lines.push(line.to_string());
    }

    // 添加 server 块的结束花括号和之后的内容
    for i in (server.end_line - 1)..lines.len() {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    let new_content = new_lines.join("\n");

    // 写入配置文件
    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "Location 块添加成功".to_string(),
    })
}

/// 查找 http 块的位置
fn find_http_block(lines: &[&str]) -> Result<(usize, usize), String> {
    let mut http_start = None;
    let mut depth = 0;

    // 查找 http 块的开始位置
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("http") && trimmed.contains('{') {
            http_start = Some(i);
            depth = 1;
            break;
        }
    }

    let http_start = http_start.ok_or_else(|| {
        "未找到 http 块，请确保配置文件中包含 http {} 块".to_string()
    })?;

    // 查找 http 块的结束位置（匹配的右花括号）
    for i in (http_start + 1)..lines.len() {
        let line = lines[i].trim();

        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return Ok((http_start, i));
                }
            }
        }
    }

    Err("http 块未正确闭合，缺少匹配的右花括号".to_string())
}

/// 生成添加 Server 块后的新配置内容（不保存到文件）
#[tauri::command]
pub async fn generate_add_server_content(
    config_path: String,
    server_text: String,
) -> Result<String, String> {
    // 读取配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let lines: Vec<&str> = content.lines().collect();

    // 查找 http 块的位置
    let (_http_start, http_end) = find_http_block(&lines)
        .map_err(|e| format!("定位 http 块失败: {}", e))?;

    // 构建新的配置内容
    let mut new_lines = Vec::new();

    // 复制 http 块结束之前的所有内容
    for i in 0..http_end {
        new_lines.push(lines[i].to_string());
    }

    // 添加新的 server 块（在 http 块的右花括号之前）
    new_lines.push(String::new()); // 空行
    for line in server_text.trim().lines() {
        // 为 server 块添加适当的缩进（4个空格）
        if line.trim().is_empty() {
            new_lines.push(String::new());
        } else {
            new_lines.push(format!("    {}", line));
        }
    }
    new_lines.push(String::new()); // 空行

    // 添加 http 块的右花括号和之后的内容
    for i in http_end..lines.len() {
        new_lines.push(lines[i].to_string());
    }

    let new_content = new_lines.join("\n");
    Ok(new_content)
}

/// 添加 Server 块（文本格式）- 先校验再保存
#[tauri::command]
pub async fn add_server_block_text(
    config_path: String,
    server_text: String,
) -> Result<EditResult, String> {
    // 生成新配置内容
    let new_content = generate_add_server_content(config_path.clone(), server_text).await?;

    // 写入配置文件
    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "Server 块添加成功".to_string(),
    })
}

/// 生成更新 Server 块后的新配置内容（不保存到文件）
#[tauri::command]
pub async fn generate_update_server_content(
    config_path: String,
    server_id: String,
    server_text: String,
) -> Result<String, String> {
    // 读取并解析配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config = parse_nginx_config(&content, &config_path)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 找到要更新的 server 块
    let server = config.servers.iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("未找到 ID 为 {} 的 Server 块", server_id))?;

    // 替换 server 块
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();

    // 复制 server 块之前的内容
    for i in 0..(server.start_line - 1) {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    // 添加新的 server 块内容
    for line in server_text.lines() {
        new_lines.push(line.to_string());
    }

    // 复制 server 块之后的内容
    for i in server.end_line..lines.len() {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    let new_content = new_lines.join("\n");
    Ok(new_content)
}

/// 更新 Server 块（文本格式）- 先校验再保存
#[tauri::command]
pub async fn update_server_block_text(
    config_path: String,
    server_id: String,
    server_text: String,
) -> Result<EditResult, String> {
    // 生成新配置内容
    let new_content = generate_update_server_content(config_path.clone(), server_id, server_text).await?;

    // 写入配置文件
    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "Server 块更新成功".to_string(),
    })
}

/// 将内容写入临时文件用于校验
#[tauri::command]
pub async fn write_temp_config_for_validation(
    original_config_path: String,
    new_content: String,
) -> Result<String, String> {
    use std::path::Path;

    let original_path = Path::new(&original_config_path);
    let parent_dir = original_path.parent()
        .ok_or_else(|| "无法获取配置文件目录".to_string())?;

    let temp_file_name = format!(
        ".nginx_temp_{}.conf",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let temp_path = parent_dir.join(temp_file_name);
    let temp_path_str = temp_path.to_str()
        .ok_or_else(|| "临时文件路径转换失败".to_string())?
        .to_string();

    // 写入临时文件
    fs::write(&temp_path, new_content)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    Ok(temp_path_str)
}

/// 删除临时配置文件
#[tauri::command]
pub async fn delete_temp_config(temp_path: String) -> Result<(), String> {
    if temp_path.contains(".nginx_temp_") {
        let _ = fs::remove_file(&temp_path);
    }
    Ok(())
}

/// 读取配置文件原始内容（用于格式化）
#[tauri::command]
pub async fn read_config_file_content(config_path: String) -> Result<String, String> {
    fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))
}

/// 写入格式化后的配置文件
#[tauri::command]
pub async fn write_formatted_config(
    config_path: String,
    formatted_content: String,
) -> Result<EditResult, String> {
    // 写入配置文件
    fs::write(&config_path, formatted_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: "配置文件格式化成功".to_string(),
    })
}

