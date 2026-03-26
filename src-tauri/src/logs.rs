use crate::settings::get_app_config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MILLIS_PER_DAY: i64 = 24 * 60 * 60 * 1000;
const MAX_FILE_CHANGE_LOGS: usize = 3;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LogKind {
    #[default]
    Operation,
    FileChange,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FileChangeScopeDiff {
    pub label: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FileChangeLogDetail {
    pub operation_label: String,
    pub config_path: String,
    pub saved_at: i64,
    pub file_diff: FileChangeScopeDiff,
    pub server_diff: Option<FileChangeScopeDiff>,
    pub location_diffs: Vec<FileChangeScopeDiff>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct StoredLogEntry {
    pub id: String,
    pub level: String,
    #[serde(alias = "message")]
    pub summary: String,
    pub timestamp_ms: i64,
    pub kind: LogKind,
    pub detail: Option<FileChangeLogDetail>,
}

fn get_logs_path() -> Result<PathBuf, String> {
    Ok(get_app_config_dir()?.join("logs.json"))
}

fn now_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn normalize_retention_days(retention_days: u32) -> u32 {
    retention_days.clamp(1, 365)
}

fn sort_logs_desc(entries: &mut [StoredLogEntry]) {
    entries.sort_by(|left, right| {
        right
            .timestamp_ms
            .cmp(&left.timestamp_ms)
            .then_with(|| right.id.cmp(&left.id))
    });
}

fn normalize_logs(
    entries: Vec<StoredLogEntry>,
    retention_days: u32,
    now_ms: i64,
) -> Vec<StoredLogEntry> {
    let cutoff = now_ms
        .saturating_sub(i64::from(normalize_retention_days(retention_days)) * MILLIS_PER_DAY);
    let mut operation_logs = Vec::new();
    let mut file_change_logs = Vec::new();

    for entry in entries {
        match entry.kind {
            LogKind::Operation => {
                if entry.timestamp_ms >= cutoff {
                    operation_logs.push(entry);
                }
            }
            LogKind::FileChange => file_change_logs.push(entry),
        }
    }

    sort_logs_desc(&mut operation_logs);
    sort_logs_desc(&mut file_change_logs);
    file_change_logs.truncate(MAX_FILE_CHANGE_LOGS);

    operation_logs.extend(file_change_logs);
    sort_logs_desc(&mut operation_logs);
    operation_logs
}

fn save_logs_to_path(path: &Path, entries: &[StoredLogEntry]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("序列化日志失败: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("写入日志文件失败: {}", e))
}

fn load_logs_from_path(path: &Path, retention_days: u32) -> Result<Vec<StoredLogEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(path)
        .map_err(|e| format!("读取日志文件失败: {}", e))?;

    if json.trim().is_empty() {
        return Ok(Vec::new());
    }

    let entries: Vec<StoredLogEntry> = serde_json::from_str(&json)
        .map_err(|e| format!("解析日志文件失败: {}", e))?;

    Ok(normalize_logs(entries, retention_days, now_timestamp_ms()))
}

#[tauri::command]
pub async fn load_operation_logs(retention_days: u32) -> Result<Vec<StoredLogEntry>, String> {
    let logs_path = get_logs_path()?;
    let entries = load_logs_from_path(&logs_path, retention_days)?;

    if logs_path.exists() {
        save_logs_to_path(&logs_path, &entries)?;
    }

    Ok(entries)
}

#[tauri::command]
pub async fn append_operation_log(
    entry: StoredLogEntry,
    retention_days: u32,
) -> Result<bool, String> {
    let logs_path = get_logs_path()?;
    let mut entries = load_logs_from_path(&logs_path, retention_days)?;
    entries.push(entry);

    let normalized = normalize_logs(entries, retention_days, now_timestamp_ms());
    save_logs_to_path(&logs_path, &normalized)?;

    Ok(true)
}

#[tauri::command]
pub async fn clear_operation_logs() -> Result<bool, String> {
    let logs_path = get_logs_path()?;

    match fs::remove_file(&logs_path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!("删除日志文件失败: {}", error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        env::temp_dir().join(format!("{}_{}", name, unique))
    }

    fn create_operation_log(id: &str, timestamp_ms: i64) -> StoredLogEntry {
        StoredLogEntry {
            id: id.to_string(),
            level: "info".to_string(),
            summary: id.to_string(),
            timestamp_ms,
            kind: LogKind::Operation,
            detail: None,
        }
    }

    fn create_file_change_log(id: &str, timestamp_ms: i64) -> StoredLogEntry {
        StoredLogEntry {
            id: id.to_string(),
            level: "success".to_string(),
            summary: id.to_string(),
            timestamp_ms,
            kind: LogKind::FileChange,
            detail: Some(FileChangeLogDetail {
                operation_label: "更新 Server".to_string(),
                config_path: "test.conf".to_string(),
                saved_at: timestamp_ms,
                file_diff: FileChangeScopeDiff {
                    label: "文件".to_string(),
                    before: "before".to_string(),
                    after: "after".to_string(),
                },
                server_diff: None,
                location_diffs: Vec::new(),
            }),
        }
    }

    #[test]
    fn normalize_logs_should_drop_expired_operation_logs() {
        let now_ms = 10 * MILLIS_PER_DAY;
        let entries = vec![
            create_operation_log("recent", now_ms - MILLIS_PER_DAY),
            create_operation_log("expired", now_ms - 31 * MILLIS_PER_DAY),
        ];

        let normalized = normalize_logs(entries, 30, now_ms);

        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].id, "recent");
    }

    #[test]
    fn normalize_logs_should_keep_only_latest_file_change_logs() {
        let entries = vec![
            create_file_change_log("1", 1),
            create_file_change_log("2", 2),
            create_file_change_log("3", 3),
            create_file_change_log("4", 4),
        ];

        let normalized = normalize_logs(entries, 30, 100);

        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[0].id, "4");
        assert_eq!(normalized[1].id, "3");
        assert_eq!(normalized[2].id, "2");
    }

    #[test]
    fn deserialize_legacy_logs_should_map_message_to_summary() {
        let json = r#"[
            {
                "id": "legacy",
                "level": "info",
                "message": "old message",
                "timestampMs": 123
            }
        ]"#;

        let entries: Vec<StoredLogEntry> = serde_json::from_str(json).expect("deserialize legacy logs");

        assert_eq!(entries[0].summary, "old message");
        assert_eq!(entries[0].kind, LogKind::Operation);
    }

    #[test]
    fn load_logs_from_missing_path_should_return_empty() {
        let path = create_temp_path("missing_logs.json");

        let entries = load_logs_from_path(&path, 30).expect("load missing logs");

        assert!(entries.is_empty());
    }

    #[test]
    fn load_logs_from_empty_file_should_return_empty() {
        let path = create_temp_path("empty_logs.json");
        fs::write(&path, "").expect("write empty log file");

        let entries = load_logs_from_path(&path, 30).expect("load empty logs");

        assert!(entries.is_empty());
        let _ = fs::remove_file(path);
    }
}
