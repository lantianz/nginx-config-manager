import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { NginxConfig, ServerBlock, LocationBlock } from '@/types/config';
import { useLogStore } from './log';
import { eventBus, EVENTS } from '@/composables/useEventBus';

interface ParseResult {
  success: boolean;
  message: string;
  config?: NginxConfig;
}

interface ConfigState {
  config: NginxConfig | null;
  loading: boolean;
  error: string | null;
  searchQuery: string;
  selectedServerId: string | null;
  selectedLocationId: string | null;
}

export const useConfigStore = defineStore('config', {
  state: (): ConfigState => ({
    config: null,
    loading: false,
    error: null,
    searchQuery: '',
    selectedServerId: null,
    selectedLocationId: null,
  }),

  getters: {
    /**
     * 获取所有 server 块
     */
    servers: (state): ServerBlock[] => {
      return state.config?.servers || [];
    },

    /**
     * 获取过滤后的 server 块（根据搜索条件）
     * 支持搜索：端口号、域名、location 路径
     */
    filteredServers: (state): ServerBlock[] => {
      if (!state.config || !state.searchQuery) {
        return state.config?.servers || [];
      }

      const query = state.searchQuery.toLowerCase();
      return state.config.servers.filter((server) => {
        // 搜索 listen 端口
        const matchListen = (server.listen || []).some((listen) =>
          listen.toLowerCase().includes(query)
        );

        // 搜索 server_name
        const matchServerName = (server.serverName || []).some((name) =>
          name.toLowerCase().includes(query)
        );

        // 搜索 location 路径
        const matchLocation = (server.locations || []).some((location) =>
          location.path.toLowerCase().includes(query)
        );

        return matchListen || matchServerName || matchLocation;
      });
    },

    /**
     * 获取当前选中的 server 块
     */
    selectedServer: (state): ServerBlock | null => {
      if (!state.config || !state.selectedServerId) {
        return null;
      }
      return (
        state.config.servers.find((s) => s.id === state.selectedServerId) || null
      );
    },

    /**
     * 获取当前选中的 location 块
     */
    selectedLocation: (state): LocationBlock | null => {
      if (!state.config || !state.selectedServerId || !state.selectedLocationId) {
        return null;
      }
      const server = state.config.servers.find((s) => s.id === state.selectedServerId);
      if (!server) {
        return null;
      }
      return (
        server.locations.find(
          (l) => l.id === state.selectedLocationId
        ) || null
      );
    },

    /**
     * 获取配置文件路径
     */
    configPath: (state): string => {
      return state.config?.filePath || '';
    },

    /**
     * 是否已加载配置
     */
    hasConfig: (state): boolean => {
      return state.config !== null;
    },

    /**
     * 获取 server 块总数
     */
    serverCount: (state): number => {
      return state.config?.servers.length || 0;
    },

    /**
     * 获取 location 块总数
     */
    locationCount: (state): number => {
      if (!state.config) return 0;
      return state.config.servers.reduce(
        (total, server) => total + server.locations.length,
        0
      );
    },
  },

  actions: {
    /**
     * 读取并解析配置文件
     * @param configPath 配置文件路径
     * @returns 返回包含成功状态和错误信息的对象
     */
    async loadConfig(configPath: string): Promise<{ success: boolean; message?: string }> {
      const logStore = useLogStore();

      this.loading = true;
      this.error = null;

      try {
        const result = await invoke<ParseResult>('read_config_file', {
          configPath: configPath,
        });

        if (!result) {
          const errorMsg = '调用 read_config_file 返回空结果';
          this.error = errorMsg;
          this.config = null;
          logStore.error(`${errorMsg} (路径: ${configPath})`);
          return { success: false, message: errorMsg };
        }

        if (result.success && result.config) {
          this.config = result.config;
          this.error = null;

          // 记录日志
          logStore.info(`配置文件加载成功: ${configPath}`);

          eventBus.emit(EVENTS.CONFIG_LOADED, { success: true });
          return { success: true };
        } else {
          this.error = result.message || '未知错误';
          this.config = null;

          // 记录错误日志
          logStore.error(`配置文件加载失败: ${configPath} - ${result.message}`);

          eventBus.emit(EVENTS.CONFIG_LOADED, { success: false, message: result.message });
          return { success: false, message: result.message };
        }
      } catch (error) {
        console.error('invoke 调用异常:', error);
        const errorMsg = `加载配置文件失败: ${error}`;
        this.error = errorMsg;
        this.config = null;

        // 记录错误日志
        logStore.error(`${errorMsg} (路径: ${configPath})`);

        eventBus.emit(EVENTS.CONFIG_LOADED, { success: false, message: errorMsg });
        return { success: false, message: errorMsg };
      } finally {
        this.loading = false;
      }
    },

    /**
     * 重新加载配置文件
     */
    async reloadConfig(): Promise<boolean> {
      if (!this.config) {
        return false;
      }
      const result = await this.loadConfig(this.config.filePath);
      return result.success;
    },

    /**
     * 清空配置
     */
    clearConfig() {
      this.config = null;
      this.error = null;
      this.searchQuery = '';
      this.selectedServerId = null;
      this.selectedLocationId = null;
    },

    /**
     * 设置搜索条件
     */
    setSearchQuery(query: string) {
      this.searchQuery = query;
    },

    /**
     * 选择 server 块
     */
    selectServer(serverId: string | null) {
      this.selectedServerId = serverId;
      this.selectedLocationId = null; // 切换 server 时清空 location 选择
    },

    /**
     * 选择 location 块
     */
    selectLocation(locationId: string | null) {
      this.selectedLocationId = locationId;
    },

    /**
     * 根据端口查找 server
     */
    findServerByPort(port: string): ServerBlock | undefined {
      if (!this.config) return undefined;
      return this.config.servers.find((server) =>
        server.listen.some((listen) => listen.includes(port))
      );
    },

    /**
     * 根据域名查找 server
     */
    findServerByName(name: string): ServerBlock | undefined {
      if (!this.config) return undefined;
      return this.config.servers.find((server) =>
        server.serverName.some((serverName) => serverName === name)
      );
    },

    /**
     * 获取指定 server 的指令值
     */
    getServerDirective(serverId: string, directiveName: string): string | null {
      if (!this.config) return null;
      const server = this.config.servers.find((s) => s.id === serverId);
      if (!server) return null;

      const directive = server.directives.find((d) => d.name === directiveName);
      return directive ? directive.value : null;
    },

    /**
     * 获取指定 location 的指令值
     */
    getLocationDirective(
      serverId: string,
      locationId: string,
      directiveName: string
    ): string | null {
      if (!this.config) return null;
      const server = this.config.servers.find((s) => s.id === serverId);
      if (!server) return null;

      const location = server.locations.find((l) => l.id === locationId);
      if (!location) return null;

      const directive = location.directives.find((d) => d.name === directiveName);
      return directive ? directive.value : null;
    },
  },
});
