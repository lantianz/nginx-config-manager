<template>
  <div class="log-viewer">
    <n-card title="操作日志" style="height: 100%;" content-style="min-height: 0;">
      <template #header-extra>
        <n-button secondary size="small" @click="handleClearLogs">
          <n-icon :component="TrashOutline" />
          清空日志
        </n-button>
      </template>

      <n-space vertical :size="12" style="height: 100%;" class="666" item-class="log-viewer-item">
        <!-- 日志统计和过滤器 -->
        <n-space :size="16" align="center" justify="space-between">
          <n-space :size="16">
            <n-statistic label="总计" :value="logs.length" />
            <n-statistic label="成功" :value="successCount" />
            <n-statistic label="错误" :value="errorCount" />
            <n-statistic label="警告" :value="warningCount" />
          </n-space>

          <!-- 日志等级过滤器 -->
          <n-radio-group v-model:value="selectedLevel" size="small">
            <n-radio-button value="all">全部</n-radio-button>
            <n-radio-button value="info">信息</n-radio-button>
            <n-radio-button value="success">成功</n-radio-button>
            <n-radio-button value="warning">警告</n-radio-button>
            <n-radio-button value="error">错误</n-radio-button>
          </n-radio-group>
        </n-space>

        <n-divider />

        <!-- 日志列表 -->
        <div class="log-list">
          <n-empty v-if="filteredLogs.length === 0" description="暂无日志记录" />

          <n-timeline v-else>
            <n-timeline-item
              v-for="log in reversedFilteredLogs"
              :key="log.id"
              :type="getTimelineType(log.level)"
              :color="getLogColor(log.level)"
            >
              <template #icon>
                <n-icon :component="getLogIcon(log.level)" />
              </template>

              <div class="log-item">
                <div class="log-header">
                  <n-tag :type="getTagType(log.level)" size="small">
                    {{ getLevelText(log.level) }}
                  </n-tag>
                  <span class="log-time">{{ formatTime(log.timestamp) }}</span>
                </div>
                <div class="log-message">{{ log.message }}</div>
              </div>
            </n-timeline-item>
          </n-timeline>
        </div>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  NCard,
  NSpace,
  NButton,
  NIcon,
  NStatistic,
  NDivider,
  NEmpty,
  NTimeline,
  NTimelineItem,
  NTag,
  NRadioGroup,
  NRadioButton,
  useDialog,
  useMessage,
} from 'naive-ui';
import {
  TrashOutline,
  InformationCircleOutline,
  CheckmarkCircleOutline,
  WarningOutline,
  CloseCircleOutline,
} from '@vicons/ionicons5';
import { useLogStore } from '../stores/log';
import type { LogLevel } from '../types/nginx';

const dialog = useDialog();
const message = useMessage();
const logStore = useLogStore();

// 日志等级过滤器
const selectedLevel = ref<'all' | LogLevel>('all');

const logs = computed(() => logStore.logs);

// 过滤后的日志
const filteredLogs = computed(() => {
  if (selectedLevel.value === 'all') {
    return logs.value;
  }
  return logs.value.filter(log => log.level === selectedLevel.value);
});

// 反转过滤后的日志（最新的在前）
const reversedFilteredLogs = computed(() => [...filteredLogs.value].reverse());

// 统计各类日志数量
const successCount = computed(() => logs.value.filter(log => log.level === 'success').length);
const errorCount = computed(() => logs.value.filter(log => log.level === 'error').length);
const warningCount = computed(() => logs.value.filter(log => log.level === 'warning').length);

// 获取日志级别文本
const getLevelText = (level: LogLevel) => {
  const map = {
    info: '信息',
    success: '成功',
    warning: '警告',
    error: '错误',
  };
  return map[level];
};

// 获取时间线类型
const getTimelineType = (level: LogLevel) => {
  const map = {
    info: 'info',
    success: 'success',
    warning: 'warning',
    error: 'error',
  };
  return map[level] as 'info' | 'success' | 'warning' | 'error';
};

// 获取日志颜色
const getLogColor = (level: LogLevel) => {
  const map = {
    info: '#0064C8',
    success: '#18A058',
    warning: '#F0A020',
    error: '#D03050',
  };
  return map[level];
};

// 获取日志图标
const getLogIcon = (level: LogLevel) => {
  const map = {
    info: InformationCircleOutline,
    success: CheckmarkCircleOutline,
    warning: WarningOutline,
    error: CloseCircleOutline,
  };
  return map[level];
};

// 获取标签类型
const getTagType = (level: LogLevel) => {
  const map = {
    info: 'info',
    success: 'success',
    warning: 'warning',
    error: 'error',
  };
  return map[level] as 'info' | 'success' | 'warning' | 'error';
};

// 格式化时间
const formatTime = (timestamp: Date) => {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};

// 清空日志
const handleClearLogs = () => {
  dialog.warning({
    title: '确认清空',
    content: '确定要清空所有日志记录吗?此操作不可恢复。',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      logStore.clear();
      message.success('日志已清空');
    },
  });
};
</script>

<style scoped>
.log-viewer {
  width: 100%;
  height: 100%;
}

.log-list {
  height: 100%;
  overflow-y: auto;
}

.log-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.log-time {
  font-size: 12px;
  color: #999;
}

.log-message {
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
<style>
.log-viewer-item:last-child {
  min-height: 0;
  flex: 1;
  overflow: hidden;
}
</style>
