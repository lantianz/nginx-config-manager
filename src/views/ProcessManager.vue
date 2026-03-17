<template>
  <div class="process-page">
    <section class="panel-card overview-panel">
      <div class="overview-grid">
        <div class="status-column">
          <div class="overview-header">
            <div class="summary-strip">
              <n-tag round size="small" type="info"
                >进程 {{ status.processCount }} 个</n-tag
              >
              <n-tag
                round
                size="small"
                :type="permissionStatus?.isElevated ? 'success' : 'warning'"
              >
                {{
                  permissionStatus?.isElevated
                    ? "管理员权限已就绪"
                    : "当前为普通权限"
                }}
              </n-tag>
            </div>

            <n-button
              :type="hasPathsConfigured ? 'default' : 'primary'"
              secondary
              class="runtime-entry-button"
              @click="showRuntimeSettings = true"
            >
              <template #icon>
                <n-icon :component="SettingsOutline" />
              </template>
              路径设置
            </n-button>
          </div>

          <n-card
            :bordered="false"
            class="inner-card status-card"
            title="当前状态"
          >
            <div class="status-panel">
              <div>
                <div class="status-label">运行状态</div>
                <div
                  class="status-value"
                  :class="status.isRunning ? 'running' : 'stopped'"
                >
                  {{ status.isRunning ? "运行中" : "未运行" }}
                </div>
              </div>

              <n-button
                secondary
                size="small"
                :loading="nginxStore.isLoading"
                @click="handleRefreshStatus"
              >
                <template #icon>
                  <n-icon :component="RefreshOutline" />
                </template>
                刷新状态
              </n-button>
            </div>

            <div class="status-detail-grid">
              <div class="detail-box">
                <span class="detail-box-label">进程数量</span>
                <strong>{{ status.processCount }}</strong>
              </div>
              <div class="detail-box">
                <span class="detail-box-label">路径状态</span>
                <strong>{{ hasPathsConfigured ? "已配置" : "待设置" }}</strong>
              </div>
            </div>
          </n-card>
        </div>

        <n-card
          :bordered="false"
          class="inner-card action-card"
          title="快捷操作"
        >
          <div class="action-grid">
            <n-button
              type="success"
              :loading="nginxStore.isLoading"
              :disabled="!nginxPath || status.isRunning"
              @click="handleStart"
            >
              <template #icon>
                <n-icon :component="PlayOutline" />
              </template>
              启动
            </n-button>
            <n-button
              type="error"
              :loading="nginxStore.isLoading"
              :disabled="!status.isRunning"
              @click="handleStop"
            >
              <template #icon>
                <n-icon :component="StopOutline" />
              </template>
              停止
            </n-button>
            <n-button
              type="warning"
              :loading="nginxStore.isLoading"
              :disabled="!nginxPath"
              @click="handleRestart"
            >
              <template #icon>
                <n-icon :component="RefreshCircleOutline" />
              </template>
              重启
            </n-button>
            <n-button
              type="info"
              :loading="nginxStore.isLoading"
              :disabled="!nginxPath || !status.isRunning"
              @click="handleReload"
            >
              <template #icon>
                <n-icon :component="ReloadOutline" />
              </template>
              重载配置
            </n-button>
            <n-button
              secondary
              class="action-wide"
              :loading="nginxStore.isLoading"
              :disabled="!nginxPath"
              @click="handleTestConfig"
            >
              <template #icon>
                <n-icon :component="CheckmarkDoneOutline" />
              </template>
              校验当前配置
            </n-button>
          </div>
        </n-card>
      </div>
    </section>

    <section class="panel-card port-panel">
      <div class="port-panel-shell">
        <div class="port-panel-header">
          <div>
            <h3>端口占用工具</h3>
          </div>

          <div class="port-toolbar">
            <n-input-number
              v-model:value="portQuery"
              :min="1"
              :max="65535"
              clearable
              placeholder="输入端口号"
              class="port-input"
            />
            <n-space wrap>
              <n-button
                type="primary"
                :loading="portQueryLoading"
                @click="handleInspectPort"
                >查询占用</n-button
              >
              <n-button
                type="warning"
                :loading="portActionLoading"
                :disabled="!portInspection?.isOccupied"
                @click="handleReleasePort"
              >
                释放该端口
              </n-button>
            </n-space>
          </div>
        </div>

        <div class="port-panel-body">
          <n-empty
            v-if="!hasQueriedPort"
            description="输入端口后开始查询"
            class="port-empty"
          />
          <n-empty
            v-else-if="hasQueriedPort && !portInspection?.isOccupied"
            description="该端口当前空闲"
            class="port-empty"
          />

          <template v-else-if="portInspection">
            <div class="inspection-summary">
              <div>
                <div class="summary-title">端口 {{ portInspection.port }}</div>
                <div class="summary-text">{{ portInspection.message }}</div>
              </div>
              <n-tag
                round
                :type="portInspection.isOccupied ? 'error' : 'success'"
              >
                {{ portInspection.isOccupied ? "已占用" : "空闲" }}
              </n-tag>
            </div>

            <div v-if="portInspection.isOccupied" class="process-list">
              <n-card
                v-for="entry in portInspection.entries"
                :key="`${entry.localPort}-${entry.pid}`"
                :bordered="false"
                class="process-item"
                size="small"
              >
                <template #header>
                  <n-space align="center" wrap>
                    <n-text strong>{{
                      entry.processName || "未知进程"
                    }}</n-text>
                    <n-tag size="small" type="info" round
                      >PID {{ entry.pid }}</n-tag
                    >
                    <n-tag
                      size="small"
                      :type="
                        entry.processName.toLowerCase().includes('nginx')
                          ? 'success'
                          : 'default'
                      "
                      round
                    >
                      {{ entry.protocol }}/{{ entry.status }}
                    </n-tag>
                  </n-space>
                </template>

                <template #header-extra>
                  <n-button
                    size="small"
                    type="error"
                    :loading="portActionLoading"
                    @click="handleTerminateProcess(entry.pid)"
                  >
                    结束进程
                  </n-button>
                </template>

                <div class="process-detail-grid">
                  <div class="process-field">
                    <span class="field-label">监听地址</span>
                    <span class="field-value allow-select"
                      >{{ entry.localAddress }}:{{ entry.localPort }}</span
                    >
                  </div>
                  <div class="process-field">
                    <span class="field-label">启动用户</span>
                    <span class="field-value allow-select">{{
                      entry.user || "未知"
                    }}</span>
                  </div>
                  <div class="process-field">
                    <span class="field-label">启动时间</span>
                    <span class="field-value allow-select">{{
                      entry.startTime || "未知"
                    }}</span>
                  </div>
                  <div class="process-field wide">
                    <span class="field-label">可执行文件</span>
                    <span class="field-value break-all allow-select">{{
                      entry.executablePath || "未获取到"
                    }}</span>
                  </div>
                  <div class="process-field wide">
                    <span class="field-label">命令行</span>
                    <span class="field-value break-all allow-select">{{
                      entry.commandLine || "未获取到"
                    }}</span>
                  </div>
                </div>
              </n-card>
            </div>
          </template>
        </div>
      </div>
    </section>

    <n-modal
      :show="showRuntimeSettings"
      preset="card"
      title="路径设置"
      :bordered="false"
      style="width: 600px"
      class="runtime-settings-modal"
      @update:show="showRuntimeSettings = $event"
    >
      <RuntimeEnvironmentSection compact :show-descriptions="false" />
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  NButton,
  NCard,
  NEmpty,
  NIcon,
  NInputNumber,
  NModal,
  NSpace,
  NTag,
  NText,
  useDialog,
  useMessage,
} from "naive-ui";
import {
  CheckmarkDoneOutline,
  PlayOutline,
  RefreshCircleOutline,
  RefreshOutline,
  ReloadOutline,
  SettingsOutline,
  StopOutline,
} from "@vicons/ionicons5";
import RuntimeEnvironmentSection from "@/components/settings/RuntimeEnvironmentSection.vue";
import type {
  PermissionStatus,
  PortInspectionResult,
  ProcessOperationResult,
} from "@/types/nginx";
import { useLogStore } from "@/stores/log";
import { useNginxStore } from "@/stores/nginx";
import { useSettingsStore } from "@/stores/settings";

const message = useMessage();
const dialog = useDialog();
const nginxStore = useNginxStore();
const settingsStore = useSettingsStore();
const logStore = useLogStore();

const showRuntimeSettings = ref(false);
const portQuery = ref<number | null>(null);
const permissionStatus = ref<PermissionStatus | null>(null);
const portInspection = ref<PortInspectionResult | null>(null);
const hasQueriedPort = ref(false);
const portQueryLoading = ref(false);
const portActionLoading = ref(false);

const status = computed(() => nginxStore.status);
const nginxPath = computed(() => settingsStore.settings.nginxPath.trim());
const configPath = computed(() => settingsStore.settings.configPath.trim());
const hasPathsConfigured = computed(() =>
  Boolean(nginxPath.value && configPath.value),
);

onMounted(async () => {
  if (!settingsStore.isLoaded) {
    await settingsStore.loadSettings();
  }
  await loadPermissionStatus();
});

const loadPermissionStatus = async () => {
  try {
    permissionStatus.value = await nginxStore.checkPermissionStatus();
  } catch (error) {
    console.error("加载权限状态失败:", error);
  }
};

const handleRefreshStatus = async () => {
  try {
    await nginxStore.checkStatus();
    logStore.info("状态已刷新");
  } catch (error) {
    message.error("刷新状态失败");
    logStore.error(`刷新状态失败: ${error}`);
  }
};

const handleStart = () => {
  if (!nginxPath.value) {
    message.warning("请先设置 Nginx 路径");
    showRuntimeSettings.value = true;
    return;
  }
  logStore.info("正在启动 Nginx...");
  nginxStore.start(nginxPath.value);
};

const handleStop = () => {
  logStore.info("正在停止 Nginx...");
  nginxStore.stop();
};

const handleRestart = () => {
  if (!nginxPath.value) {
    message.warning("请先设置 Nginx 路径");
    showRuntimeSettings.value = true;
    return;
  }
  logStore.info("正在重启 Nginx...");
  nginxStore.restart(nginxPath.value);
};

const handleReload = () => {
  if (!nginxPath.value) {
    message.warning("请先设置 Nginx 路径");
    showRuntimeSettings.value = true;
    return;
  }
  logStore.info("正在重载配置...");
  nginxStore.reload(nginxPath.value);
};

const handleTestConfig = () => {
  if (!nginxPath.value) {
    message.warning("请先设置 Nginx 路径");
    showRuntimeSettings.value = true;
    return;
  }
  logStore.info("正在校验配置...");
  nginxStore.testConfig(nginxPath.value);
};

const inspectPort = async () => {
  if (!portQuery.value) {
    throw new Error("请输入端口号");
  }

  const results = await nginxStore.inspectPorts([portQuery.value]);
  portInspection.value = results[0] ?? null;
  hasQueriedPort.value = true;

  if (portInspection.value?.permissionStatus) {
    permissionStatus.value = portInspection.value.permissionStatus;
  }
};

const handleInspectPort = async () => {
  if (!portQuery.value) {
    message.warning("请输入要查询的端口");
    return;
  }

  portQueryLoading.value = true;
  try {
    await inspectPort();
    logStore.info(`已查询端口 ${portQuery.value} 的占用情况`);
  } catch (error) {
    message.error(`查询端口失败: ${error}`);
    logStore.error(`查询端口失败: ${error}`);
  } finally {
    portQueryLoading.value = false;
  }
};

const finishPortAction = async (
  result: ProcessOperationResult,
  successLog: string,
  failureLog: string,
) => {
  if (result.success) {
    message.success(result.message);
    logStore.success(successLog);
  } else {
    message.error(result.message);
    logStore.error(`${failureLog}: ${result.message}`);
  }

  await loadPermissionStatus();
  if (portQuery.value) {
    await inspectPort().catch(() => undefined);
  }
};

const handleTerminateProcess = (pid: number) => {
  dialog.warning({
    title: "确认结束进程",
    content: `确定要结束 PID ${pid} 吗？结束后可能会影响当前占用该端口的服务。`,
    positiveText: "结束进程",
    negativeText: "取消",
    onPositiveClick: async () => {
      portActionLoading.value = true;
      try {
        const result = await nginxStore.terminateProcess(pid);
        await finishPortAction(
          result,
          `已结束 PID ${pid}`,
          `结束 PID ${pid} 失败`,
        );
      } catch (error) {
        message.error(`结束进程失败: ${error}`);
        logStore.error(`结束进程失败: ${error}`);
      } finally {
        portActionLoading.value = false;
      }
    },
  });
};

const handleReleasePort = () => {
  if (!portInspection.value?.isOccupied) {
    message.warning("当前端口未被占用");
    return;
  }

  const targetPort = portInspection.value.port;

  dialog.warning({
    title: "确认释放端口",
    content: `确定要释放端口 ${targetPort} 吗？这会结束所有占用该端口的进程。`,
    positiveText: "释放端口",
    negativeText: "取消",
    onPositiveClick: async () => {
      portActionLoading.value = true;
      try {
        const result = await nginxStore.releasePort(targetPort);
        await finishPortAction(
          result,
          `已释放端口 ${targetPort}`,
          `释放端口 ${targetPort} 失败`,
        );
      } catch (error) {
        message.error(`释放端口失败: ${error}`);
        logStore.error(`释放端口失败: ${error}`);
      } finally {
        portActionLoading.value = false;
      }
    },
  });
};
</script>

<style scoped>
.process-page {
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

.overview-panel {
  padding: 14px;
}

.overview-grid {
  display: grid;
  grid-template-columns: minmax(300px, 0.84fr) minmax(360px, 1.16fr);
  gap: 12px;
  align-items: stretch;
}

.status-column {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.overview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.summary-strip {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.runtime-entry-button {
  flex-shrink: 0;
}

.inner-card {
  height: 100%;
  border-radius: var(--radius-lg);
  background: var(--surface-bg-strong);
}

.inner-card :deep(.n-card__header) {
  padding: 14px 16px 0;
}

.inner-card :deep(.n-card__content) {
  padding: 12px 16px 16px;
}

.status-card,
.action-card {
  min-height: 170px;
}

.status-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.status-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.status-value {
  margin-top: 4px;
  font-size: 22px;
  font-weight: 600;
  line-height: 1.2;
}

.status-value.running {
  color: #059669;
}

.status-value.stopped {
  color: #e11d48;
}

.status-detail-grid {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.detail-box {
  min-height: 72px;
  padding: 12px;
  border-radius: var(--radius-md);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-box-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.detail-box strong {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary);
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  align-content: start;
}

.action-grid :deep(.n-button) {
  min-height: 42px;
  border-radius: var(--radius-md);
}

.action-wide {
  grid-column: 1 / -1;
}

.port-panel {
  min-height: 0;
  padding: 16px;
  overflow: hidden;
}

.port-panel-shell {
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 12px;
}

.port-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--surface-border);
}

.port-panel-header h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.port-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.port-input {
  width: 180px;
}

.port-panel-body {
  min-height: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  scrollbar-gutter: stable;
  padding-right: 4px;
}

.port-empty {
  margin: auto 0;
}

.inspection-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: var(--radius-md);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
}

.summary-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.summary-text {
  margin-top: 4px;
  font-size: 13px;
  color: var(--text-secondary);
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.process-item {
  border-radius: var(--radius-lg);
  background: var(--surface-bg-strong);
  border: 1px solid var(--surface-border);
}

.process-detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 14px;
}

.process-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.process-field.wide {
  grid-column: 1 / -1;
}

.field-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.field-value {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-primary);
}

.break-all {
  word-break: break-all;
}

.runtime-settings-modal {
  width: min(640px, calc(100vw - 40px));
}

@media (max-width: 920px) {
  .overview-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 860px) {
  .overview-header,
  .port-panel-header,
  .port-toolbar,
  .status-panel,
  .inspection-summary {
    flex-direction: column;
    align-items: stretch;
  }

  .status-detail-grid,
  .action-grid,
  .process-detail-grid {
    grid-template-columns: 1fr;
  }

  .port-input {
    width: 100%;
  }
}
</style>
