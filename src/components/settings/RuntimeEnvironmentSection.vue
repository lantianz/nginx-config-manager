<template>
  <div class="runtime-settings" :class="{ compact: props.compact }">
    <div class="setting-row">
      <div class="setting-copy">
        <div class="setting-title">Nginx 程序</div>
        <p v-if="props.showDescriptions">选择 `nginx.exe` 文件。</p>
      </div>
      <div class="setting-control">
        <n-input
          :value="localNginxPath"
          placeholder="例如: C:\\nginx-1.26.3\\nginx.exe"
          @update:value="localNginxPath = $event"
          @blur="handleBlur"
        >
          <template #prefix>
            <n-icon :component="FolderOpenOutline" />
          </template>
          <template #suffix>
            <n-button text @click="selectNginxPath">浏览</n-button>
          </template>
        </n-input>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-copy">
        <div class="setting-title">配置文件路径</div>
        <p v-if="props.showDescriptions">选择当前使用的 `nginx.conf`。</p>
      </div>
      <div class="setting-control">
        <n-input
          :value="localConfigPath"
          placeholder="例如: C:\\nginx-1.26.3\\conf\\nginx.conf"
          @update:value="localConfigPath = $event"
          @blur="handleBlur"
        >
          <template #prefix>
            <n-icon :component="DocumentTextOutline" />
          </template>
          <template #suffix>
            <n-button text @click="selectConfigPath">浏览</n-button>
          </template>
        </n-input>
      </div>
    </div>

    <div class="setting-actions" v-if="props.showActions">
      <n-button secondary :disabled="!localConfigPath" @click="openConfigFile"
        >打开配置文件</n-button
      >
      <n-button type="primary" @click="savePaths(true)">保存路径</n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { NButton, NIcon, NInput, useMessage } from "naive-ui";
import { DocumentTextOutline, FolderOpenOutline } from "@vicons/ionicons5";
import { useSettingsStore } from "@/stores/settings";

interface Props {
  compact?: boolean;
  showActions?: boolean;
  showDescriptions?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
  showActions: true,
  showDescriptions: true,
});

const emit = defineEmits<{
  saved: [];
}>();

const message = useMessage();
const settingsStore = useSettingsStore();

const localNginxPath = ref("");
const localConfigPath = ref("");

watch(
  () => settingsStore.settings,
  (value) => {
    localNginxPath.value = value.nginxPath;
    localConfigPath.value = value.configPath;
  },
  { immediate: true, deep: true },
);

const savePaths = (showMessage = false) => {
  const nextNginxPath = localNginxPath.value.trim();
  const nextConfigPath = localConfigPath.value.trim();
  const hasChanged =
    nextNginxPath !== settingsStore.settings.nginxPath ||
    nextConfigPath !== settingsStore.settings.configPath;

  if (nextNginxPath !== settingsStore.settings.nginxPath) {
    settingsStore.updateNginxPath(nextNginxPath);
  }

  if (nextConfigPath !== settingsStore.settings.configPath) {
    settingsStore.updateConfigPath(nextConfigPath);
  }

  localNginxPath.value = nextNginxPath;
  localConfigPath.value = nextConfigPath;

  if (showMessage) {
    message.success(hasChanged ? "路径设置已保存" : "路径未变更");
  }

  emit("saved");
};

const handleBlur = () => {
  savePaths(false);
};

const selectNginxPath = async () => {
  try {
    const selected = await open({
      title: "选择 Nginx 可执行文件",
      multiple: false,
      directory: false,
      filters: [
        {
          name: "Nginx 可执行文件",
          extensions: ["exe"],
        },
      ],
    });

    if (!selected) {
      return;
    }

    localNginxPath.value = selected as string;
    savePaths(true);
  } catch (error) {
    message.error(`选择文件失败: ${error}`);
  }
};

const selectConfigPath = async () => {
  try {
    const selected = await open({
      title: "选择 Nginx 配置文件",
      multiple: false,
      directory: false,
      filters: [
        {
          name: "Nginx 配置文件",
          extensions: ["conf"],
        },
      ],
    });

    if (!selected) {
      return;
    }

    localConfigPath.value = selected as string;
    savePaths(true);
  } catch (error) {
    message.error(`选择文件失败: ${error}`);
  }
};

const openConfigFile = async () => {
  if (!localConfigPath.value.trim()) {
    message.warning("请先配置配置文件路径");
    return;
  }

  try {
    const result = await invoke<string>("open_file_in_system", {
      filePath: localConfigPath.value.trim(),
    });
    message.success(result);
  } catch (error) {
    message.error(`打开文件失败: ${error}`);
  }
};
</script>

<style scoped>
.runtime-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-row {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}

.setting-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.setting-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.setting-copy p {
  margin: 0;
  font-size: 12px;
  line-height: 1.7;
  color: var(--text-secondary);
}

.setting-control {
  min-width: 0;
}

.setting-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  flex-wrap: wrap;
}

.runtime-settings.compact .setting-row {
  grid-template-columns: 1fr;
  gap: 10px;
}

.runtime-settings.compact .setting-copy p {
  font-size: 12px;
}

@media (max-width: 860px) {
  .setting-row {
    grid-template-columns: 1fr;
    gap: 10px;
  }
}
</style>
