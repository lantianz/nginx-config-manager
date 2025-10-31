import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types/common';

export const useSettingsStore = defineStore('settings', () => {
  // 状态
  const settings = ref<AppSettings>({
    nginxPath: '',
    configPath: '',
    theme: 'dark',
    language: 'zh-CN',
  });

  // 加载设置
  const loadSettings = async () => {
    try {
      const savedSettings = await invoke<AppSettings>('load_app_settings');
      settings.value = {
        nginxPath: savedSettings.nginxPath || '',
        configPath: savedSettings.configPath || '',
        theme: savedSettings.theme || 'auto',
        language: savedSettings.language || 'zh-CN',
      };
    } catch (error) {
      console.error('加载设置失败:', error);
    }
  };

  // 保存设置
  const saveSettings = async () => {
    try {
      await invoke('save_app_settings', {
        settings: {
          nginxPath: settings.value.nginxPath,
          configPath: settings.value.configPath,
          theme: settings.value.theme,
          language: settings.value.language,
        },
      });
    } catch (error) {
      console.error('保存设置失败:', error);
    }
  };

  // 更新 Nginx 路径
  const updateNginxPath = (path: string) => {
    settings.value.nginxPath = path;
    saveSettings();
  };

  // 更新配置文件路径
  const updateConfigPath = (path: string) => {
    settings.value.configPath = path;
    saveSettings();
  };

  // 更新主题
  const updateTheme = (theme: 'light' | 'dark' | 'auto') => {
    settings.value.theme = theme;
    saveSettings();
  };

  return {
    settings,
    loadSettings,
    saveSettings,
    updateNginxPath,
    updateConfigPath,
    updateTheme,
  };
});

