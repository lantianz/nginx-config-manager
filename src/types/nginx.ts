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

/**
 * 操作日志条目
 */
export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  message: string;
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
