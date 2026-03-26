<template>
  <n-drawer
    :show="show"
    :width="drawerWidth"
    placement="right"
    :mask-closable="true"
    @update:show="emit('update:show', $event)"
  >
    <n-drawer-content v-if="entry" title="变更详情" closable>
      <div class="detail-shell">
        <section class="detail-meta">
          <div class="meta-row">
            <span class="meta-label">操作</span>
            <span class="meta-value">{{ entry.detail.operationLabel }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">文件</span>
            <span class="meta-value allow-select">{{ entry.detail.configPath }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">时间</span>
            <span class="meta-value">{{ formatTime(entry.detail.savedAt) }}</span>
          </div>
        </section>

        <n-tabs v-model:value="activeTab" type="line" animated class="detail-tabs">
          <n-tab-pane name="file" tab="文件差异">
            <div class="diff-panel">
              <vue-monaco-diff-editor
                :original="entry.detail.fileDiff.before"
                :modified="entry.detail.fileDiff.after"
                language="nginx"
                :theme="editorTheme"
                :options="diffOptions"
                class="diff-editor"
              />
            </div>
          </n-tab-pane>

          <n-tab-pane name="server" tab="Server 差异">
            <div v-if="entry.detail.serverDiff" class="diff-panel">
              <div class="diff-panel-title">{{ entry.detail.serverDiff.label }}</div>
              <vue-monaco-diff-editor
                :original="entry.detail.serverDiff.before"
                :modified="entry.detail.serverDiff.after"
                language="nginx"
                :theme="editorTheme"
                :options="diffOptions"
                class="diff-editor"
              />
            </div>
            <n-empty
              v-else
              description="本次未生成 Server 级差异"
              class="detail-empty"
            />
          </n-tab-pane>

          <n-tab-pane name="location" tab="Location 差异">
            <template v-if="entry.detail.locationDiffs.length > 0">
              <div class="location-toolbar">
                <n-select
                  v-model:value="selectedLocationIndex"
                  :options="locationOptions"
                  class="location-select"
                />
              </div>

              <div v-if="activeLocationDiff" class="diff-panel">
                <div class="diff-panel-title">{{ activeLocationDiff.label }}</div>
                <vue-monaco-diff-editor
                  :original="activeLocationDiff.before"
                  :modified="activeLocationDiff.after"
                  language="nginx"
                  :theme="editorTheme"
                  :options="diffOptions"
                  class="diff-editor"
                />
              </div>
            </template>
            <n-empty
              v-else
              description="本次未生成 Location 级差异"
              class="detail-empty"
            />
          </n-tab-pane>
        </n-tabs>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
  NDrawer,
  NDrawerContent,
  NEmpty,
  NSelect,
  NTabPane,
  NTabs,
} from 'naive-ui';
import { VueMonacoDiffEditor } from '@guolao/vue-monaco-editor';
import type { FileChangeLogEntry } from '@/types/nginx';

const props = defineProps<{
  show: boolean;
  entry: FileChangeLogEntry | null;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
}>();

const activeTab = ref<'file' | 'server' | 'location'>('file');
const selectedLocationIndex = ref<string>('0');

const drawerWidth = 'min(960px, calc(100vw - 32px))';

const editorTheme = computed(() =>
  document.body.classList.contains('dark') ? 'vs-dark' : 'vs',
);

const diffOptions = {
  automaticLayout: true,
  readOnly: true,
  renderSideBySide: true,
  minimap: { enabled: false },
  originalEditable: false,
  scrollBeyondLastLine: false,
  fontSize: 13,
  fontFamily: 'Consolas, PingFang SC, monospace',
};

const locationOptions = computed(() =>
  props.entry?.detail.locationDiffs.map((diff, index) => ({
    label: diff.label,
    value: String(index),
  })) ?? [],
);

const activeLocationDiff = computed(() => {
  if (!props.entry) {
    return null;
  }

  return props.entry.detail.locationDiffs[Number(selectedLocationIndex.value)] ?? null;
});

watch(
  () => props.entry?.id,
  () => {
    activeTab.value = 'file';
    selectedLocationIndex.value = '0';
  },
);

const formatTime = (timestampMs: number) => {
  const date = new Date(timestampMs);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};
</script>

<style scoped>
.detail-shell {
  height: 100%;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 12px;
}

.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border-radius: var(--radius-lg);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
}

.meta-row {
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.meta-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.meta-value {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

.detail-tabs {
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.detail-tabs :deep(.n-tabs-pane-wrapper),
.detail-tabs :deep(.n-tab-pane) {
  height: 100%;
  min-height: 0;
}

.diff-panel {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.diff-panel-title {
  font-size: 12px;
  color: var(--text-secondary);
}

.diff-editor {
  flex: 1;
  min-height: 0;
  border-radius: var(--radius-md);
  overflow: hidden;
  border: 1px solid var(--surface-border);
}

.detail-empty {
  margin-top: 36px;
}

.location-toolbar {
  margin-bottom: 10px;
}

.location-select {
  width: min(420px, 100%);
}

@media (max-width: 860px) {
  .meta-row {
    grid-template-columns: 1fr;
    gap: 6px;
  }
}
</style>
