import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { LogEntry } from '../types/nginx';

export const useLogStore = defineStore('log', () => {
  // 状态
  const logs = ref<LogEntry[]>([]);
  const maxLogs = 1000; // 最多保留 1000 条日志

  // 添加日志
  const addLog = (level: LogEntry['level'], message: string) => {
    const log: LogEntry = {
      id: `${Date.now()}-${Math.random()}`,
      timestamp: new Date(),
      level,
      message,
    };

    logs.value.unshift(log);

    // 限制日志数量
    if (logs.value.length > maxLogs) {
      logs.value = logs.value.slice(0, maxLogs);
    }
  };

  // 便捷方法
  const info = (message: string) => addLog('info', message);
  const success = (message: string) => addLog('success', message);
  const warning = (message: string) => addLog('warning', message);
  const error = (message: string) => addLog('error', message);

  // 清空日志
  const clear = () => {
    logs.value = [];
  };

  return {
    logs,
    addLog,
    info,
    success,
    warning,
    error,
    clear,
  };
});

