// Nginx 相关类型定义

/**
 * Nginx 状态信息
 */
export interface NginxStatus {
  isRunning: boolean;
  processCount: number;
  uptime?: string;
  lastOperation?: string;
}

export interface PermissionStatus {
  isWindows: boolean;
  isElevated: boolean;
  message: string;
}

export interface PortProcessInfo {
  protocol: string;
  localAddress: string;
  localPort: number;
  pid: number;
  processName: string;
  executablePath?: string | null;
  commandLine?: string | null;
  user?: string | null;
  status: string;
  startTime?: string | null;
}

export interface PortInspectionResult {
  port: number;
  isOccupied: boolean;
  entries: PortProcessInfo[];
  permissionStatus: PermissionStatus;
  message: string;
}

export interface ProcessOperationResult {
  success: boolean;
  message: string;
  requiresElevation: boolean;
}

/**
 * Nginx 配置校验结果
 */
export interface TestResult {
  success: boolean;
  output: string;
  errors?: string[];
  warnings?: string[];
}

/**
 * 日志级别类型
 */
export type LogLevel = 'info' | 'success' | 'warning' | 'error';

export type LogKind = 'operation' | 'file-change';

export type LogViewFilter = 'all' | 'file-change';

export interface FileChangeScopeDiff {
  label: string;
  before: string;
  after: string;
}

export interface FileChangeLogDetail {
  operationLabel: string;
  configPath: string;
  savedAt: number;
  fileDiff: FileChangeScopeDiff;
  serverDiff?: FileChangeScopeDiff | null;
  locationDiffs: FileChangeScopeDiff[];
}

/**
 * 操作日志条目
 */
interface BaseLogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  kind: LogKind;
  summary: string;
}

export interface OperationLogEntry extends BaseLogEntry {
  kind: 'operation';
  detail?: undefined;
}

export interface FileChangeLogEntry extends BaseLogEntry {
  kind: 'file-change';
  detail: FileChangeLogDetail;
}

export type LogEntry = OperationLogEntry | FileChangeLogEntry;

export interface StoredLogEntry {
  id: string;
  level: LogLevel;
  timestampMs: number;
  kind: LogKind;
  summary: string;
  detail?: FileChangeLogDetail;
}

/**
 * 操作日志级别对应的颜色
 */
export const LogLevelColors = {
  info: '#0064C8',      // 蓝色
  success: '#18A058',   // 绿色
  warning: '#F0A020',   // 橙色
  error: '#D03050',     // 红色
} as const;
