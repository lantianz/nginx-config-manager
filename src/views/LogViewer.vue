<template>
  <div class="log-page">
    <section class="panel-card toolbar-panel">
      <div class="toolbar-copy">
        <div>
          <h3>操作日志</h3>
          <p>查看运行轨迹和最近的配置文件变更，按需进入差异详情。</p>
        </div>

        <div class="stats-strip">
          <div class="stat-chip">
            <span>总计</span><strong>{{ logs.length }}</strong>
          </div>
          <div class="stat-chip">
            <span>文件变更</span><strong>{{ fileChangeCount }}</strong>
          </div>
          <div class="stat-chip">
            <span>成功</span><strong>{{ successCount }}</strong>
          </div>
          <div class="stat-chip">
            <span>错误</span><strong>{{ errorCount }}</strong>
          </div>
        </div>
      </div>

      <div class="toolbar-actions">
        <n-radio-group v-model:value="selectedView" size="small">
          <n-radio-button value="all">全部日志</n-radio-button>
          <n-radio-button value="file-change">仅看文件变更</n-radio-button>
        </n-radio-group>

        <n-radio-group v-model:value="selectedLevel" size="small">
          <n-radio-button value="all">全部等级</n-radio-button>
          <n-radio-button value="info">信息</n-radio-button>
          <n-radio-button value="success">成功</n-radio-button>
          <n-radio-button value="warning">警告</n-radio-button>
          <n-radio-button value="error">错误</n-radio-button>
        </n-radio-group>

        <n-button secondary size="small" @click="handleClearLogs">
          <template #icon>
            <n-icon :component="TrashOutline" />
          </template>
          清空日志
        </n-button>
      </div>
    </section>

    <section class="panel-card list-panel">
      <div class="list-panel-content">
        <div class="list-scroll">
          <n-empty
            v-if="filteredLogs.length === 0"
            description="暂无匹配的日志记录"
            class="page-empty"
          />

          <div v-else class="log-list">
            <article
              v-for="log in filteredLogs"
              :key="log.id"
              class="log-card"
              :class="{ 'file-change-card': log.kind === 'file-change' }"
            >
              <div class="log-card-header">
                <div class="log-card-main">
                  <n-tag :type="getTagType(log.level)" size="small" round>
                    {{ getLevelText(log.level) }}
                  </n-tag>
                  <n-tag v-if="log.kind === 'file-change'" size="small" type="info" round>
                    文件变更
                  </n-tag>
                  <span class="log-summary">{{ log.summary }}</span>
                </div>
                <span class="log-time">{{ formatTime(log.timestamp) }}</span>
              </div>

              <div v-if="log.kind === 'file-change'" class="change-meta">
                <div class="change-path allow-select">{{ log.detail.configPath }}</div>
                <div class="change-scopes">
                  <n-tag size="small" round>文件</n-tag>
                  <n-tag v-if="log.detail.serverDiff" size="small" round>Server</n-tag>
                  <n-tag v-if="log.detail.locationDiffs.length > 0" size="small" round>
                    {{ log.detail.locationDiffs.length }} 个 Location
                  </n-tag>
                </div>
              </div>

              <div v-else class="log-message">{{ log.summary }}</div>

              <div v-if="log.kind === 'file-change'" class="log-card-footer">
                <n-button size="small" @click="openDetail(log)">查看变更</n-button>
              </div>
            </article>
          </div>
        </div>
      </div>
    </section>

    <LogDetailDrawer v-model:show="showDetailDrawer" :entry="selectedChangeLog" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import {
  NButton,
  NEmpty,
  NIcon,
  NRadioButton,
  NRadioGroup,
  NTag,
  useDialog,
  useMessage,
} from "naive-ui";
import { TrashOutline } from "@vicons/ionicons5";
import LogDetailDrawer from "@/components/log/LogDetailDrawer.vue";
import { useLogStore } from "@/stores/log";
import type { FileChangeLogEntry, LogEntry, LogLevel, LogViewFilter } from "@/types/nginx";

const dialog = useDialog();
const message = useMessage();
const logStore = useLogStore();

const selectedLevel = ref<"all" | LogLevel>("all");
const selectedView = ref<LogViewFilter>("all");
const showDetailDrawer = ref(false);
const selectedChangeLog = ref<FileChangeLogEntry | null>(null);

const logs = computed(() => logStore.logs);

const filteredLogs = computed(() => {
  return logs.value.filter((log) => {
    const matchView =
      selectedView.value === "all" || log.kind === selectedView.value;
    const matchLevel =
      selectedLevel.value === "all" || log.level === selectedLevel.value;
    return matchView && matchLevel;
  });
});

const fileChangeCount = computed(
  () => logs.value.filter((log) => log.kind === "file-change").length,
);
const successCount = computed(
  () => logs.value.filter((log) => log.level === "success").length,
);
const errorCount = computed(
  () => logs.value.filter((log) => log.level === "error").length,
);

const getLevelText = (level: LogLevel) =>
  ({
    info: "信息",
    success: "成功",
    warning: "警告",
    error: "错误",
  })[level];

const getTagType = (level: LogLevel) =>
  ({
    info: "info",
    success: "success",
    warning: "warning",
    error: "error",
  })[level] as "info" | "success" | "warning" | "error";

const formatTime = (timestamp: Date) => {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};

const openDetail = (log: LogEntry) => {
  if (log.kind !== "file-change") {
    return;
  }

  selectedChangeLog.value = log;
  showDetailDrawer.value = true;
};

const handleClearLogs = () => {
  dialog.warning({
    title: "确认清空",
    content: "确定要清空所有日志记录吗？此操作不可恢复。",
    positiveText: "确定",
    negativeText: "取消",
    onPositiveClick: async () => {
      showDetailDrawer.value = false;
      selectedChangeLog.value = null;
      await logStore.clear();
      message.success("日志已清空");
    },
  });
};
</script>

<style scoped>
.log-page {
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
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.toolbar-copy {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  flex: 1;
}

.toolbar-copy h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.toolbar-copy p {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.toolbar-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 12px;
  flex-wrap: wrap;
}

.stats-strip {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.stat-chip {
  min-width: 86px;
  padding: 4px 12px;
  border-radius: var(--radius-md);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.stat-chip span {
  font-size: 12px;
  color: var(--text-secondary);
}

.stat-chip strong {
  font-size: 14px;
  color: var(--text-primary);
}

.list-panel {
  min-height: 0;
  overflow: hidden;
}

.list-panel-content {
  height: 100%;
  padding: 16px;
  overflow: hidden;
}

.list-scroll {
  height: 100%;
  overflow: auto;
  padding-right: 4px;
  scrollbar-gutter: stable;
}

.page-empty {
  margin: auto 0;
}

.log-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.log-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border-radius: var(--radius-lg);
  background: var(--surface-bg-strong);
  border: 1px solid var(--surface-border);
}

.file-change-card {
  border-color: rgba(59, 130, 246, 0.16);
}

.log-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.log-card-main {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.log-summary {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.6;
  word-break: break-word;
}

.log-time {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.log-message {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
}

.change-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.change-path {
  font-size: 12px;
  color: var(--text-secondary);
  word-break: break-all;
}

.change-scopes {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.log-card-footer {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 980px) {
  .toolbar-panel,
  .toolbar-actions {
    flex-direction: column;
    align-items: flex-start;
  }

  .log-card-header,
  .change-meta {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
