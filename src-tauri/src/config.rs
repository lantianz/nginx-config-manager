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
    pub relative_start_line: usize,
    pub relative_end_line: usize,
}

/// Server 配置块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBlock {
    pub id: String,
    pub enabled: bool,
    pub category: Option<String>,
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

const DISABLED_SERVER_BEGIN_MARKER: &str = "# nginx-config-manager managed-disabled-server begin";
const DISABLED_SERVER_END_MARKER: &str = "# nginx-config-manager managed-disabled-server end";

fn is_managed_disabled_server_start(line: &str) -> bool {
    line == DISABLED_SERVER_BEGIN_MARKER
}

fn is_managed_disabled_server_end(line: &str) -> bool {
    line == DISABLED_SERVER_END_MARKER
}

fn comment_out_line(line: &str) -> String {
    if line.is_empty() {
        "#".to_string()
    } else {
        format!("# {}", line)
    }
}

fn uncomment_managed_line(line: &str) -> String {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return line.to_string();
    }

    let prefix_len = line.len() - trimmed.len();
    let restored = if trimmed.len() <= 1 {
        ""
    } else if trimmed.as_bytes()[1] == b' ' {
        &trimmed[2..]
    } else {
        &trimmed[1..]
    };

    format!("{}{}", &line[..prefix_len], restored)
}

fn wrap_disabled_server_block(server_text: &str) -> String {
    let mut lines = Vec::new();
    lines.push(DISABLED_SERVER_BEGIN_MARKER.to_string());
    lines.extend(server_text.lines().map(comment_out_line));
    lines.push(DISABLED_SERVER_END_MARKER.to_string());
    lines.join("\n")
}

fn unwrap_disabled_server_block(disabled_text: &str) -> Result<String, String> {
    let lines: Vec<&str> = disabled_text.lines().collect();

    if lines.len() < 3 {
        return Err("停用的 server 块格式不完整".to_string());
    }

    if !is_managed_disabled_server_start(lines[0].trim()) {
        return Err("停用的 server 块缺少起始标记".to_string());
    }

    if !is_managed_disabled_server_end(lines[lines.len() - 1].trim()) {
        return Err("停用的 server 块缺少结束标记".to_string());
    }

    Ok(lines[1..lines.len() - 1]
        .iter()
        .map(|line| uncomment_managed_line(line))
        .collect::<Vec<String>>()
        .join("\n"))
}

fn render_server_block_by_state(server_text: &str, enabled: bool) -> String {
    if enabled {
        server_text.to_string()
    } else {
        wrap_disabled_server_block(server_text)
    }
}

fn replace_server_range(content: &str, start_line: usize, end_line: usize, replacement: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();

    for i in 0..(start_line - 1) {
        if i < lines.len() {
            new_lines.push(lines[i].to_string());
        }
    }

    for line in replacement.lines() {
        new_lines.push(line.to_string());
    }

    for i in end_line..lines.len() {
        new_lines.push(lines[i].to_string());
    }

    new_lines.join("\n")
}

fn parse_server_category(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }

    let value = trimmed.trim_start_matches('#').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
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

        if is_managed_disabled_server_start(line) {
            match parse_disabled_server_block(&lines, i) {
                Ok((server, end_line)) => {
                    servers.push(server);
                    i = end_line + 1;
                }
                Err(e) => {
                    return Err(format!("解析停用的 server 块失败 (行 {}): {}", i + 1, e));
                }
            }
            continue;
        }

        // 跳过空行和普通注释
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
    let mut category = None;
    let mut seen_content = false;
    
    // 查找 server 块的结束位置
    let end = find_block_end(lines, start)?;
    
    // 提取原始内容
    let raw_content = lines[start..=end].join("\n");
    
    // 解析 server 块内容
    let mut i = start + 1;
    while i < end {
        let line = lines[i].trim();
        
        if line.is_empty() {
            i += 1;
            continue;
        }

        if line.starts_with('#') {
            if !seen_content && category.is_none() {
                category = parse_server_category(line);
            }
            i += 1;
            continue;
        }
        
        // 检测 location 块
        if line.starts_with("location") && line.contains('{') {
            seen_content = true;
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
                seen_content = true;
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
    for location in &mut locations {
        location.relative_start_line = location.start_line.saturating_sub(start);
        location.relative_end_line = location.end_line.saturating_sub(start);
    }

    Ok((
        ServerBlock {
            id,
            enabled: true,
            category,
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

fn find_disabled_server_end(lines: &[&str], start: usize) -> Result<usize, String> {
    for i in (start + 1)..lines.len() {
        if is_managed_disabled_server_end(lines[i].trim()) {
            return Ok(i);
        }
    }

    Err("未找到停用 server 块的结束标记".to_string())
}

fn parse_disabled_server_block(lines: &[&str], start: usize) -> Result<(ServerBlock, usize), String> {
    let end = find_disabled_server_end(lines, start)?;
    let disabled_text = lines[start..=end].join("\n");
    let restored = unwrap_disabled_server_block(&disabled_text)?;
    let restored_lines: Vec<String> = restored.lines().map(|line| line.to_string()).collect();
    let restored_refs: Vec<&str> = restored_lines.iter().map(|line| line.as_str()).collect();
    let (mut server, _) = parse_server_block(&restored_refs, 0)?;

    server.id = format!("server_disabled_{}", start);
    server.enabled = false;
    server.raw_content = restored;
    server.start_line = start + 1;
    server.end_line = end + 1;

    Ok((server, end))
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
            relative_start_line: 0,
            relative_end_line: 0,
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

    if let Some(category) = &server.category {
        new_server_content.push_str(&format!("    # {}\n", category));
    }

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
    let replacement = render_server_block_by_state(&new_server_content, server.enabled);
    let new_content = replace_server_range(&content, server.start_line, server.end_line, &replacement);

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

    let replacement = render_server_block_by_state(&server_text, server.enabled);
    let new_content = replace_server_range(&content, server.start_line, server.end_line, &replacement);
    Ok(new_content)
}

#[tauri::command]
pub async fn generate_toggle_server_state_content(
    config_path: String,
    server_id: String,
    enabled: bool,
) -> Result<String, String> {
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config = parse_nginx_config(&content, &config_path)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    let server = config.servers.iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("未找到 ID 为 {} 的 Server 块", server_id))?;

    if server.enabled == enabled {
        return Ok(content);
    }

    let replacement = render_server_block_by_state(&server.raw_content, enabled);
    Ok(replace_server_range(&content, server.start_line, server.end_line, &replacement))
}

#[tauri::command]
pub async fn set_server_enabled_state(
    config_path: String,
    server_id: String,
    enabled: bool,
) -> Result<EditResult, String> {
    let new_content = generate_toggle_server_state_content(config_path.clone(), server_id, enabled).await?;

    fs::write(&config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(EditResult {
        success: true,
        message: if enabled {
            "Server 块已恢复启用".to_string()
        } else {
            "Server 块已临时停用".to_string()
        },
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SERVER: &str = "    server {\n        listen 8080;\n        server_name demo.local;\n    }";

    #[test]
    fn wrap_and_unwrap_disabled_server_block_should_round_trip() {
        let disabled = wrap_disabled_server_block(SAMPLE_SERVER);
        let restored = unwrap_disabled_server_block(&disabled).expect("restore disabled block");

        assert_eq!(restored, SAMPLE_SERVER);
    }

    #[test]
    fn parse_nginx_config_should_keep_managed_disabled_servers() {
        let disabled = wrap_disabled_server_block(SAMPLE_SERVER);
        let content = format!("events {{}}\nhttp {{\n{}\n}}\n", disabled);

        let config = parse_nginx_config(&content, "test.conf").expect("parse config");

        assert_eq!(config.servers.len(), 1);
        assert!(!config.servers[0].enabled);
        assert_eq!(config.servers[0].listen, vec!["8080"]);
        assert_eq!(config.servers[0].server_name, vec!["demo.local"]);
    }

    #[test]
    fn parse_server_block_should_extract_category_and_relative_location_lines() {
        let content = r#"http {
    server {
        # 位置能力服务
        listen 8081;
        server_name map.local;

        location /api {
            proxy_pass http://127.0.0.1:9000;
        }
    }
}"#;

        let config = parse_nginx_config(content, "test.conf").expect("parse config");
        let server = &config.servers[0];
        let location = &server.locations[0];

        assert_eq!(server.category.as_deref(), Some("位置能力服务"));
        assert_eq!(location.relative_start_line, 6);
        assert_eq!(location.relative_end_line, 8);
    }
}
