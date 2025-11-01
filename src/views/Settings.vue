<template>
  <div class="settings-page">
    <n-space vertical :size="16">
      <!-- 关于应用 -->
      <n-card title="关于应用" :bordered="false">
        <n-space vertical :size="16">
          <n-descriptions :column="1" label-placement="left" bordered>
            <n-descriptions-item label="应用名称">
              <n-text strong>Nginx 管理工具</n-text>
            </n-descriptions-item>
            <n-descriptions-item label="应用版本">
              <n-tag type="info">v{{ appVersion }}</n-tag>
            </n-descriptions-item>
            <n-descriptions-item label="应用描述">
              一个基于 Tauri 的 Nginx 配置文件管理工具，提供进程管理、日志查看、配置编辑等功能
            </n-descriptions-item>
            <n-descriptions-item label="技术栈">
              <n-space>
                <n-tag size="small">Vue 3</n-tag>
                <n-tag size="small">Tauri 2</n-tag>
                <n-tag size="small">Naive UI</n-tag>
                <n-tag size="small">TypeScript</n-tag>
                <n-tag size="small">Rust</n-tag>
              </n-space>
            </n-descriptions-item>
            <n-descriptions-item label="开发者">
              <n-button text tag="a" href="https://github.com/lantianz" target="_blank" @click="openGitHub" style="padding: 0; font-size: 14px;">
                <template #icon>
                  <n-icon :component="LogoGithub" />
                </template>
                lantianz
              </n-button>
            </n-descriptions-item>
          </n-descriptions>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import {
  NCard,
  NSpace,
  NText,
  NDescriptions,
  NDescriptionsItem,
  NTag,
  NButton,
  NIcon,
} from 'naive-ui';
import { LogoGithub } from '@vicons/ionicons5';
import { openUrl } from '@tauri-apps/plugin-opener';

// 应用版本
const appVersion = ref('0.2.0');

// 打开 GitHub 链接
const openGitHub = async (e: Event) => {
  e.preventDefault();
  try {
    await openUrl('https://github.com/lantianz');
  } catch (error) {
    console.error('打开链接失败:', error);
  }
};
</script>

<style scoped>
.settings-page {
  padding: 0;
}
</style>
