import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { NginxStatus } from '../types/nginx';

export const useNginxStore = defineStore('nginx', () => {
  // 状态
  const status = ref<NginxStatus>({
    isRunning: false,
    processCount: 0,
  });

  const isLoading = ref(false);

  // 检查 Nginx 状态
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

  // 启动 Nginx
  const start = async (nginxPath: string) => {
    try {
      isLoading.value = true;
      const result = await invoke<{ success: boolean; message: string }>('start_nginx', { nginxPath });
      await checkStatus();
      return result;
    } catch (error) {
      console.error('启动失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  // 停止 Nginx
  const stop = async () => {
    try {
      isLoading.value = true;
      const result = await invoke<{ success: boolean; message: string }>('stop_nginx');
      await checkStatus();
      return result;
    } catch (error) {
      console.error('停止失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  // 重启 Nginx
  const restart = async (nginxPath: string) => {
    try {
      isLoading.value = true;
      const result = await invoke<{ success: boolean; message: string }>('restart_nginx', { nginxPath });
      await checkStatus();
      return result;
    } catch (error) {
      console.error('重启失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  // 重载配置
  const reload = async (nginxPath: string) => {
    try {
      isLoading.value = true;
      const result = await invoke<{ success: boolean; message: string }>('reload_nginx', { nginxPath });
      await checkStatus();
      return result;
    } catch (error) {
      console.error('重载失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  // 校验配置
  const testConfig = async (nginxPath: string) => {
    try {
      isLoading.value = true;
      const result = await invoke<{ success: boolean; message: string }>('test_nginx_config', { nginxPath });
      return result;
    } catch (error) {
      console.error('校验配置失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
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
  };
});

