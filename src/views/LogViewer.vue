<template>
  <div class="log-page">
    <section class="panel-card toolbar-panel">
      <div class="toolbar-left">
        <div>
          <h3>操作日志</h3>
          <p>查看最近的操作轨迹，并按日志等级筛选。</p>
        </div>
        <div class="stats-strip">
          <div class="stat-chip">
            <span>总计</span><strong>{{ logs.length }}</strong>
          </div>
          <div class="stat-chip">
            <span>成功</span><strong>{{ successCount }}</strong>
          </div>
          <div class="stat-chip">
            <span>错误</span><strong>{{ errorCount }}</strong>
          </div>
          <div class="stat-chip">
            <span>警告</span><strong>{{ warningCount }}</strong>
          </div>
        </div>
      </div>

      <div class="toolbar-right">
        <n-radio-group v-model:value="selectedLevel" size="small">
          <n-radio-button value="all">全部</n-radio-button>
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
            description="暂无日志记录"
            class="page-empty"
          />

          <n-timeline v-else>
            <n-timeline-item
              v-for="log in filteredLogs"
              :key="log.id"
              :type="getTimelineType(log.level)"
              :color="getLogColor(log.level)"
            >
              <template #icon>
                <n-icon :component="getLogIcon(log.level)" />
              </template>

              <div class="log-item">
                <div class="log-header">
                  <n-tag :type="getTagType(log.level)" size="small" round>
                    {{ getLevelText(log.level) }}
                  </n-tag>
                  <span class="log-time">{{ formatTime(log.timestamp) }}</span>
                </div>
                <div class="log-message">{{ log.message }}</div>
              </div>
            </n-timeline-item>
          </n-timeline>
        </div>
      </div>
    </section>
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
  NTimeline,
  NTimelineItem,
  useDialog,
  useMessage,
} from "naive-ui";
import {
  CheckmarkCircleOutline,
  CloseCircleOutline,
  InformationCircleOutline,
  TrashOutline,
  WarningOutline,
} from "@vicons/ionicons5";
import { useLogStore } from "@/stores/log";
import type { LogLevel } from "@/types/nginx";

const dialog = useDialog();
const message = useMessage();
const logStore = useLogStore();

const selectedLevel = ref<"all" | LogLevel>("all");
const logs = computed(() => logStore.logs);

const filteredLogs = computed(() => {
  if (selectedLevel.value === "all") {
    return logs.value;
  }
  return logs.value.filter((log) => log.level === selectedLevel.value);
});

const successCount = computed(
  () => logs.value.filter((log) => log.level === "success").length,
);
const errorCount = computed(
  () => logs.value.filter((log) => log.level === "error").length,
);
const warningCount = computed(
  () => logs.value.filter((log) => log.level === "warning").length,
);

const getLevelText = (level: LogLevel) =>
  ({
    info: "信息",
    success: "成功",
    warning: "警告",
    error: "错误",
  })[level];

const getTimelineType = (level: LogLevel) =>
  ({
    info: "info",
    success: "success",
    warning: "warning",
    error: "error",
  })[level] as "info" | "success" | "warning" | "error";

const getLogColor = (level: LogLevel) =>
  ({
    info: "#3b82f6",
    success: "#10b981",
    warning: "#f59e0b",
    error: "#f43f5e",
  })[level];

const getLogIcon = (level: LogLevel) =>
  ({
    info: InformationCircleOutline,
    success: CheckmarkCircleOutline,
    warning: WarningOutline,
    error: CloseCircleOutline,
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

const handleClearLogs = () => {
  dialog.warning({
    title: "确认清空",
    content: "确定要清空所有日志记录吗？此操作不可恢复。",
    positiveText: "确定",
    negativeText: "取消",
    onPositiveClick: () => {
      logStore.clear();
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

.toolbar-left {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  flex: 1;
}

.toolbar-left h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.toolbar-left p {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.toolbar-right {
  width: fit-content;
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
  min-width: 78px;
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

.log-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.log-time {
  font-size: 12px;
  color: var(--text-secondary);
}

.log-message {
  font-size: 13px;
  line-height: 1.7;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
  -webkit-user-select: text;
}

@media (max-width: 860px) {
  .toolbar-panel {
    flex-direction: column;
  }

  .toolbar-right {
    justify-content: flex-start;
  }
}
</style>
