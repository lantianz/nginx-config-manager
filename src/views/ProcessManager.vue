<template>
  <div class="process-manager">
    <!-- 路径配置区域 -->
    <n-card title="路径配置" class="config-card">
      <n-space vertical :size="0">
        <n-form-item label="Nginx 程序(安装路径或可执行文件)">
          <n-input :value="nginxPath" @update:value="nginxPath = $event" placeholder="例如: C:\nginx-1.24.0"
            @blur="handlePathChange">
            <template #suffix>
              <n-button text @click="selectNginxPath">
                <n-icon :component="FolderOpenOutline" />
              </n-button>
            </template>
          </n-input>
        </n-form-item>

        <n-form-item label="配置文件路径">
          <n-input :value="configPath" @update:value="configPath = $event"
            placeholder="例如: C:\nginx-1.24.0\conf\nginx.conf" @blur="handlePathChange">
            <template #suffix>
              <n-button text @click="selectConfigPath">
                <n-icon :component="DocumentTextOutline" />
              </n-button>
            </template>
          </n-input>
        </n-form-item>
      </n-space>
    </n-card>

    <div class="status-control-container">

      <!-- 状态显示区域 -->
      <n-card title="运行状态" class="status-card">
        <template #header-extra>
          <n-button secondary type="info" :loading="nginxStore.isLoading" @click="handleRefreshStatus">
            <n-icon :component="RefreshOutline" />
            刷新状态
          </n-button>
        </template>
        <n-space vertical :size="16">
          <n-statistic label="当前状态">
            <template #prefix>
              <n-icon :component="status.isRunning ? CheckmarkCircleOutline : CloseCircleOutline"
                :color="status.isRunning ? '#18a058' : '#d03050'" size="24" />
            </template>
            <span :style="{ color: status.isRunning ? '#18a058' : '#d03050' }">
              {{ status.isRunning ? '运行中' : '未运行' }}
            </span>
          </n-statistic>

          <n-statistic v-if="status.isRunning" label="进程数量" :value="status.processCount" />


        </n-space>
      </n-card>

      <!-- 控制按钮区域 -->
      <n-card title="进程控制" class="control-card">
        <n-space :size="12">
          <n-button type="success" :loading="nginxStore.isLoading" :disabled="!nginxPath || status.isRunning"
            @click="handleStart">
            <n-icon :component="PlayOutline" />
            启动
          </n-button>

          <n-button type="error" :loading="nginxStore.isLoading" :disabled="!status.isRunning" @click="handleStop">
            <n-icon :component="StopOutline" />
            停止
          </n-button>

          <n-button type="warning" :loading="nginxStore.isLoading" :disabled="!nginxPath" @click="handleRestart">
            <n-icon :component="RefreshCircleOutline" />
            重启
          </n-button>

          <n-button type="info" :loading="nginxStore.isLoading" :disabled="!nginxPath || !status.isRunning"
            @click="handleReload">
            <n-icon :component="ReloadOutline" />
            重载配置
          </n-button>

          <n-button secondary :loading="nginxStore.isLoading" :disabled="!nginxPath" @click="handleTestConfig">
            <n-icon :component="CheckmarkDoneOutline" />
            校验配置
          </n-button>
        </n-space>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import {
  NCard,
  NSpace,
  NFormItem,
  NInput,
  NButton,
  NIcon,
  NStatistic,
  useMessage,
} from 'naive-ui';
import {
  FolderOpenOutline,
  DocumentTextOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  RefreshOutline,
  PlayOutline,
  StopOutline,
  RefreshCircleOutline,
  ReloadOutline,
  CheckmarkDoneOutline,
} from '@vicons/ionicons5';
import { useNginxStore } from '../stores/nginx';
import { useSettingsStore } from '../stores/settings';
import { useLogStore } from '../stores/log';

const message = useMessage();
const nginxStore = useNginxStore();
const settingsStore = useSettingsStore();
const logStore = useLogStore();

const nginxPath = ref('');
const configPath = ref('');

const status = computed(() => nginxStore.status);

// 初始化：触发设置加载（AppLayout 已加载过则直接响应 watch）
onMounted(() => {
  settingsStore.loadSettings();
});

// 响应式同步设置到本地路径（settings 加载完成后自动触发）
watch(
  () => settingsStore.settings,
  (s) => {
    nginxPath.value = s.nginxPath;
    configPath.value = s.configPath;
  },
  { immediate: true, deep: true }
);

// 路径变更处理
const handlePathChange = () => {
  settingsStore.updateNginxPath(nginxPath.value);
  settingsStore.updateConfigPath(configPath.value);
};

// 选择 Nginx 路径
const selectNginxPath = async () => {
  try {
    const selected = await open({
      title: '选择 Nginx 可执行文件',
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'Nginx 可执行文件',
          extensions: ['exe']
        }
      ]
    });

    if (selected) {
      // 保存完整的文件路径（支持文件路径和目录路径两种方式）
      nginxPath.value = selected as string;
      handlePathChange();
      message.success('已选择 Nginx 可执行文件');
    }
  } catch (error) {
    message.error('选择文件失败: ' + error);
  }
};

// 选择配置文件路径
const selectConfigPath = async () => {
  try {
    const selected = await open({
      title: '选择 Nginx 配置文件',
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'Nginx 配置文件',
          extensions: ['conf']
        }
      ]
    });

    if (selected) {
      configPath.value = selected as string;
      handlePathChange();
      message.success('已选择配置文件');
    }
  } catch (error) {
    message.error('选择文件失败: ' + error);
  }
};

// 刷新状态
const handleRefreshStatus = async () => {
  try {
    await nginxStore.checkStatus();
    logStore.info('状态已刷新');
  } catch (error) {
    message.error('刷新状态失败');
    logStore.error(`刷新状态失败: ${error}`);
  }
};

// 启动 Nginx
const handleStart = () => {
  if (!nginxPath.value) {
    message.warning('请先设置 Nginx 路径');
    return;
  }
  logStore.info('正在启动 Nginx...');
  nginxStore.start(nginxPath.value);
};

// 停止 Nginx
const handleStop = () => {
  logStore.info('正在停止 Nginx...');
  nginxStore.stop();
};

// 重启 Nginx
const handleRestart = () => {
  if (!nginxPath.value) {
    message.warning('请先设置 Nginx 路径');
    return;
  }
  logStore.info('正在重启 Nginx...');
  nginxStore.restart(nginxPath.value);
};

// 重载配置
const handleReload = () => {
  if (!nginxPath.value) {
    message.warning('请先设置 Nginx 路径');
    return;
  }
  logStore.info('正在重载配置...');
  nginxStore.reload(nginxPath.value);
};

// 校验配置
const handleTestConfig = () => {
  if (!nginxPath.value) {
    message.warning('请先设置 Nginx 路径');
    return;
  }
  logStore.info('正在校验配置...');
  nginxStore.testConfig(nginxPath.value);
};
</script>

<style scoped>
.process-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-card,
.status-card,
.control-card {
  width: 100%;
}

.status-control-container {
  display: flex;
  flex-direction: row;
  gap: 16px;
}
</style>
