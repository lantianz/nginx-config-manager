<template>
  <div class="title-bar" :class="{ dark: isDark }">
    <div class="title-bar-drag-region" data-tauri-drag-region>
      <div class="title-bar-left">
        <div class="title-bar-logo-wrap">
          <img src="../../assets/nginx-logo.svg" alt="Logo" class="title-bar-icon" />
        </div>
        <span class="title-bar-title">Nginx 管理工具</span>
      </div>
    </div>

    <div class="title-bar-right">
      <button class="title-bar-button" @click="minimizeWindow" title="最小化">
        <n-icon :size="15">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor" d="M20 14H4v-2h16" />
          </svg>
        </n-icon>
      </button>
      <button class="title-bar-button" @click="toggleMaximize" :title="isMaximized ? '还原' : '最大化'">
        <n-icon :size="15">
          <svg v-if="!isMaximized" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor" d="M4 4h16v16H4V4m2 2v12h12V6H6Z" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor" d="M4 8h4V4h12v12h-4v4H4V8m12 0v6h2V6h-8v2h6m-2 4H6v6h8v-6Z" />
          </svg>
        </n-icon>
      </button>
      <button class="title-bar-button close-button" @click="closeWindow" title="关闭">
        <n-icon :size="15">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor" d="M19 6.41L17.59 5L12 10.59L6.41 5L5 6.41L10.59 12L5 17.59L6.41 19L12 13.41L17.59 19L19 17.59L13.41 12L19 6.41Z" />
          </svg>
        </n-icon>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { NIcon } from 'naive-ui';

defineProps<{
  isDark: boolean;
}>();

const appWindow = getCurrentWebviewWindow();
const isMaximized = ref(false);

const minimizeWindow = async () => {
  try {
    await appWindow.minimize();
  } catch (error) {
    console.error('最小化失败:', error);
  }
};

const toggleMaximize = async () => {
  try {
    await appWindow.toggleMaximize();
  } catch (error) {
    console.error('切换最大化失败:', error);
  }
};

const closeWindow = async () => {
  try {
    await appWindow.close();
  } catch (error) {
    console.error('关闭窗口失败:', error);
  }
};

let unlistenResize: (() => void) | null = null;

onMounted(async () => {
  try {
    isMaximized.value = await appWindow.isMaximized();
    unlistenResize = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  } catch (error) {
    console.error('初始化标题栏失败:', error);
  }
});

onUnmounted(() => {
  unlistenResize?.();
});
</script>

<style scoped>
.title-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  background: rgba(255, 255, 255, 0.74);
  border-bottom: 1px solid var(--surface-border);
  backdrop-filter: blur(18px);
}

.title-bar.dark {
  background: rgba(15, 23, 42, 0.82);
}

.title-bar-drag-region {
  flex: 1;
  height: 100%;
  display: flex;
  align-items: center;
}

.title-bar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-left: 12px;
}

.title-bar-logo-wrap {
  width: 22px;
  height: 22px;
  border-radius: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--brand-soft);
}

.title-bar-icon {
  width: 14px;
  height: 14px;
  pointer-events: none;
}

.title-bar-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  pointer-events: none;
}

.title-bar-right {
  display: flex;
  height: 100%;
}

.title-bar-button {
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease;
}

.title-bar-button:hover {
  background: rgba(148, 163, 184, 0.16);
  color: var(--text-primary);
}

.title-bar-button.close-button:hover {
  background: #ef4444;
  color: #ffffff;
}

.title-bar-button:active {
  background: rgba(148, 163, 184, 0.24);
}
</style>
