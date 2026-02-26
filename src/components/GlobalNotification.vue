<template>
  <!-- 纯逻辑组件，无 UI -->
</template>

<script setup lang="ts">
import { onUnmounted } from 'vue';
import { useMessage } from 'naive-ui';
import { eventBus, EVENTS } from '@/composables/useEventBus';
import type { NginxOperationResult } from '@/composables/useEventBus';
import { useLogStore } from '@/stores/log';

const message = useMessage();
const logStore = useLogStore();

const OPERATION_LABELS: Record<NginxOperationResult['operation'], string> = {
  start: '启动',
  stop: '停止',
  restart: '重启',
  reload: '重载配置',
  test: '配置校验',
};

const off = eventBus.on<NginxOperationResult>(EVENTS.NGINX_OPERATION_RESULT, ({ success, message: msg, operation }) => {
  const label = OPERATION_LABELS[operation] ?? operation;
  if (success) {
    message.success(msg || `${label}成功`);
    logStore.success(msg || `${label}成功`);
  } else {
    message.error(msg || `${label}失败`);
    logStore.error(`${label}失败: ${msg}`);
  }
});

onUnmounted(off);
</script>
