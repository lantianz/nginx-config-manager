<template>
  <div class="config-manager">
    <!-- 顶部操作栏 -->
    <n-card :bordered="false" class="toolbar-card">
      <n-space vertical :size="16">
        <!-- 配置文件路径 -->
        <div class="config-input-group">
          <n-text strong>配置文件:</n-text>
          <n-input v-model:value="localConfigPath" placeholder="请输入或选择 nginx.conf 配置文件路径" style="min-width: 0; flex: 1"
            :disabled="configStore.loading">
            <template #suffix>
              <n-button text @click="selectConfigFile" :disabled="configStore.loading">
                <n-icon :component="FolderOpenOutline" />
              </n-button>
            </template>
          </n-input>
          <n-button type="primary" @click="loadConfig(true)" :loading="configStore.loading" :disabled="!localConfigPath">
            <template #icon>
              <n-icon :component="RefreshOutline" />
            </template>
            加载
          </n-button>
          <n-button @click="handleFormatEntireConfig" :loading="isFormatting" :disabled="!localConfigPath || configStore.loading">
            <template #icon>
              <n-icon :component="CodeSlashOutline" />
            </template>
            格式化
          </n-button>
          <n-button @click="openConfigFile" :disabled="!localConfigPath">
            <template #icon>
              <n-icon :component="OpenOutline" />
            </template>
            打开
          </n-button>
        </div>

        <!-- 搜索和统计 -->
        <n-space align="center" justify="space-between" v-if="configStore.hasConfig">
          <n-space>
            <n-input v-model:value="configStore.searchQuery" placeholder="搜索端口号、域名..." style="width: 300px"
              clearable>
              <template #prefix>
                <n-icon :component="SearchOutline" />
              </template>
            </n-input>

            <n-button type="primary" @click="openAddServerModal">
              <template #icon>
                <n-icon :component="AddOutline" />
              </template>
              新增 Server
            </n-button>

            <n-button type="info" @click="handleReloadConfig" :loading="nginxStore.isLoading" :disabled="!settingsStore.settings.nginxPath || !nginxStore.status.isRunning">
              <template #icon>
                <n-icon :component="ReloadOutline" />
              </template>
              重载配置
            </n-button>
          </n-space>

          <n-space>
            <n-tag type="info">
              <template #icon>
                <n-icon :component="ServerOutline" />
              </template>
              {{ configStore.serverCount }} 个 Server
            </n-tag>
            <n-tag type="success">
              <template #icon>
                <n-icon :component="LocationOutline" />
              </template>
              {{ configStore.locationCount }} 个 Location
            </n-tag>
          </n-space>
        </n-space>
      </n-space>
    </n-card>

    <!-- 错误提示 -->
    <n-alert v-if="configStore.error" type="error" :title="configStore.error" closable @close="configStore.error = null"
      style="margin-top: 16px" />

    <!-- Server 列表 -->
    <div v-if="configStore.hasConfig" class="server-list">
      <n-empty v-if="configStore.filteredServers.length === 0" description="没有找到匹配的配置" style="margin-top: 60px" />

      <n-card v-for="server in configStore.filteredServers" :key="server.id" :bordered="true" class="server-card"
        hoverable @click="selectServer(server.id)" :class="{ selected: configStore.selectedServerId === server.id }">
        <template #header>
          <n-space align="center">
            <n-icon :component="ServerOutline" size="20" color="#18a058" />
            <n-text strong>Server Block</n-text>
            <n-tag size="small" type="info">
              行 {{ server.startLine }} - {{ server.endLine }}
            </n-tag>
          </n-space>
        </template>

        <n-space vertical :size="12">
          <!-- Listen 端口 -->
          <n-space align="center">
            <n-text strong style="width: 100px">监听端口:</n-text>
            <n-space>
              <n-tag v-for="(listen, index) in (server.listen || [])" :key="index" type="primary" size="small">
                {{ listen }}
              </n-tag>
              <n-tag v-if="!server.listen || server.listen.length === 0" type="default" size="small">
                未配置
              </n-tag>
            </n-space>
          </n-space>

          <!-- Server Name -->
          <n-space align="center">
            <n-text strong style="width: 100px">域名:</n-text>
            <n-space>
              <n-tag v-for="(name, index) in (server.serverName || [])" :key="index" type="success" size="small">
                {{ name }}
              </n-tag>
              <n-tag v-if="!server.serverName || server.serverName.length === 0" type="default" size="small">
                未配置
              </n-tag>
            </n-space>
          </n-space>

          <!-- Locations 列表 -->
          <n-space align="center">
            <n-text strong>location:

              <n-input v-if="server.locations && server.locations.length > 0"
                v-model:value="locationSearchQueries[server.id]" placeholder="搜索" size="tiny" style="width: 150px"
                clearable>
                <template #prefix>
                  <n-icon :component="SearchOutline" />
                </template>
              </n-input>
            </n-text>
            <n-space v-if="getFilteredLocations(server).length > 0">
              <n-popover
                v-for="location in getFilteredLocations(server)"
                :key="location.id"
                trigger="hover"
                placement="top"
              >
                <template #trigger>
                  <n-tag
                    type="info"
                    size="small"
                    :bordered="false"
                  >
                    <template #icon>
                      <n-icon :component="LocationOutline" />
                    </template>
                    {{ location.modifier || '' }} {{ location.path }}
                  </n-tag>
                </template>
                <pre :style="{
                  margin: 0,
                  whiteSpace: 'pre',
                  overflowX: 'auto',
                  overflowY: 'auto',
                  maxWidth: '800px',
                  maxHeight: '400px',
                  padding: '12px',
                  background: 'var(--n-color)',
                  color: 'var(--n-text-color)',
                  borderRadius: '4px',
                  border: '1px solid var(--n-border-color)'
                }">{{ location.rawContent }}</pre>
              </n-popover>
            </n-space>
          </n-space>
        </n-space>

        <template #action>
          <n-space>
            <n-button size="small" type="primary" @click.stop="openDetailModal(server)">
              详情
            </n-button>
            <n-button size="small" @click.stop="openEditServerModal(server)">
              编辑
            </n-button>
            <n-button size="small" type="error" @click.stop="handleDeleteServer(server.id)">
              删除
            </n-button>
          </n-space>
        </template>
      </n-card>
    </div>

    <!-- 空状态 -->
    <n-empty v-else-if="!configStore.loading" size="large" description="请加载配置文件" style="margin-top: 150px">
      <template #icon>
        <n-icon :component="DocumentTextOutline" />
      </template>
    </n-empty>

    <!-- Server 详情弹窗 -->
    <n-modal v-model:show="showDetailModal" preset="card" title="Server 配置详情" style="width: 800px; max-height: 85vh" :bordered="false">
      <pre :style="{
        whiteSpace: 'pre-wrap',
        wordWrap: 'break-word',
        background: 'var(--n-color)',
        color: 'var(--n-text-color)',
        padding: '16px',
        borderRadius: '4px',
        overflowY: 'auto',
        maxHeight: 'calc(85vh - 150px)',
        border: '1px solid var(--n-border-color)',
        fontFamily: 'Consolas, PingFang SC, monospace'
      }">{{ detailContent }}</pre>
    </n-modal>

    <!-- 文本编辑器弹窗 -->
    <n-modal
      v-model:show="showEditorModal"
      preset="card"
      :title="editorModalTitle"
      style="width: 900px; max-height: 90vh"
      :bordered="false"
    >
      <vue-monaco-editor
        v-model:value="editorContent"
        language="nginx"
        :options="{
          theme: 'vs-dark',
          minimap: { enabled: false },
          fontSize: 14,
          fontFamily: 'Consolas, PingFang SC, monospace',
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
          automaticLayout: true,
        }"
        :style="{
          height: 'calc(85vh - 150px)',
          border: '1px solid var(--n-border-color)',
          borderRadius: '4px',
          minHeight: '400px',
          maxHeight: '600px'
        }"
      />

      <template #footer>
        <n-space justify="space-between">
          <n-button secondary @click="handleFormatConfig">
            <template #icon>
              <n-icon><code-slash-outline /></n-icon>
            </template>
            格式化
          </n-button>
          <n-space>
            <n-button @click="showEditorModal = false">取消</n-button>
            <n-button type="primary" @click="handleSaveEditor">保存</n-button>
          </n-space>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  useMessage,
  useDialog,
  NCard,
  NSpace,
  NText,
  NInput,
  NButton,
  NIcon,
  NTag,
  NAlert,
  NEmpty,
  NModal,
  NPopover,
} from 'naive-ui';
import { VueMonacoEditor } from '@guolao/vue-monaco-editor';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '@/stores/config';
import { useSettingsStore } from '@/stores/settings';
import type { ServerBlock } from '@/types/config';
import {
  FolderOpenOutline,
  RefreshOutline,
  SearchOutline,
  ServerOutline,
  DocumentTextOutline,
  LocationOutline,
  AddOutline,
  OpenOutline,
  CodeSlashOutline,
  ReloadOutline,
} from '@vicons/ionicons5';
import { formatNginxConfig } from '@/utils/nginxFormatter';
import { useLogStore } from '@/stores/log';
import { useNginxStore } from '@/stores/nginx';

const message = useMessage();
const dialog = useDialog();
const configStore = useConfigStore();
const settingsStore = useSettingsStore();
const logStore = useLogStore();
const nginxStore = useNginxStore();

const localConfigPath = ref('');

// 卡片内 Location 搜索（每个 server 一个搜索框）
const locationSearchQueries = ref<Record<string, string>>({});

// 文本编辑器相关状态
const showEditorModal = ref(false);
const editorModalTitle = ref('');
const editorContent = ref('');
const editorMode = ref<'add' | 'edit'>('add');
const editingServerId = ref('');

// Server 详情弹窗
const showDetailModal = ref(false);
const detailContent = ref('');

// 格式化整个配置文件的加载状态
const isFormatting = ref(false);

// 初始化时加载配置路径
onMounted(async () => {
  // 先加载 settings
  await settingsStore.loadSettings();

  // 从 settings 中获取配置文件路径
  localConfigPath.value = settingsStore.settings.configPath;

  // 如果有配置路径，自动加载
  if (localConfigPath.value) {
    loadConfig();
  }
});

// 选择配置文件
const selectConfigFile = async () => {
  try {
    const selected = await open({
      title: '选择 Nginx 配置文件',
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'Nginx 配置文件',
          extensions: ['conf'],
        },
      ],
    });

    if (selected) {
      localConfigPath.value = selected as string;
      message.success('已选择配置文件');
    }
  } catch (error) {
    message.error('选择文件失败: ' + error);
  }
};

// 加载配置
const loadConfig = async (showTips = false) => {
  if (!localConfigPath.value) {
    message.warning('请先选择配置文件');
    return;
  }

  const result = await configStore.loadConfig(localConfigPath.value);

  if (result.success) {
    // 保存配置路径到设置
    settingsStore.updateConfigPath(localConfigPath.value);

    if (showTips) {
      message.success('配置文件加载成功');
    }
  } else {
    message.error(result.message || '配置文件加载失败');
  }
};

// 打开配置文件
const openConfigFile = async () => {
  if (!localConfigPath.value) {
    message.warning('请先选择配置文件');
    return;
  }

  try {
    const result = await invoke<string>('open_file_in_system', {
      filePath: localConfigPath.value,
    });
    message.success(result);
  } catch (error) {
    message.error(`打开文件失败: ${error}`);
  }
};

// 重载配置
const handleReloadConfig = async () => {
  const nginxPath = settingsStore.settings.nginxPath;
  if (!nginxPath) {
    message.warning('请先在设置中配置 Nginx 路径');
    return;
  }

  try {
    logStore.info('正在重载配置...');
    const result = await nginxStore.reload(nginxPath);

    if (result?.success) {
      message.success(result.message);
      logStore.success(result.message);
    } else {
      message.error(result?.message || '重载失败');
      logStore.error(result?.message || '重载失败');
    }
  } catch (error) {
    message.error('重载失败');
    logStore.error(`重载失败: ${error}`);
  }
};

// 选择 server
const selectServer = (serverId: string) => {
  configStore.selectServer(serverId);
};

// 获取过滤后的 locations（卡片内搜索）
const getFilteredLocations = (server: ServerBlock) => {
  const query = locationSearchQueries.value[server.id];
  if (!query || !server.locations) {
    return server.locations || [];
  }

  const lowerQuery = query.toLowerCase();
  return server.locations.filter(location =>
    location.path.toLowerCase().includes(lowerQuery) ||
    (location.modifier && location.modifier.toLowerCase().includes(lowerQuery))
  );
};

// ==================== 编辑功能 ====================

// 打开新增 Server 弹窗（文本编辑器）
const openAddServerModal = () => {
  editorMode.value = 'add';
  editorModalTitle.value = '新增 Server';
  editorContent.value = `server {
    listen 80;
    server_name localhost;

    location / {
        root html;
        index index.html;
    }
}`;
  editingServerId.value = '';
  showEditorModal.value = true;
};

// 打开编辑 Server 弹窗（文本编辑器）
const openEditServerModal = (server: ServerBlock) => {
  editorMode.value = 'edit';
  editorModalTitle.value = '编辑 Server';
  editorContent.value = server.rawContent;
  editingServerId.value = server.id;
  showEditorModal.value = true;
};

// 打开 Server 详情弹窗
const openDetailModal = (server: ServerBlock) => {
  detailContent.value = server.rawContent;
  showDetailModal.value = true;
};

// 删除 Server
const handleDeleteServer = (serverId: string) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除这个 Server 块吗？此操作不可撤销。',
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        const result = await invoke<{ success: boolean; message: string }>('delete_server_block', {
          configPath: localConfigPath.value,
          serverId,
        });

        if (!result.success) {
          message.error(result.message);
          return;
        }

        // 删除成功后，进行配置校验
        message.loading('正在校验配置...', { duration: 0 });

        const testResult = await invoke<{ success: boolean; message: string }>('test_nginx_config_file', {
          nginxPath: settingsStore.settings.nginxPath,
          configPath: localConfigPath.value,
        });

        message.destroyAll();

        if (testResult.success) {
          message.success('删除成功，配置校验通过');
          logStore.success('删除 Server 配置成功');
          await configStore.loadConfig(localConfigPath.value); // 静默加载
        } else {
          // 配置校验失败
          logStore.warning(`删除 Server 后配置校验失败: ${testResult.message}`);

          dialog.error({
            title: '配置校验失败',
            content: testResult.message,
            positiveText: '知道了',
            style: {
              width: '600px',
            },
          });

          message.warning('删除成功但配置校验失败，请检查配置');
          await configStore.loadConfig(localConfigPath.value); // 静默加载
        }
      } catch (error) {
        message.destroyAll();
        message.error(`删除失败: ${error}`);
        logStore.error(`删除 Server 失败: ${error}`);
      }
    },
  });
};

// 保存编辑器内容 - 先校验再保存
const handleSaveEditor = async () => {
  if (!editorContent.value.trim()) {
    message.error('配置内容不能为空');
    return;
  }

  let tempFilePath = '';

  try {
    // 步骤 1: 生成新配置内容（不保存）
    message.loading('正在生成配置...', { duration: 0 });

    let newContent: string;
    if (editorMode.value === 'add') {
      // 生成添加 server 后的新内容
      const result = await invoke<string>('generate_add_server_content', {
        configPath: localConfigPath.value,
        serverText: editorContent.value,
      });
      newContent = result;
    } else {
      // 生成更新 server 后的新内容
      const result = await invoke<string>('generate_update_server_content', {
        configPath: localConfigPath.value,
        serverId: editingServerId.value,
        serverText: editorContent.value,
      });
      newContent = result;
    }

    // 步骤 2: 写入临时文件
    tempFilePath = await invoke<string>('write_temp_config_for_validation', {
      originalConfigPath: localConfigPath.value,
      newContent: newContent,
    });

    // 步骤 3: 校验临时文件
    message.destroyAll();
    message.loading('正在校验配置...', { duration: 0 });

    const testResult = await invoke<{ success: boolean; message: string }>('test_nginx_config_file', {
      nginxPath: settingsStore.settings.nginxPath,
      configPath: tempFilePath,
    });

    message.destroyAll();

    if (!testResult.success) {
      // 校验失败，不保存配置
      logStore.error(`配置校验失败，未保存配置: ${testResult.message}`);

      dialog.error({
        title: '配置校验失败',
        content: `配置校验未通过，未保存到文件。\n\n${testResult.message}`,
        positiveText: '知道了',
        style: {
          width: '600px',
        },
      });
      return;
    }

    // 步骤 4: 校验通过，保存配置
    let saveResult;
    if (editorMode.value === 'add') {
      saveResult = await invoke<{ success: boolean; message: string }>('add_server_block_text', {
        configPath: localConfigPath.value,
        serverText: editorContent.value,
      });
    } else {
      saveResult = await invoke<{ success: boolean; message: string }>('update_server_block_text', {
        configPath: localConfigPath.value,
        serverId: editingServerId.value,
        serverText: editorContent.value,
      });
    }

    if (!saveResult.success) {
      message.error(saveResult.message);
      logStore.error(`保存配置失败: ${saveResult.message}`);
      return;
    }

    // 步骤 5: 保存成功
    const actionText = editorMode.value === 'add' ? '新增 Server 配置成功' : '更新 Server 配置成功';
    message.success('配置校验通过，已保存');
    logStore.success(actionText);

    showEditorModal.value = false;
    await configStore.loadConfig(localConfigPath.value); // 静默加载
  } catch (error) {
    message.destroyAll();
    message.error(`操作失败: ${error}`);
    logStore.error(`保存配置失败: ${error}`);
  } finally {
    // 清理临时文件
    if (tempFilePath) {
      try {
        await invoke('delete_temp_config', { tempPath: tempFilePath });
      } catch (e) {
        console.error('删除临时文件失败:', e);
      }
    }
  }
};

// 格式化编辑器中的配置
const handleFormatConfig = () => {
  try {
    if (!editorContent.value.trim()) {
      message.warning('配置内容为空，无需格式化');
      return;
    }

    const formattedContent = formatNginxConfig(editorContent.value);
    editorContent.value = formattedContent;
    message.success('格式化成功');
  } catch (error) {
    console.error('格式化失败:', error);
    message.error(`格式化失败: ${error instanceof Error ? error.message : String(error)}`);
  }
};

// 格式化整个配置文件
const handleFormatEntireConfig = async () => {
  if (!localConfigPath.value) {
    message.warning('请先选择配置文件');
    return;
  }

  isFormatting.value = true;
  let tempFilePath = '';

  try {
    // 步骤 1: 读取配置文件
    logStore.info(`开始格式化配置文件: ${localConfigPath.value}`);
    message.loading('正在读取配置文件...', { duration: 0 });

    const content = await invoke<string>('read_config_file_content', {
      configPath: localConfigPath.value,
    });

    // 步骤 2: 格式化配置
    message.destroyAll();
    message.loading('正在格式化配置...', { duration: 0 });

    const formattedContent = formatNginxConfig(content);

    // 步骤 3: 写入临时文件并校验
    message.destroyAll();
    message.loading('正在校验配置...', { duration: 0 });

    tempFilePath = await invoke<string>('write_temp_config_for_validation', {
      originalConfigPath: localConfigPath.value,
      newContent: formattedContent,
    });

    const testResult = await invoke<{ success: boolean; message: string }>('test_nginx_config_file', {
      nginxPath: settingsStore.settings.nginxPath,
      configPath: tempFilePath,
    });

    message.destroyAll();

    if (!testResult.success) {
      // 校验失败，不保存
      logStore.warning(`配置文件格式化完成，但校验失败: ${testResult.message}`);

      dialog.error({
        title: '配置校验失败',
        content: `格式化后的配置校验未通过，未保存到文件。\n\n${testResult.message}`,
        positiveText: '知道了',
        style: {
          width: '600px',
        },
      });
      return;
    }

    // 步骤 4: 校验通过，写入文件
    await invoke<{ success: boolean; message: string }>('write_formatted_config', {
      configPath: localConfigPath.value,
      formattedContent: formattedContent,
    });

    // 步骤 5: 成功提示
    message.success('配置文件格式化成功，校验通过');
    logStore.success('配置文件格式化成功');

    // 步骤 6: 重新加载配置（静默）
    await configStore.loadConfig(localConfigPath.value);
  } catch (error) {
    message.destroyAll();
    message.error(`格式化失败: ${error}`);
    logStore.error(`格式化配置文件失败: ${error}`);
  } finally {
    isFormatting.value = false;

    // 清理临时文件
    if (tempFilePath) {
      try {
        await invoke('delete_temp_config', { tempPath: tempFilePath });
      } catch (e) {
        console.error('删除临时文件失败:', e);
      }
    }
  }
};
</script>

<style scoped>
.config-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.toolbar-card {
  margin-bottom: 16px;
}

.config-input-group {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.server-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 4px 0;
}

.server-card {
  transition: all 0.3s ease;
}

.server-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.server-card.selected {
  border-color: #18a058;
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.2);
}
</style>
