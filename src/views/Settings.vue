<template>
  <div class="settings-page">
    <n-space vertical :size="16">
      <!-- 关于应用 -->
      <n-card title="关于应用" :bordered="false">
        <n-space vertical :size="16">
          <n-descriptions :column="1" label-placement="left" bordered>
            <n-descriptions-item label="应用名称">
              <n-text strong>Nginx 配置管理器</n-text>
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
              lantz
            </n-descriptions-item>
          </n-descriptions>
        </n-space>
      </n-card>

      <!-- 更新日志 -->
      <n-card title="更新日志" :bordered="false">
        <n-timeline>
          <n-timeline-item
            v-for="log in changelog"
            :key="log.version"
            :type="log.type"
            :title="`版本 ${log.version}`"
            :time="log.date"
          >
            <template #header>
              <n-space align="center">
                <n-text strong>版本 {{ log.version }}</n-text>
                <n-tag :type="log.type" size="small">{{ log.date }}</n-tag>
              </n-space>
            </template>

            <n-space vertical :size="8">
              <!-- 新增功能 -->
              <div v-if="log.features && log.features.length > 0">
                <n-text strong style="color: #18a058">✨ 新增功能</n-text>
                <ul style="margin: 8px 0; padding-left: 20px">
                  <li v-for="(feature, index) in log.features" :key="index">
                    {{ feature }}
                  </li>
                </ul>
              </div>

              <!-- 修复问题 -->
              <div v-if="log.fixes && log.fixes.length > 0">
                <n-text strong style="color: #d03050">🐛 修复问题</n-text>
                <ul style="margin: 8px 0; padding-left: 20px">
                  <li v-for="(fix, index) in log.fixes" :key="index">
                    {{ fix }}
                  </li>
                </ul>
              </div>

              <!-- 改进优化 -->
              <div v-if="log.improvements && log.improvements.length > 0">
                <n-text strong style="color: #2080f0">🚀 改进优化</n-text>
                <ul style="margin: 8px 0; padding-left: 20px">
                  <li v-for="(improvement, index) in log.improvements" :key="index">
                    {{ improvement }}
                  </li>
                </ul>
              </div>
            </n-space>
          </n-timeline-item>
        </n-timeline>
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
  NTimeline,
  NTimelineItem,
} from 'naive-ui';

// 应用版本
const appVersion = ref('0.1.0');

// 更新日志数据
const changelog = ref([
  {
    version: '0.1.0',
    date: '2025-10-31',
    type: 'success' as const,
    features: [
      'Nginx 进程管理：启动、停止、重启、重载配置',
      'Nginx 日志查看：实时查看访问日志和错误日志，支持日志级别过滤',
      'Nginx 配置管理：解析和显示 Server 块、Location 块',
      '配置文件编辑：使用 Monaco Editor 编辑配置，支持 Nginx 语法高亮',
      '配置搜索：全局搜索端口号、域名、Location 路径',
      '卡片内 Location 搜索：在每个 Server 卡片内独立搜索 Location',
      '新增/编辑/删除 Server 块：文本编辑方式，保持原始格式',
      'Location 标签 Hover 提示：显示完整的 Location 配置',
      '打开配置文件：使用系统默认程序打开配置文件',
    ],
    improvements: [
    ],
    fixes: [
    ],
  },
]);
</script>

<style scoped>
.settings-page {
  padding: 0;
}

ul {
  list-style-type: disc;
}

li {
  margin: 4px 0;
  line-height: 1.6;
}
</style>
