<template>
  <div class="config-page">
    <section class="panel-card toolbar-panel">
      <div class="toolbar-scroll">
        <div class="action-row">
          <n-space wrap>
            <n-button
              type="primary"
              :loading="configStore.loading"
              :disabled="!localConfigPath"
              @click="loadConfig(true)"
            >
              <template #icon>
                <n-icon :component="RefreshOutline" />
              </template>
              重新加载
            </n-button>
            <n-button
              secondary
              :loading="isFormatting"
              :disabled="!localConfigPath || configStore.loading"
              @click="handleFormatEntireConfig"
            >
              <template #icon>
                <n-icon :component="CodeSlashOutline" />
              </template>
              格式化全文
            </n-button>
            <n-button :disabled="!localConfigPath" @click="openConfigFile">
              <template #icon>
                <n-icon :component="OpenOutline" />
              </template>
              打开文件
            </n-button>
          </n-space>
        </div>

        <div class="filter-row" v-if="configStore.hasConfig">
          <n-input
            v-model:value="configStore.searchQuery"
            clearable
            placeholder="搜索端口、域名、分类或 location 路径"
            class="filter-search"
          >
            <template #prefix>
              <n-icon :component="SearchOutline" />
            </template>
          </n-input>

          <n-select
            v-model:value="configStore.categoryFilter"
            clearable
            filterable
            :options="categoryOptions"
            placeholder="项目分类"
            class="filter-select"
          />

          <n-radio-group
            v-model:value="configStore.statusFilter"
            size="small"
            name="server-status-filter"
          >
            <n-radio-button value="all">全部</n-radio-button>
            <n-radio-button value="enabled">启用</n-radio-button>
            <n-radio-button value="disabled">停用</n-radio-button>
          </n-radio-group>

          <n-space wrap>
            <n-button type="primary" @click="openAddServerModal">
              <template #icon>
                <n-icon :component="AddOutline" />
              </template>
              新增 Server
            </n-button>
            <n-button
              secondary
              :disabled="
                !settingsStore.settings.nginxPath ||
                !nginxStore.status.isRunning
              "
              :loading="nginxStore.isLoading"
              @click="handleReloadConfig"
            >
              <template #icon>
                <n-icon :component="ReloadOutline" />
              </template>
              重载配置
            </n-button>
            <n-button v-if="hasActiveFilters" quaternary @click="resetFilters"
              >清空筛选</n-button
            >
          </n-space>
        </div>

        <div class="stats-row" v-if="configStore.hasConfig">
          <div class="stat-chip">
            <span class="stat-label">Server</span>
            <strong>{{ configStore.serverCount }}</strong>
          </div>
          <div class="stat-chip">
            <span class="stat-label">启用</span>
            <strong>{{ configStore.enabledServerCount }}</strong>
          </div>
          <div class="stat-chip">
            <span class="stat-label">停用</span>
            <strong>{{ configStore.disabledServerCount }}</strong>
          </div>
          <div class="stat-chip" v-if="configStore.categoryFilter">
            <span class="stat-label">当前分类</span>
            <strong>{{ configStore.categoryFilter }}</strong>
          </div>
        </div>

        <n-alert
          v-if="configStore.error"
          type="error"
          :title="configStore.error"
          closable
          class="inline-alert"
          @close="configStore.error = null"
        />
      </div>
    </section>

    <section class="panel-card list-panel">
      <div class="list-panel-header">
        <div>
          <h3>Server 列表</h3>
          <p v-if="configStore.hasConfig">
            已根据当前配置完成解析，可继续筛选、查看或编辑。
          </p>
          <p v-else>如果已配置默认路径，页面会在进入时自动尝试加载。</p>
        </div>
        <n-tag round size="small" type="info">
          {{
            configStore.hasConfig
              ? `${configStore.filteredServers.length} 个结果`
              : "等待加载配置"
          }}
        </n-tag>
      </div>

      <div class="list-panel-content">
        <n-spin
          :show="configStore.loading"
          description="正在分析配置文件..."
          class="list-panel-body"
        >
          <div class="server-scroll">
            <template v-if="configStore.hasConfig">
              <n-empty
                v-if="configStore.filteredServers.length === 0"
                description="没有找到匹配的 Server 配置"
                class="page-empty"
              />
              <ServerSummaryCard
                v-for="server in configStore.filteredServers"
                :key="server.id"
                :server="server"
                :selected="configStore.selectedServerId === server.id"
                @select="configStore.selectServer(server.id)"
                @detail="openDetailModal(server)"
                @edit="openEditServerModal(server)"
                @toggle="handleToggleServerState(server)"
                @delete="handleDeleteServer(server.id)"
              />
            </template>
            <n-empty v-else description="请先加载配置文件" class="page-empty">
              <template #icon>
                <n-icon :component="DocumentTextOutline" />
              </template>
            </n-empty>
          </div>
        </n-spin>
      </div>
    </section>

    <ServerCodeModal
      v-model:show="showCodeModal"
      :title="codeModalTitle"
      :content="editorContent"
      :locations="codeModalLocations"
      :read-only="codeModalReadOnly"
      :category-name="categoryName"
      :category-mode="codeModalCategoryMode"
      @update:content="editorContent = $event"
      @update:categoryName="categoryName = $event"
      @save="handleSaveEditor"
      @format="handleFormatConfig"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  NAlert,
  NButton,
  NEmpty,
  NIcon,
  NInput,
  NRadioButton,
  NRadioGroup,
  NSelect,
  NSpace,
  NSpin,
  NTag,
  useDialog,
  useMessage,
} from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import {
  AddOutline,
  CodeSlashOutline,
  DocumentTextOutline,
  OpenOutline,
  RefreshOutline,
  ReloadOutline,
  SearchOutline,
} from "@vicons/ionicons5";
import ServerCodeModal from "@/components/config/ServerCodeModal.vue";
import ServerSummaryCard from "@/components/config/ServerSummaryCard.vue";
import { eventBus, EVENTS } from "@/composables/useEventBus";
import { useConfigStore } from "@/stores/config";
import { useLogStore } from "@/stores/log";
import { useNginxStore } from "@/stores/nginx";
import { useSettingsStore } from "@/stores/settings";
import type { ServerBlock } from "@/types/config";
import {
  applyServerCategoryToContent,
  getCategoryLineDelta,
} from "@/utils/nginxCategory";
import {
  formatNginxConfig,
  formatNginxServerBlock,
} from "@/utils/nginxFormatter";

const message = useMessage();
const dialog = useDialog();
const configStore = useConfigStore();
const settingsStore = useSettingsStore();
const logStore = useLogStore();
const nginxStore = useNginxStore();

const localConfigPath = ref("");
const showCodeModal = ref(false);
const editorContent = ref("");
const editorMode = ref<"add" | "edit" | "detail">("detail");
const editingServer = ref<ServerBlock | null>(null);
const categoryName = ref("");
const originalCategoryName = ref("");
const isFormatting = ref(false);
const pendingAutoLoadPath = ref<string | null>(null);

const emitConfigOperationResult = (
  level: "success" | "warning" | "error" | "info",
  messageText: string,
) => {
  eventBus.emit(EVENTS.CONFIG_OPERATION_RESULT, {
    level,
    message: messageText,
    operation: "toggle-server-state",
  });
};

const categoryOptions = computed(() =>
  configStore.availableCategories.map((category) => ({
    label: category,
    value: category,
  })),
);

const hasActiveFilters = computed(() =>
  Boolean(
    configStore.searchQuery ||
    configStore.categoryFilter ||
    configStore.statusFilter !== "all",
  ),
);

const codeModalTitle = computed(() => {
  if (editorMode.value === "add") {
    return "新增 Server";
  }
  if (editorMode.value === "edit") {
    return editingServer.value?.enabled ? "编辑 Server" : "编辑已停用 Server";
  }
  return editingServer.value?.category || "Server 详情";
});

const codeModalReadOnly = computed(() => editorMode.value === "detail");
const codeModalCategoryMode = computed<"hidden" | "display" | "edit">(() => {
  if (editorMode.value === "detail") {
    return "display";
  }

  if (editorMode.value === "edit") {
    return "edit";
  }

  return "hidden";
});

const codeModalLocations = computed(() => {
  const locations = editingServer.value?.locations ?? [];
  if (editorMode.value !== "edit") {
    return locations;
  }

  const delta = getCategoryLineDelta(
    originalCategoryName.value,
    categoryName.value,
  );
  if (!delta) {
    return locations;
  }

  return locations.map((location) => ({
    ...location,
    relativeStartLine: Math.max(1, location.relativeStartLine + delta),
    relativeEndLine: Math.max(1, location.relativeEndLine + delta),
  }));
});

const ensureSettingsLoaded = async () => {
  if (!settingsStore.isLoaded) {
    await settingsStore.loadSettings();
  }
};

const syncConfigPathFromSettings = async (path: string) => {
  if (!path) {
    return;
  }

  localConfigPath.value = path;

  if (configStore.configPath === path || pendingAutoLoadPath.value === path) {
    return;
  }

  pendingAutoLoadPath.value = path;
  try {
    await loadConfig(false, path);
  } finally {
    if (pendingAutoLoadPath.value === path) {
      pendingAutoLoadPath.value = null;
    }
  }
};

const tryAutoLoadConfig = async () => {
  await ensureSettingsLoaded();
  const savedPath = settingsStore.settings.configPath;
  if (!savedPath) {
    return;
  }
  await syncConfigPathFromSettings(savedPath);
};

onMounted(() => {
  void tryAutoLoadConfig();
});

watch(
  () => settingsStore.settings.configPath,
  (path) => {
    if (!path) {
      return;
    }
    void syncConfigPathFromSettings(path);
  },
);

const ensureNginxPathConfigured = () => {
  if (settingsStore.settings.nginxPath) {
    return true;
  }

  message.warning("请先在应用设置中配置 Nginx 路径");
  return false;
};

const resetFilters = () => {
  configStore.searchQuery = "";
  configStore.statusFilter = "all";
  configStore.categoryFilter = null;
};

const loadConfig = async (showTips = false, path = localConfigPath.value) => {
  if (!path) {
    if (showTips) {
      message.warning("请先选择配置文件");
    }
    return;
  }

  localConfigPath.value = path;
  const result = await configStore.loadConfig(path);

  if (result.success) {
    if (settingsStore.settings.configPath !== path) {
      settingsStore.updateConfigPath(path);
    }
    if (showTips) {
      message.success("配置文件加载成功");
    }
  } else if (showTips) {
    message.error(result.message || "配置文件加载失败");
  }
};

const openConfigFile = async () => {
  if (!localConfigPath.value) {
    message.warning("请先选择配置文件");
    return;
  }

  try {
    const result = await invoke<string>("open_file_in_system", {
      filePath: localConfigPath.value,
    });
    message.success(result);
  } catch (error) {
    message.error(`打开文件失败: ${error}`);
  }
};

const handleReloadConfig = () => {
  const nginxPath = settingsStore.settings.nginxPath;
  if (!nginxPath) {
    message.warning("请先在应用设置中配置 Nginx 路径");
    return;
  }

  logStore.info("正在重载配置...");
  nginxStore.reload(nginxPath);
};

const openAddServerModal = () => {
  editorMode.value = "add";
  editingServer.value = null;
  categoryName.value = "";
  originalCategoryName.value = "";
  editorContent.value = `server {
    # 新项目分类
    listen 80;
    server_name localhost;

    location / {
        root html;
        index index.html;
    }
}`;
  showCodeModal.value = true;
};

const openEditServerModal = (server: ServerBlock) => {
  configStore.selectServer(server.id);
  editorMode.value = "edit";
  editingServer.value = server;
  categoryName.value = server.category?.trim() ?? "";
  originalCategoryName.value = server.category?.trim() ?? "";
  editorContent.value = server.rawContent;
  showCodeModal.value = true;
};

const openDetailModal = (server: ServerBlock) => {
  configStore.selectServer(server.id);
  editorMode.value = "detail";
  editingServer.value = server;
  categoryName.value = server.category?.trim() ?? "";
  originalCategoryName.value = server.category?.trim() ?? "";
  editorContent.value = server.rawContent;
  showCodeModal.value = true;
};

watch(categoryName, (value) => {
  if (editorMode.value !== "edit") {
    return;
  }

  editorContent.value = applyServerCategoryToContent(
    editorContent.value,
    value,
  );
});

const validateGeneratedConfig = async (newContent: string, title: string) => {
  if (!ensureNginxPathConfigured()) {
    return false;
  }

  let tempFilePath = "";

  try {
    message.loading("正在校验配置...", { duration: 0 });

    tempFilePath = await invoke<string>("write_temp_config_for_validation", {
      originalConfigPath: localConfigPath.value,
      newContent,
    });

    const testResult = await invoke<{ success: boolean; message: string }>(
      "test_nginx_config_file",
      {
        nginxPath: settingsStore.settings.nginxPath,
        configPath: tempFilePath,
      },
    );

    message.destroyAll();

    if (testResult.success) {
      return true;
    }

    logStore.warning(`${title}: ${testResult.message}`);
    dialog.error({
      title,
      content: testResult.message,
      positiveText: "知道了",
      style: { width: "640px" },
    });
    return false;
  } finally {
    if (tempFilePath) {
      await invoke("delete_temp_config", { tempPath: tempFilePath }).catch(
        () => undefined,
      );
    }
  }
};

const reloadNginxAfterConfigChange = async (
  actionText: string,
): Promise<{ level: "success" | "warning"; message: string }> => {
  const nginxPath = settingsStore.settings.nginxPath;
  if (!nginxPath || !nginxStore.status.isRunning) {
    return {
      level: "success",
      message: `${actionText}成功，Nginx 当前未运行，未执行重载`,
    };
  }

  const reloadResult = await invoke<{ success: boolean; message: string }>(
    "reload_nginx",
    {
      nginxPath,
    },
  );

  if (reloadResult.success) {
    await nginxStore.checkStatus().catch(() => undefined);
    return {
      level: "success",
      message: `${actionText}成功，已自动重载 Nginx`,
    };
  }

  return {
    level: "warning",
    message: `${actionText}成功，但 Nginx 重载失败：${reloadResult.message}`,
  };
};

const executeToggleServerState = async (
  server: ServerBlock,
  targetEnabled: boolean,
  actionText: string,
) => {
  try {
    message.loading(`正在${actionText}...`, { duration: 0 });
    const newContent = await invoke<string>(
      "generate_toggle_server_state_content",
      {
        configPath: localConfigPath.value,
        serverId: server.id,
        enabled: targetEnabled,
      },
    );

    message.destroyAll();
    const valid = await validateGeneratedConfig(
      newContent,
      `${actionText}失败，配置校验未通过`,
    );
    if (!valid) {
      return;
    }

    const result = await invoke<{ success: boolean; message: string }>(
      "set_server_enabled_state",
      {
        configPath: localConfigPath.value,
        serverId: server.id,
        enabled: targetEnabled,
      },
    );

    if (!result.success) {
      emitConfigOperationResult("error", `${actionText}失败：${result.message}`);
      return;
    }

    await configStore.loadConfig(localConfigPath.value);
    const notice = await reloadNginxAfterConfigChange(actionText);
    emitConfigOperationResult(notice.level, notice.message);
  } catch (error) {
    message.destroyAll();
    emitConfigOperationResult("error", `${actionText}失败：${error}`);
  }
};

const handleToggleServerState = (server: ServerBlock) => {
  const targetEnabled = !server.enabled;
  const actionText = targetEnabled ? "恢复 Server" : "停用 Server";

  dialog.warning({
    title: `确认${actionText}`,
    content: targetEnabled
      ? "恢复后会重新生效对应 listen 端口与 server 配置，并在 Nginx 运行时自动尝试重载。"
      : "停用后会将该 Server 块整段注释保留，并在 Nginx 运行时自动尝试重载。",
    positiveText: targetEnabled ? "恢复启用" : "临时停用",
    negativeText: "取消",
    onPositiveClick: () => {
      void executeToggleServerState(server, targetEnabled, actionText);
    },
  });
};

const handleDeleteServer = (serverId: string) => {
  dialog.warning({
    title: "确认删除",
    content: "确定要删除这个 Server 块吗？此操作不可撤销。",
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        const result = await invoke<{ success: boolean; message: string }>(
          "delete_server_block",
          {
            configPath: localConfigPath.value,
            serverId,
          },
        );

        if (!result.success) {
          message.error(result.message);
          return;
        }

        if (!settingsStore.settings.nginxPath) {
          message.success("删除成功，但尚未配置 Nginx 路径，未执行校验");
          logStore.success("删除 Server 配置成功");
          await configStore.loadConfig(localConfigPath.value);
          return;
        }

        message.loading("正在校验配置...", { duration: 0 });
        const testResult = await invoke<{ success: boolean; message: string }>(
          "test_nginx_config_file",
          {
            nginxPath: settingsStore.settings.nginxPath,
            configPath: localConfigPath.value,
          },
        );

        message.destroyAll();

        if (testResult.success) {
          message.success("删除成功，配置校验通过");
          logStore.success("删除 Server 配置成功");
        } else {
          logStore.warning(`删除 Server 后配置校验失败: ${testResult.message}`);
          dialog.error({
            title: "配置校验失败",
            content: testResult.message,
            positiveText: "知道了",
            style: { width: "640px" },
          });
          message.warning("删除成功但配置校验失败，请检查配置");
        }

        await configStore.loadConfig(localConfigPath.value);
      } catch (error) {
        message.destroyAll();
        message.error(`删除失败: ${error}`);
        logStore.error(`删除 Server 失败: ${error}`);
      }
    },
  });
};

const handleSaveEditor = async () => {
  if (editorMode.value === "detail") {
    showCodeModal.value = false;
    return;
  }

  if (!editorContent.value.trim()) {
    message.error("配置内容不能为空");
    return;
  }

  if (!ensureNginxPathConfigured()) {
    return;
  }

  let tempFilePath = "";
  const nextEditorContent =
    editorMode.value === "edit"
      ? applyServerCategoryToContent(editorContent.value, categoryName.value)
      : editorContent.value;

  try {
    message.loading("正在生成配置...", { duration: 0 });

    const newContent =
      editorMode.value === "add"
        ? await invoke<string>("generate_add_server_content", {
            configPath: localConfigPath.value,
            serverText: nextEditorContent,
          })
        : await invoke<string>("generate_update_server_content", {
            configPath: localConfigPath.value,
            serverId: editingServer.value?.id,
            serverText: nextEditorContent,
          });

    tempFilePath = await invoke<string>("write_temp_config_for_validation", {
      originalConfigPath: localConfigPath.value,
      newContent,
    });

    message.destroyAll();
    message.loading("正在校验配置...", { duration: 0 });

    const testResult = await invoke<{ success: boolean; message: string }>(
      "test_nginx_config_file",
      {
        nginxPath: settingsStore.settings.nginxPath,
        configPath: tempFilePath,
      },
    );

    message.destroyAll();

    if (!testResult.success) {
      logStore.error(`配置校验失败，未保存配置: ${testResult.message}`);
      dialog.error({
        title: "配置校验失败",
        content: `配置校验未通过，未保存到文件。\n\n${testResult.message}`,
        positiveText: "知道了",
        style: { width: "640px" },
      });
      return;
    }

    const saveResult =
      editorMode.value === "add"
        ? await invoke<{ success: boolean; message: string }>(
            "add_server_block_text",
            {
              configPath: localConfigPath.value,
              serverText: nextEditorContent,
            },
          )
        : await invoke<{ success: boolean; message: string }>(
            "update_server_block_text",
            {
              configPath: localConfigPath.value,
              serverId: editingServer.value?.id,
              serverText: nextEditorContent,
            },
          );

    if (!saveResult.success) {
      message.error(saveResult.message);
      logStore.error(`保存配置失败: ${saveResult.message}`);
      return;
    }

    const successText =
      editorMode.value === "add"
        ? "新增 Server 配置成功"
        : "更新 Server 配置成功";
    editorContent.value = nextEditorContent;
    message.success(successText);
    logStore.success(successText);

    showCodeModal.value = false;
    await configStore.loadConfig(localConfigPath.value);
  } catch (error) {
    message.destroyAll();
    message.error(`操作失败: ${error}`);
    logStore.error(`保存配置失败: ${error}`);
  } finally {
    if (tempFilePath) {
      await invoke("delete_temp_config", { tempPath: tempFilePath }).catch(
        () => undefined,
      );
    }
  }
};

const handleFormatConfig = () => {
  try {
    if (!editorContent.value.trim()) {
      message.warning("配置内容为空，无需格式化");
      return;
    }

    editorContent.value = formatNginxServerBlock(editorContent.value);
    message.success("格式化成功");
  } catch (error) {
    message.error(
      `格式化失败: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
};

const handleFormatEntireConfig = async () => {
  if (!localConfigPath.value) {
    message.warning("请先选择配置文件");
    return;
  }

  if (!ensureNginxPathConfigured()) {
    return;
  }

  isFormatting.value = true;
  let tempFilePath = "";

  try {
    logStore.info(`开始格式化配置文件: ${localConfigPath.value}`);
    message.loading("正在读取配置文件...", { duration: 0 });

    const content = await invoke<string>("read_config_file_content", {
      configPath: localConfigPath.value,
    });

    message.destroyAll();
    message.loading("正在格式化配置...", { duration: 0 });
    const formattedContent = formatNginxConfig(content);

    message.destroyAll();
    message.loading("正在校验配置...", { duration: 0 });

    tempFilePath = await invoke<string>("write_temp_config_for_validation", {
      originalConfigPath: localConfigPath.value,
      newContent: formattedContent,
    });

    const testResult = await invoke<{ success: boolean; message: string }>(
      "test_nginx_config_file",
      {
        nginxPath: settingsStore.settings.nginxPath,
        configPath: tempFilePath,
      },
    );

    message.destroyAll();

    if (!testResult.success) {
      logStore.warning(`配置文件格式化完成，但校验失败: ${testResult.message}`);
      dialog.error({
        title: "配置校验失败",
        content: `格式化后的配置校验未通过，未保存到文件。\n\n${testResult.message}`,
        positiveText: "知道了",
        style: { width: "640px" },
      });
      return;
    }

    await invoke("write_formatted_config", {
      configPath: localConfigPath.value,
      formattedContent,
    });

    message.success("配置文件格式化成功，校验通过");
    logStore.success("配置文件格式化成功");
    await configStore.loadConfig(localConfigPath.value);
  } catch (error) {
    message.destroyAll();
    message.error(`格式化失败: ${error}`);
    logStore.error(`格式化配置文件失败: ${error}`);
  } finally {
    isFormatting.value = false;
    if (tempFilePath) {
      await invoke("delete_temp_config", { tempPath: tempFilePath }).catch(
        () => undefined,
      );
    }
  }
};
</script>

<style scoped>
.config-page {
  height: 100%;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: var(--page-gap);
  overflow: hidden;
}

.panel-card {
  background: var(--surface-bg);
  border: 1px solid var(--surface-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--surface-shadow);
  backdrop-filter: blur(18px);
}

.toolbar-panel {
  padding: 16px;
  overflow: hidden;
}

.toolbar-scroll {
  max-height: clamp(156px, 28vh, 248px);
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow: auto;
  padding-right: 4px;
  scrollbar-gutter: stable;
}

.action-row,
.filter-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-search {
  min-width: 260px;
  flex: 1 1 320px;
}

.filter-select {
  width: 220px;
}

.stats-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.stat-chip {
  min-width: 90px;
  padding: 4px 12px;
  border-radius: var(--radius-md);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.stat-chip strong {
  font-size: 14px;
  color: var(--text-primary);
}

.inline-alert {
  margin-top: 4px;
}

.list-panel {
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 16px 12px;
  border-bottom: 1px solid var(--surface-border);
}

.list-panel-header h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.list-panel-header p {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.list-panel-content {
  flex: 1;
  min-height: 0;
  padding: 16px;
  overflow: hidden;
}

.list-panel-body {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.list-panel-body :deep(.n-spin-content) {
  height: 100%;
}

.server-scroll {
  height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 4px;
  scrollbar-gutter: stable;
}

.page-empty {
  margin: auto 0;
}

@media (max-width: 960px) {
  .toolbar-panel,
  .list-panel {
    border-radius: var(--radius-lg);
  }

  .filter-search,
  .filter-select {
    width: 100%;
    min-width: 0;
  }

  .list-panel-header {
    flex-direction: column;
  }
}
</style>
