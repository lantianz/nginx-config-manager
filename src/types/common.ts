// 通用类型定义

/**
 * 应用设置
 */
export interface AppSettings {
  nginxPath: string;
  configPath: string;
  theme: 'light' | 'dark' | 'auto';
  language: 'zh-CN' | 'en-US';
}

/**
 * 菜单项
 */
export interface MenuItem {
  key: string;
  label: string;
  icon?: string;
  path: string;
}

