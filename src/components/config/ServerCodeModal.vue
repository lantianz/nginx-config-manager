<template>
  <n-modal
    :show="show"
    preset="card"
    :title="title"
    class="server-code-modal"
    :style="{ width: 'min(800px, calc(100vw - 32px))' }"
    :bordered="false"
    @update:show="handleShowChange"
  >
    <div class="server-code-shell">
      <aside class="location-panel">
        <div class="location-panel-header">
          <span class="location-panel-title">Location 导航</span>
          <n-tag size="small" type="info" round>
            {{ filteredLocations.length }}/{{ locations.length }}
          </n-tag>
        </div>

        <div class="location-panel-toolbar" v-if="locations.length > 0">
          <n-input
            v-model:value="locationQuery"
            clearable
            size="small"
            placeholder="搜索路径或行号"
          >
            <template #prefix>
              <n-icon :component="SearchOutline" />
            </template>
          </n-input>
        </div>

        <div class="location-panel-content">
          <n-scrollbar class="location-panel-body">
            <n-empty v-if="locations.length === 0" size="small" description="当前 Server 没有 location" />
            <n-empty v-else-if="filteredLocations.length === 0" size="small" description="没有匹配的 location" />

            <div v-else class="location-list">
              <button
                v-for="location in filteredLocations"
                :key="location.id"
                type="button"
                class="location-jump-button"
                :class="{ active: activeLocationId === location.id }"
                @click="jumpToLocation(location)"
              >
                <div class="location-jump-content">
                  <span class="location-jump-path">{{ buildLocationLabel(location) }}</span>
                  <span class="location-jump-line">第 {{ location.relativeStartLine }} 行</span>
                </div>
              </button>
            </div>
          </n-scrollbar>
        </div>
      </aside>

      <div class="editor-panel">
        <div class="editor-panel-header">
          <span class="editor-panel-title">
            {{ activeLocation ? buildLocationLabel(activeLocation) : (readOnly ? 'Server 详情' : 'Server 编辑') }}
          </span>
          <n-tag v-if="activeLocation" size="small" round>
            第 {{ activeLocation.relativeStartLine }} 行
          </n-tag>
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
import { NButton, NEmpty, NIcon, NInput, NModal, NScrollbar, NSpace, NTag } from 'naive-ui';
import { SearchOutline } from '@vicons/ionicons5';
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
const locationQuery = ref('');
const activeLocationId = ref<string | null>(null);

const filteredLocations = computed(() => {
  const keyword = locationQuery.value.trim().toLowerCase();
  if (!keyword) {
    return props.locations;
  }

  return props.locations.filter((location) => {
    const searchableText = [
      location.path,
      location.modifier || '',
      String(location.relativeStartLine),
    ]
      .join(' ')
      .toLowerCase();

    return searchableText.includes(keyword);
  });
});

const activeLocation = computed(() =>
  props.locations.find((location) => location.id === activeLocationId.value) ?? null
);

const categoryInput = computed({
  get: () => props.categoryName,
  set: (value: string) => {
    emit('update:categoryName', value);
  },
});

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
      locationQuery.value = '';
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
  filteredLocations,
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
  max-width: 800px;
  max-height: min(82vh, 760px);
}

.server-code-shell {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 12px;
  height: min(62vh, 560px);
  min-height: 460px;
  overflow: hidden;
}

.location-panel {
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--surface-bg-soft);
  border: 1px solid var(--surface-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.location-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 14px 14px 10px;
}

.location-panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.location-panel-toolbar {
  padding: 0 14px 12px;
}

.location-panel-content {
  flex: 1;
  min-height: 0;
  padding: 0 8px 12px 12px;
  overflow: hidden;
}

.location-panel-body {
  height: 100%;
}

.location-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.location-jump-button {
  width: 100%;
  padding: 12px;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.76);
  text-align: left;
  transition: border-color 0.18s ease, background 0.18s ease, transform 0.18s ease;
}

.location-jump-button:hover {
  border-color: rgba(59, 130, 246, 0.22);
  background: rgba(255, 255, 255, 0.94);
  transform: translateY(-1px);
}

.location-jump-button.active {
  border-color: rgba(59, 130, 246, 0.28);
  background: rgba(59, 130, 246, 0.12);
  box-shadow: 0 0 0 1px rgba(59, 130, 246, 0.08);
}

.location-jump-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.location-jump-path {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.5;
  color: var(--text-primary);
  word-break: break-all;
}

.location-jump-line {
  font-size: 12px;
  color: var(--text-secondary);
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
  grid-template-columns: 120px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
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
    grid-template-columns: 1fr;
    height: min(72vh, 620px);
  }

  .location-panel {
    max-height: 220px;
  }

  .category-section {
    grid-template-columns: 1fr;
    gap: 8px;
  }
}
</style>
