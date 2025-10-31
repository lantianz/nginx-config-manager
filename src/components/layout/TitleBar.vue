<template>
  <div class="title-bar" :class="{ dark: isDark }">
    <div class="title-bar-drag-region" data-tauri-drag-region>
      <div class="title-bar-left">
        <img src="../../assets/nginx-logo.svg" alt="Logo" class="title-bar-icon" />
        <span class="title-bar-title">Nginx 管理工具</span>
      </div>
    </div>
    <div class="title-bar-right">
      <button class="title-bar-button" @click="minimizeWindow" title="最小化">
        <n-icon :size="16">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor" d="M20 14H4v-2h16" />
          </svg>
        </n-icon>
      </button>
      <button class="title-bar-button" @click="toggleMaximize" :title="isMaximized ? '还原' : '最大化'">
        <n-icon :size="16">
          <svg v-if="!isMaximized" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor" d="M4 4h16v16H4V4m2 2v12h12V6H6Z" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor"
              d="M4 8h4V4h12v12h-4v4H4V8m12 0v6h2V6h-8v2h6m-2 4H6v6h8v-6Z" />
          </svg>
        </n-icon>
      </button>
      <button class="title-bar-button close-button" @click="closeWindow" title="关闭">
        <n-icon :size="16">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path fill="currentColor"
              d="M19 6.41L17.59 5L12 10.59L6.41 5L5 6.41L10.59 12L5 17.59L6.41 19L12 13.41L17.59 19L19 17.59L13.41 12L19 6.41Z" />
          </svg>
        </n-icon>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { NIcon } from 'naive-ui';

// Props
defineProps<{
  isDark: boolean;
}>();

const appWindow = getCurrentWebviewWindow();
const isMaximized = ref(false);

// 最小化窗口
const minimizeWindow = async () => {
  try {
    await appWindow.minimize();
    console.log('窗口已最小化');
  } catch (error) {
    console.error('最小化失败:', error);
  }
};

// 切换最大化/还原
const toggleMaximize = async () => {
  try {
    await appWindow.toggleMaximize();
    console.log('切换最大化状态');
  } catch (error) {
    console.error('切换最大化失败:', error);
  }
};

// 关闭窗口
const closeWindow = async () => {
  try {
    await appWindow.close();
    console.log('窗口已关闭');
  } catch (error) {
    console.error('关闭窗口失败:', error);
  }
};

// 监听窗口最大化状态变化
let unlistenResize: (() => void) | null = null;

onMounted(async () => {
  try {
    // 获取初始最大化状态
    isMaximized.value = await appWindow.isMaximized();
    console.log('初始最大化状态:', isMaximized.value);

    // 监听窗口大小变化
    unlistenResize = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
      console.log('窗口大小变化，最大化状态:', isMaximized.value);
    });
  } catch (error) {
    console.error('初始化标题栏失败:', error);
  }
});

onUnmounted(() => {
  if (unlistenResize) {
    unlistenResize();
  }
});
</script>

<style scoped>
.title-bar {
  height: 32px;
  background-color: #ffffff;
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  -webkit-user-select: none;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  border-bottom: 1px solid #e0e0e0;
  transition: background-color 0.3s, border-color 0.3s;
}

.title-bar.dark {
  background-color: #1a1a1a;
  border-bottom-color: #333333;
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
  padding-left: 12px;
  height: 100%;
}

.title-bar-icon {
  width: 16px;
  height: 16px;
  margin-right: 8px;
  pointer-events: none;
}

.title-bar-title {
  font-size: 12px;
  font-weight: 500;
  color: #333333;
  transition: color 0.3s;
  pointer-events: none;
}

.title-bar.dark .title-bar-title {
  color: #e0e0e0;
}

.title-bar-right {
  display: flex;
  height: 100%;
}

.title-bar-button {
  width: 46px;
  height: 100%;
  border: none;
  background-color: transparent;
  color: #333333;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s, color 0.3s;
  padding: 0;
}

.title-bar.dark .title-bar-button {
  color: #e0e0e0;
}

.title-bar-button:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.title-bar.dark .title-bar-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.title-bar-button.close-button:hover {
  background-color: #e81123;
  color: #ffffff;
}

.title-bar.dark .title-bar-button.close-button:hover {
  background-color: #e81123;
  color: #ffffff;
}

.title-bar-button:active {
  background-color: rgba(0, 0, 0, 0.1);
}

.title-bar.dark .title-bar-button:active {
  background-color: rgba(255, 255, 255, 0.15);
}

.title-bar-button.close-button:active {
  background-color: #c50f1f;
}
</style>

