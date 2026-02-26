import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types/common';
import { eventBus, EVENTS } from '../composables/useEventBus';

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    nginxPath: '',
    configPath: '',
    theme: 'dark',
    language: 'zh-CN',
  });

  /** 标记设置是否已从后端加载完成，避免重复加载 */
  const isLoaded = ref(false);

  const loadSettings = async () => {
    try {
      const savedSettings = await invoke<AppSettings>('load_app_settings');
      settings.value = {
        nginxPath: savedSettings.nginxPath || '',
        configPath: savedSettings.configPath || '',
        theme: savedSettings.theme || 'auto',
        language: savedSettings.language || 'zh-CN',
      };
      isLoaded.value = true;
      eventBus.emit(EVENTS.SETTINGS_LOADED);
    } catch (error) {
      console.error('加载设置失败:', error);
    }
  };

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

  const updateNginxPath = (path: string) => {
    settings.value.nginxPath = path;
    saveSettings();
  };

  const updateConfigPath = (path: string) => {
    settings.value.configPath = path;
    saveSettings();
  };

  const updateTheme = (theme: 'light' | 'dark' | 'auto') => {
    settings.value.theme = theme;
    saveSettings();
  };

  return { settings, isLoaded, loadSettings, saveSettings, updateNginxPath, updateConfigPath, updateTheme };
});
