import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  NginxStatus,
  PermissionStatus,
  PortInspectionResult,
  ProcessOperationResult,
} from '../types/nginx';
import { eventBus, EVENTS } from '../composables/useEventBus';
import type { NginxOperationResult } from '../composables/useEventBus';

export const useNginxStore = defineStore('nginx', () => {
  const status = ref<NginxStatus>({
    isRunning: false,
    processCount: 0,
  });

  const isLoading = ref(false);

  const runWithLoading = async <T>(task: () => Promise<T>) => {
    isLoading.value = true;
    try {
      return await task();
    } finally {
      isLoading.value = false;
    }
  };

  const refreshStatus = async () => {
    const result = await invoke<{ is_running: boolean; process_count: number; message: string }>('check_nginx_status');
    status.value = {
      isRunning: result.is_running,
      processCount: result.process_count,
      lastOperation: result.message,
    };
  };

  const checkStatus = async () => {
    try {
      await runWithLoading(refreshStatus);
    } catch (error) {
      console.error('检查状态失败:', error);
      throw error;
    }
  };

  const dispatch = (
    operation: NginxOperationResult['operation'],
    promise: Promise<{ success: boolean; message: string }>,
  ) => {
    isLoading.value = true;
    promise
      .then(result => {
        eventBus.emit<NginxOperationResult>(EVENTS.NGINX_OPERATION_RESULT, { ...result, operation });
        refreshStatus().catch((error) => {
          console.error('刷新 Nginx 状态失败:', error);
        });
      })
      .catch(error => {
        eventBus.emit<NginxOperationResult>(EVENTS.NGINX_OPERATION_RESULT, {
          success: false,
          message: String(error),
          operation,
        });
      })
      .finally(() => {
        isLoading.value = false;
      });
  };

  const start = (nginxPath: string) =>
    dispatch('start', invoke('start_nginx', { nginxPath }));

  const stop = () =>
    dispatch('stop', invoke('stop_nginx'));

  const restart = (nginxPath: string) =>
    dispatch('restart', invoke('restart_nginx', { nginxPath }));

  const reload = (nginxPath: string) =>
    dispatch('reload', invoke('reload_nginx', { nginxPath }));

  const testConfig = (nginxPath: string) =>
    dispatch('test', invoke('test_nginx_config', { nginxPath }));

  const checkPermissionStatus = () =>
    invoke<PermissionStatus>('check_process_permission_status');

  const inspectPorts = (ports: number[]) =>
    invoke<PortInspectionResult[]>('inspect_ports', { ports });

  const terminateProcess = async (pid: number) => {
    const result = await invoke<ProcessOperationResult>('terminate_process', { pid });
    await refreshStatus().catch(() => undefined);
    return result;
  };

  const releasePort = async (port: number) => {
    const result = await invoke<ProcessOperationResult>('release_port', { port });
    await refreshStatus().catch(() => undefined);
    return result;
  };

  return {
    status,
    isLoading,
    checkStatus,
    start,
    stop,
    restart,
    reload,
    testConfig,
    checkPermissionStatus,
    inspectPorts,
    terminateProcess,
    releasePort,
  };
});
