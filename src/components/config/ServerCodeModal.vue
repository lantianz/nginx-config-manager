<template>
  <n-modal
    :show="show"
    preset="card"
    :title="title"
    class="server-code-modal"
    :style="{ width: 'min(1100px, calc(100vw - 32px))' }"
    :bordered="false"
    @update:show="handleShowChange"
  >
    <div class="server-code-shell">
      <div class="editor-toolbar">
        <div class="editor-toolbar-main">
          <n-select
            v-if="locations.length > 0"
            :value="activeLocationId"
            filterable
            clearable
            :options="locationOptions"
            placeholder="搜索并跳转到 location"
            class="location-select"
            @update:value="handleLocationSelect"
          />
          <div v-else class="location-empty">当前 Server 没有 location</div>
        </div>

        <div class="editor-toolbar-meta">
          <n-tag size="small" type="info" round>
            {{ locations.length }} 个 location
          </n-tag>
          <n-tag v-if="activeLocation" size="small" round>
            第 {{ activeLocation.relativeStartLine }} 行
          </n-tag>
        </div>
      </div>

      <div v-if="categoryMode !== 'hidden'" class="category-section">
        <div class="category-meta">
          <span class="category-label">项目分类</span>
        </div>

        <div class="category-control">
          <n-input
            v-if="categoryMode === 'edit'"
            v-model:value="categoryInput"
            maxlength="40"
            clearable
            placeholder="输入项目分类，留空表示不设置"
          />
          <div v-else class="category-display" :class="{ empty: !categoryName }">
            {{ categoryName || '未设置分类' }}
          </div>
        </div>
      </div>

      <div class="editor-panel">
        <div class="editor-panel-header">
          <span class="editor-panel-title">
            {{ activeLocation ? buildLocationLabel(activeLocation) : (readOnly ? 'Server 详情' : 'Server 编辑') }}
          </span>
          <span class="editor-panel-subtitle">
            {{
              activeLocation
                ? `第 ${activeLocation.relativeStartLine} - ${activeLocation.relativeEndLine} 行`
                : '支持直接查看、格式化与保存当前 Server 内容'
            }}
          </span>
        </div>

        <div class="editor-panel-content">
          <vue-monaco-editor
            :value="content"
            language="nginx"
            :theme="editorTheme"
            :options="editorOptions"
            class="editor-instance"
            @update:value="handleContentChange"
            @mount="handleEditorMount"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <n-space justify="space-between">
        <n-button v-if="!readOnly" secondary @click="$emit('format')">格式化</n-button>
        <div v-else />
        <n-space>
          <n-button @click="handleShowChange(false)">关闭</n-button>
          <n-button v-if="!readOnly" type="primary" @click="$emit('save')">保存</n-button>
        </n-space>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, shallowRef, watch } from 'vue';
import { NButton, NInput, NModal, NSelect, NSpace, NTag } from 'naive-ui';
import { VueMonacoEditor } from '@guolao/vue-monaco-editor';
import type { LocationBlock } from '@/types/config';

interface Props {
  show: boolean;
  title: string;
  content: string;
  locations: LocationBlock[];
  readOnly?: boolean;
  categoryName?: string;
  categoryMode?: 'hidden' | 'display' | 'edit';
}

const props = withDefaults(defineProps<Props>(), {
  readOnly: false,
  categoryName: '',
  categoryMode: 'hidden',
});

const emit = defineEmits<{
  'update:show': [value: boolean];
  'update:content': [value: string];
  'update:categoryName': [value: string];
  save: [];
  format: [];
}>();

interface EditorRangeLike {
  startLineNumber: number;
  startColumn: number;
  endLineNumber: number;
  endColumn: number;
}

interface EditorDecorationLike {
  range: EditorRangeLike;
  options: {
    className?: string;
    isWholeLine?: boolean;
    linesDecorationsClassName?: string;
  };
}

interface EditorDecorationsCollectionLike {
  set: (decorations: readonly EditorDecorationLike[]) => string[];
  clear: () => void;
}

interface CodeEditorLike {
  revealRangeInCenter: (range: EditorRangeLike) => void;
  setSelection: (selection: EditorRangeLike, source?: string) => void;
  focus: () => void;
  createDecorationsCollection?: (
    decorations?: EditorDecorationLike[]
  ) => EditorDecorationsCollectionLike;
}

const editorInstance = shallowRef<CodeEditorLike | null>(null);
const highlightCollection = shallowRef<EditorDecorationsCollectionLike | null>(null);
const activeLocationId = ref<string | null>(null);

const activeLocation = computed(() =>
  props.locations.find((location) => location.id === activeLocationId.value) ?? null
);

const categoryInput = computed({
  get: () => props.categoryName,
  set: (value: string) => {
    emit('update:categoryName', value);
  },
});

const locationOptions = computed(() =>
  props.locations.map((location) => ({
    label: `${buildLocationLabel(location)} · 第 ${location.relativeStartLine} 行`,
    value: location.id,
  }))
);

const editorTheme = computed(() => (document.body.classList.contains('dark') ? 'vs-dark' : 'vs'));

const editorOptions = computed(() => ({
  automaticLayout: true,
  minimap: { enabled: false },
  fontSize: 14,
  fontFamily: 'Consolas, PingFang SC, monospace',
  lineNumbers: 'on' as const,
  scrollBeyondLastLine: false,
  readOnly: props.readOnly,
  roundedSelection: true,
  smoothScrolling: true,
  padding: {
    top: 12,
    bottom: 18,
    left: 12,
  },
}));

watch(
  () => props.show,
  async (show) => {
    if (show) {
      activeLocationId.value = props.locations[0]?.id ?? null;
      await nextTick();
      if (props.locations[0]) {
        jumpToLocation(props.locations[0]);
      }
      return;
    }

    highlightCollection.value?.clear();
  }
);

watch(
  () => props.locations,
  (locations) => {
    if (locations.some((location) => location.id === activeLocationId.value)) {
      return;
    }

    activeLocationId.value = locations[0]?.id ?? null;
  },
  { immediate: true }
);

const buildLocationLabel = (location: LocationBlock) =>
  `${location.modifier ? `${location.modifier} ` : ''}${location.path}`.trim();

const handleShowChange = (value: boolean) => {
  emit('update:show', value);
};

const handleContentChange = (value: string) => {
  emit('update:content', value);
};

const handleEditorMount = (instance: CodeEditorLike) => {
  editorInstance.value = instance;
  highlightCollection.value = instance.createDecorationsCollection?.() ?? null;

  if (activeLocation.value) {
    jumpToLocation(activeLocation.value);
  }
};

const handleLocationSelect = (locationId: string | null) => {
  activeLocationId.value = locationId;

  if (!locationId) {
    highlightCollection.value?.clear();
    return;
  }

  const location = props.locations.find((item) => item.id === locationId);
  if (!location) {
    return;
  }

  jumpToLocation(location);
};

const jumpToLocation = (location: LocationBlock) => {
  activeLocationId.value = location.id;

  if (!editorInstance.value) {
    return;
  }

  const range: EditorRangeLike = {
    startLineNumber: location.relativeStartLine,
    startColumn: 1,
    endLineNumber: Math.max(location.relativeStartLine, location.relativeEndLine),
    endColumn: 1,
  };

  editorInstance.value.revealRangeInCenter(range);
  editorInstance.value.setSelection(range, 'location-jump');
  highlightCollection.value?.set([
    {
      range,
      options: {
        isWholeLine: true,
        className: 'location-line-highlight',
        linesDecorationsClassName: 'location-line-glyph',
      },
    },
  ]);
  editorInstance.value.focus();
};
</script>

<style scoped>
.server-code-modal {
  max-width: 1100px;
  max-height: min(86vh, 840px);
}

.server-code-shell {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 12px;
  height: min(72vh, 680px);
  min-height: 500px;
  overflow: hidden;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border-radius: var(--radius-lg);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
  flex-wrap: wrap;
}

.editor-toolbar-main {
  min-width: 0;
  flex: 1 1 360px;
}

.location-select {
  width: min(460px, 100%);
}

.location-empty {
  min-height: 38px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-radius: var(--radius-md);
  border: 1px solid var(--surface-border);
  background: var(--surface-bg-strong);
  color: var(--text-secondary);
  font-size: 13px;
}

.editor-toolbar-meta {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.editor-panel {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border-radius: var(--radius-lg);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
  overflow: hidden;
}

.editor-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.editor-panel-title {
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-panel-subtitle {
  font-size: 12px;
  color: var(--text-secondary);
}

.editor-panel-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  border-radius: var(--radius-md);
  border: 1px solid var(--surface-border);
  background: var(--surface-bg-strong);
}

.category-section {
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr);
  gap: 12px;
  align-items: center;
  padding: 12px;
  border-radius: var(--radius-md);
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
}

.category-meta {
  display: flex;
  flex-direction: column;
}

.category-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.category-control {
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.category-display {
  min-height: 38px;
  padding: 9px 12px;
  border-radius: 12px;
  border: 1px solid var(--surface-border);
  background: var(--surface-bg-strong);
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-primary);
  word-break: break-all;
}

.category-display.empty {
  color: var(--text-secondary);
}

.editor-instance {
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.server-code-modal :deep(.n-card__content) {
  padding: 16px !important;
  overflow: hidden;
}

.server-code-modal :deep(.n-card__footer) {
  padding: 12px 16px 16px !important;
}

.server-code-modal :deep(.location-line-highlight) {
  background: rgba(59, 130, 246, 0.12);
  border-top: 1px solid rgba(59, 130, 246, 0.16);
  border-bottom: 1px solid rgba(59, 130, 246, 0.16);
}

.server-code-modal :deep(.location-line-glyph) {
  border-left: 3px solid rgba(59, 130, 246, 0.88);
  margin-left: 4px;
}

@media (max-width: 860px) {
  .server-code-shell {
    height: min(78vh, 720px);
  }

  .editor-toolbar {
    align-items: flex-start;
  }

  .location-select {
    width: 100%;
  }

  .category-section {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .editor-panel-header {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
