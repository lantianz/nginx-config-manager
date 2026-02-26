import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { NginxStatus } from '../types/nginx';
import { eventBus, EVENTS } from '../composables/useEventBus';
import type { NginxOperationResult } from '../composables/useEventBus';

export const useNginxStore = defineStore('nginx', () => {
  const status = ref<NginxStatus>({
    isRunning: false,
    processCount: 0,
  });

  const isLoading = ref(false);

  // 检查 Nginx 状态（保持 async，供刷新按钮直接调用）
  const checkStatus = async () => {
    try {
      isLoading.value = true;
      const result = await invoke<{ is_running: boolean; process_count: number; message: string }>('check_nginx_status');
      status.value = {
        isRunning: result.is_running,
        processCount: result.process_count,
        lastOperation: result.message,
      };
    } catch (error) {
      console.error('检查状态失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * 内部调度：fire-and-forget 执行操作，完成后 emit 结果事件并异步刷新状态
   */
  const _dispatch = (
    operation: NginxOperationResult['operation'],
    promise: Promise<{ success: boolean; message: string }>,
  ) => {
    isLoading.value = true;
    promise
      .then(result => {
        eventBus.emit<NginxOperationResult>(EVENTS.NGINX_OPERATION_RESULT, { ...result, operation });
        checkStatus(); // 异步刷新状态，不阻塞
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
    _dispatch('start', invoke('start_nginx', { nginxPath }));

  const stop = () =>
    _dispatch('stop', invoke('stop_nginx'));

  const restart = (nginxPath: string) =>
    _dispatch('restart', invoke('restart_nginx', { nginxPath }));

  const reload = (nginxPath: string) =>
    _dispatch('reload', invoke('reload_nginx', { nginxPath }));

  const testConfig = (nginxPath: string) =>
    _dispatch('test', invoke('test_nginx_config', { nginxPath }));

  return { status, isLoading, checkStatus, start, stop, restart, reload, testConfig };
});
