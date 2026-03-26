import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  FileChangeLogDetail,
  LogEntry,
  LogKind,
  LogLevel,
  StoredLogEntry,
} from '../types/nginx';
import { useSettingsStore } from './settings';

const sanitizeRetentionDays = (days: number) => {
  if (!Number.isFinite(days)) {
    return 30;
  }

  return Math.min(365, Math.max(1, Math.round(days)));
};

const compareLogs = (left: LogEntry, right: LogEntry) => {
  const timeDelta = right.timestamp.getTime() - left.timestamp.getTime();
  if (timeDelta !== 0) {
    return timeDelta;
  }

  return right.id.localeCompare(left.id);
};

const mergePersistedWithPendingLogs = (
  persistedEntries: LogEntry[],
  currentEntries: LogEntry[],
  preserveAllPending: boolean,
) => {
  const persistedIds = new Set(persistedEntries.map((entry) => entry.id));

  const pendingEntries = currentEntries.filter((entry) => {
    if (persistedIds.has(entry.id)) {
      return false;
    }

    return preserveAllPending || entry.kind === 'operation';
  });

  return [...persistedEntries, ...pendingEntries];
};

export const useLogStore = defineStore('log', () => {
  const logs = ref<LogEntry[]>([]);
  const isLoaded = ref(false);

  const getRetentionDays = () =>
    sanitizeRetentionDays(useSettingsStore().settings.logRetentionDays);

  const toStoredLog = (entry: LogEntry): StoredLogEntry => ({
    id: entry.id,
    level: entry.level,
    summary: entry.summary,
    timestampMs: entry.timestamp.getTime(),
    kind: entry.kind,
    detail: entry.detail,
  });

  const fromStoredLog = (entry: StoredLogEntry): LogEntry => ({
    id: entry.id,
    level: entry.level,
    summary: entry.summary,
    timestamp: new Date(entry.timestampMs),
    kind: entry.kind,
    detail: entry.kind === 'file-change' ? entry.detail : undefined,
  }) as LogEntry;

  const applyLogs = (entries: LogEntry[], mergeWithExisting = false) => {
    const nextLogs = mergeWithExisting ? [...entries, ...logs.value] : entries;
    const dedupedLogs = new Map<string, LogEntry>();

    nextLogs.forEach((entry) => {
      if (!dedupedLogs.has(entry.id)) {
        dedupedLogs.set(entry.id, entry);
      }
    });

    logs.value = Array.from(dedupedLogs.values()).sort(compareLogs);
  };

  const loadPersisted = async (retentionDays = getRetentionDays()) => {
    try {
      const storedLogs = await invoke<StoredLogEntry[]>('load_operation_logs', {
        retentionDays,
      });
      const persistedEntries = storedLogs.map(fromStoredLog);
      applyLogs(
        mergePersistedWithPendingLogs(
          persistedEntries,
          logs.value,
          !isLoaded.value,
        ),
      );
    } catch (error) {
      console.error('加载持久化日志失败:', error);
    } finally {
      isLoaded.value = true;
    }
  };

  const initialize = async () => {
    await loadPersisted();
  };

  const createLogEntry = (
    level: LogLevel,
    summary: string,
    kind: LogKind,
    detail?: FileChangeLogDetail,
  ): LogEntry => ({
    id: `${Date.now()}-${Math.random()}`,
    timestamp: new Date(),
    level,
    summary,
    kind,
    detail,
  }) as LogEntry;

  const persistEntry = async (entry: LogEntry) => {
    try {
      await invoke('append_operation_log', {
        entry: toStoredLog(entry),
        retentionDays: getRetentionDays(),
      });

      if (entry.kind === 'file-change') {
        await loadPersisted();
      }
    } catch (error) {
      console.error('保存日志失败:', error);
    }
  };

  const appendEntry = (entry: LogEntry) => {
    applyLogs([entry], true);
    void persistEntry(entry);
  };

  const addLog = (level: LogLevel, summary: string) => {
    appendEntry(createLogEntry(level, summary, 'operation'));
  };

  const recordFileChange = (
    level: LogLevel,
    summary: string,
    detail: FileChangeLogDetail,
  ) => {
    appendEntry(createLogEntry(level, summary, 'file-change', detail));
  };

  const info = (summary: string) => addLog('info', summary);
  const success = (summary: string) => addLog('success', summary);
  const warning = (summary: string) => addLog('warning', summary);
  const error = (summary: string) => addLog('error', summary);

  const clear = async () => {
    logs.value = [];
    try {
      await invoke('clear_operation_logs');
    } catch (error) {
      console.error('清空日志失败:', error);
    }
  };

  return {
    logs,
    isLoaded,
    initialize,
    loadPersisted,
    addLog,
    recordFileChange,
    info,
    success,
    warning,
    error,
    clear,
  };
});
