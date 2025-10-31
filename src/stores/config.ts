import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { NginxConfig, ServerBlock, LocationBlock } from '@/types/config';

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
     */
    async loadConfig(configPath: string): Promise<boolean> {
      this.loading = true;
      this.error = null;

      try {
        const result = await invoke<ParseResult>('read_config_file', {
          configPath,
        });

        if (result.success && result.config) {
          this.config = result.config;
          this.error = null;
          return true;
        } else {
          this.error = result.message;
          this.config = null;
          return false;
        }
      } catch (error) {
        this.error = `加载配置文件失败: ${error}`;
        this.config = null;
        return false;
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
      return await this.loadConfig(this.config.filePath);
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
