<template>
  <n-card
    :bordered="false"
    class="server-summary-card"
    :class="{ selected, disabled: !server.enabled }"
    hoverable
    @click="$emit('select')"
  >
    <template #header>
      <n-space align="center" wrap>
        <n-icon
          :component="ServerOutline"
          size="18"
          :color="server.enabled ? '#3b82f6' : '#f59e0b'"
        />
        <n-text strong>{{ server.category || "未分类 Server" }}</n-text>
        <n-tag
          size="small"
          :type="server.enabled ? 'success' : 'warning'"
          round
        >
          {{ server.enabled ? "启用中" : "已停用" }}
        </n-tag>
        <n-tag size="small" type="info" round
          >行 {{ server.startLine }} - {{ server.endLine }}</n-tag
        >
      </n-space>
    </template>

    <template #header-extra>
      <n-tag size="small" type="default" round>
        {{ server.locations.length }} 个 location
      </n-tag>
    </template>

    <n-space vertical :size="12">
      <div class="summary-row">
        <span class="summary-label">监听端口</span>
        <n-space wrap>
          <n-tag
            v-for="listen in server.listen"
            :key="listen"
            size="small"
            type="info"
            round
          >
            {{ listen }}
          </n-tag>
          <n-tag v-if="server.listen.length === 0" size="small" round
            >未配置</n-tag
          >
        </n-space>
      </div>

      <div class="summary-row">
        <span class="summary-label">域名</span>
        <n-space wrap>
          <n-tag
            v-for="name in server.serverName"
            :key="name"
            size="small"
            type="success"
            round
          >
            {{ name }}
          </n-tag>
          <n-tag v-if="server.serverName.length === 0" size="small" round
            >未配置</n-tag
          >
        </n-space>
      </div>
    </n-space>

    <template #action>
      <n-space wrap>
        <n-button size="small" secondary @click.stop="$emit('detail')"
          >详情</n-button
        >
        <n-button size="small" @click.stop="$emit('edit')">编辑</n-button>
        <n-button
          size="small"
          :type="server.enabled ? 'warning' : 'success'"
          @click.stop="$emit('toggle')"
        >
          {{ server.enabled ? "临时停用" : "恢复启用" }}
        </n-button>
        <n-button size="small" type="error" @click.stop="$emit('delete')"
          >删除</n-button
        >
      </n-space>
    </template>
  </n-card>
</template>

<script setup lang="ts">
import { NButton, NCard, NIcon, NSpace, NTag, NText } from "naive-ui";
import { ServerOutline } from "@vicons/ionicons5";
import type { ServerBlock } from "@/types/config";

defineProps<{
  server: ServerBlock;
  selected?: boolean;
}>();

defineEmits<{
  select: [];
  detail: [];
  edit: [];
  toggle: [];
  delete: [];
}>();
</script>

<style scoped>
.server-summary-card {
  border-radius: var(--radius-lg);
  background: var(--surface-bg-strong);
  border: 1px solid var(--surface-border);
  /* box-shadow: 0 8px 18px rgba(15, 23, 42, 0.05); */
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.server-summary-card:hover {
  transform: scale(0.99);
  /* box-shadow: 0 10px 22px rgba(15, 23, 42, 0.08); */
}

.server-summary-card.selected {
  border-color: rgba(59, 130, 246, 0.36);
  box-shadow:
    0 0 0 1px rgba(59, 130, 246, 0.16),
    0 10px 22px rgba(59, 130, 246, 0.08);
}

.server-summary-card.disabled {
  background: rgba(255, 251, 235, 0.9);
}

.summary-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.summary-label {
  flex: 0 0 64px;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 24px;
}
</style>
