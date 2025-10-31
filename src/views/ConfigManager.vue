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
          <n-button type="primary" @click="loadConfig" :loading="configStore.loading" :disabled="!localConfigPath">
            <template #icon>
              <n-icon :component="RefreshOutline" />
            </template>
            加载
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
            <n-input v-model:value="configStore.searchQuery" placeholder="搜索端口号、域名或 Location 路径" style="width: 300px"
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
          <n-space align="center" v-if="getFilteredLocations(server).length > 0">
            <n-text strong style="width: 100px">Location:

              <n-input v-if="server.locations && server.locations.length > 0"
                v-model:value="locationSearchQueries[server.id]" placeholder="搜索" size="tiny" style="width: 150px"
                clearable>
                <template #prefix>
                  <n-icon :component="SearchOutline" />
                </template>
              </n-input>
            </n-text>
            <n-space>
              <n-popover
                v-for="location in getFilteredLocations(server)"
                :key="location.id"
                trigger="hover"
                placement="top"
              >
                <template #trigger>
                  <n-tag
                    type="warning"
                    size="small"
                  >
                    <template #icon>
                      <n-icon :component="LocationOutline" />
                    </template>
                    {{ location.modifier || '' }} {{ location.path }}
                  </n-tag>
                </template>
                <pre style="margin: 0; white-space: pre; overflow-x: auto; overflow-y: auto; max-width: 800px; max-height: 400px; padding: 8px; background: #f5f5f5; border-radius: 4px;">{{ location.rawContent }}</pre>
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
      <pre style="white-space: pre-wrap; word-wrap: break-word; background: #f5f5f5; padding: 16px; border-radius: 4px; overflow-y: auto; max-height: calc(85vh - 150px);">{{ detailContent }}</pre>
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
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
          automaticLayout: true,
        }"
        :style="{ height: 'calc(85vh - 150px)', border: '1px solid #d9d9d9', borderRadius: '4px', minHeight: '400px', maxHeight: '600px' }"
      />

      <template #footer>
        <n-space justify="end">
          <n-button @click="showEditorModal = false">取消</n-button>
          <n-button type="primary" @click="handleSaveEditor">保存</n-button>
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
} from '@vicons/ionicons5';

const message = useMessage();
const dialog = useDialog();
const configStore = useConfigStore();
const settingsStore = useSettingsStore();

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
const loadConfig = async () => {
  if (!localConfigPath.value) {
    message.warning('请先选择配置文件');
    return;
  }

  const success = await configStore.loadConfig(localConfigPath.value);
  if (success) {
    message.success('配置文件加载成功');
    // 保存配置路径到设置
    settingsStore.updateConfigPath(localConfigPath.value);

    // 调试：打印加载的配置数据
    console.log('=== 配置加载成功 ===');
    console.log('Server 总数:', configStore.serverCount);
    console.log('所有 Servers:', configStore.servers);

    // 打印前3个 server 的详细信息
    configStore.servers.slice(0, 3).forEach((server, index) => {
      console.log(`\n--- Server ${index + 1} ---`);
      console.log('ID:', server.id);
      console.log('行数:', server.startLine, '-', server.endLine);
      console.log('监听端口:', server.listen);
      console.log('域名:', server.serverName);
      console.log('Locations:', server.locations);
    });
  } else {
    message.error(configStore.error || '配置文件加载失败');
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

        if (result.success) {
          message.success(result.message);
          // 重新加载配置
          await configStore.loadConfig(localConfigPath.value);
        } else {
          message.error(result.message);
        }
      } catch (error) {
        message.error(`删除失败: ${error}`);
      }
    },
  });
};

// 保存编辑器内容
const handleSaveEditor = async () => {
  if (!editorContent.value.trim()) {
    message.error('配置内容不能为空');
    return;
  }

  try {
    if (editorMode.value === 'add') {
      // 新增 Server - 需要后端支持直接添加文本格式的 Server
      const result = await invoke<{ success: boolean; message: string }>('add_server_block_text', {
        configPath: localConfigPath.value,
        serverText: editorContent.value,
      });

      if (result.success) {
        message.success(result.message);
        showEditorModal.value = false;
        await configStore.loadConfig(localConfigPath.value);
      } else {
        message.error(result.message);
      }
    } else {
      // 编辑 Server - 需要后端支持直接更新文本格式的 Server
      const result = await invoke<{ success: boolean; message: string }>('update_server_block_text', {
        configPath: localConfigPath.value,
        serverId: editingServerId.value,
        serverText: editorContent.value,
      });

      if (result.success) {
        message.success(result.message);
        showEditorModal.value = false;
        await configStore.loadConfig(localConfigPath.value);
      } else {
        message.error(result.message);
      }
    }
  } catch (error) {
    message.error(`保存失败: ${error}`);
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
