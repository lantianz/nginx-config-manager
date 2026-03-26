<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN">
    <n-message-provider :container-style="{ top: '52px' }">
      <n-dialog-provider>
        <n-notification-provider :container-style="{ top: '52px' }">
          <GlobalNotification />

          <div class="app-shell" :class="{ dark: isDark }">
            <TitleBar :is-dark="isDark" />

            <div class="workspace">
              <aside class="sider-panel surface-card" :class="{ collapsed }">
                <div class="brand-row">
                  <div class="brand-main">
                    <div class="brand-mark">
                      <img src="../../assets/nginx-logo.svg" alt="Nginx Logo" class="brand-logo" />
                    </div>
                    <div v-if="!collapsed" class="brand-copy">
                      <span class="brand-title">Nginx 控制台</span>
                      <span class="brand-subtitle">配置与运行管理</span>
                    </div>
                  </div>
                </div>

                <n-menu
                  v-model:value="activeKey"
                  class="nav-menu"
                  :collapsed="collapsed"
                  :collapsed-width="72"
                  :collapsed-icon-size="20"
                  :options="menuOptions"
                  @update:value="handleMenuSelect"
                />

                <div class="sider-footer">
                  <n-button quaternary circle class="collapse-button" @click="collapsed = !collapsed">
                    <template #icon>
                      <n-icon :component="ChevronBackOutline" class="collapse-icon" :class="{ collapsed }" />
                    </template>
                  </n-button>
                </div>
              </aside>

              <section class="main-panel">
                <header class="topbar surface-card">
                  <div class="topbar-copy">
                    <h1>{{ currentPageTitle }}</h1>
                    <p>{{ currentPageDescription }}</p>
                  </div>

                  <div class="topbar-actions">
                    <n-tag round size="small" :type="nginxStore.status.isRunning ? 'success' : 'default'">
                      {{ nginxStore.status.isRunning ? '运行中' : '已停止' }}
                    </n-tag>
                  </div>
                </header>

                <div class="content-host">
                  <router-view v-slot="{ Component }">
                    <component :is="Component" class="page-view" />
                  </router-view>
                </div>

                <footer class="status-bar">
                  <div class="status-strip">
                    <n-tag round size="small" :type="nginxStore.status.isRunning ? 'success' : 'default'">
                      Nginx {{ nginxStore.status.isRunning ? '运行中' : '未运行' }}
                    </n-tag>
                    <span class="status-text">进程 {{ nginxStore.status.processCount }} 个</span>
                    <span class="status-text status-path allow-select">{{ settingsStore.settings.configPath || '未配置 nginx.conf 路径' }}</span>
                  </div>
                </footer>
              </section>
            </div>
          </div>
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NButton,
  NConfigProvider,
  NDialogProvider,
  NIcon,
  NMenu,
  NMessageProvider,
  NNotificationProvider,
  NTag,
  darkTheme,
  dateZhCN,
  zhCN,
} from 'naive-ui';
import {
  ChevronBackOutline,
  DocumentTextOutline,
  ListOutline,
  ServerOutline,
  SettingsOutline,
} from '@vicons/ionicons5';
import GlobalNotification from '@/components/GlobalNotification.vue';
import TitleBar from '@/components/layout/TitleBar.vue';
import { useLogStore } from '@/stores/log';
import { useNginxStore } from '@/stores/nginx';
import { useSettingsStore } from '@/stores/settings';

const router = useRouter();
const route = useRoute();
const logStore = useLogStore();
const nginxStore = useNginxStore();
const settingsStore = useSettingsStore();

const collapsed = ref(false);
const activeKey = ref('process');
const isDark = ref(false);

const theme = computed(() => (isDark.value ? darkTheme : null));

const themeOverrides = computed(() => ({
  common: {
    primaryColor: '#3b82f6',
    primaryColorHover: '#2563eb',
    primaryColorPressed: '#1d4ed8',
    primaryColorSuppl: '#60a5fa',
    infoColor: '#3b82f6',
    successColor: '#10b981',
    warningColor: '#f59e0b',
    errorColor: '#f43f5e',
    borderRadius: '12px',
    fontFamily: 'PingFang SC, Microsoft YaHei, Segoe UI, sans-serif',
  },
  Card: {
    color: 'var(--surface-bg-strong)',
    borderRadius: '16px',
  },
  Modal: {
    color: 'var(--surface-bg-strong)',
    borderRadius: '18px',
  },
  Drawer: {
    color: 'var(--surface-bg-strong)',
  },
  Layout: {
    color: 'transparent',
    siderColor: 'transparent',
    headerColor: 'transparent',
    footerColor: 'transparent',
  },
}));

const pageMeta: Record<string, { title: string; description: string }> = {
  process: {
    title: '运行与端口',
    description: '启动、停止、重载与按端口查询，保持稳定的本地运维流程。',
  },
  config: {
    title: '配置管理',
    description: '按项目分类、启停状态与关键词筛选 server，并在详情中定位 location。',
  },
  logs: {
    title: '操作日志',
    description: '查看关键操作轨迹，快速回放最近一次执行结果。',
  },
  settings: {
    title: '应用设置',
    description: '集中管理运行环境、界面外观与桌面端应用信息。',
  },
};

const currentPageTitle = computed(() => pageMeta[activeKey.value]?.title || '运行与端口');
const currentPageDescription = computed(() => pageMeta[activeKey.value]?.description || '');

const menuOptions = computed(() => [
  {
    label: '运行与端口',
    key: 'process',
    icon: () => h(NIcon, null, { default: () => h(ServerOutline) }),
  },
  {
    label: '配置管理',
    key: 'config',
    icon: () => h(NIcon, null, { default: () => h(DocumentTextOutline) }),
  },
  {
    label: '操作日志',
    key: 'logs',
    icon: () => h(NIcon, null, { default: () => h(ListOutline) }),
  },
  {
    label: '应用设置',
    key: 'settings',
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) }),
  },
]);

watch(
  () => route.path,
  (path) => {
    activeKey.value = path.slice(1) || 'process';
  },
  { immediate: true }
);

watch(
  isDark,
  (dark) => {
    document.body.classList.toggle('dark', dark);
  },
  { immediate: true }
);

watch(
  () => settingsStore.settings.logRetentionDays,
  (retentionDays, previousDays) => {
    if (!settingsStore.isLoaded || !logStore.isLoaded || retentionDays === previousDays) {
      return;
    }

    void logStore.loadPersisted(retentionDays);
  }
);

watch(
  () => settingsStore.settings.theme,
  (savedTheme) => {
    if (savedTheme === 'dark') {
      isDark.value = true;
      return;
    }

    if (savedTheme === 'light') {
      isDark.value = false;
      return;
    }

    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches;
  },
  { immediate: true }
);

const handleMenuSelect = (key: string) => {
  activeKey.value = key;
  void router.push(`/${key}`);
};

onMounted(async () => {
  if (!settingsStore.isLoaded) {
    await settingsStore.loadSettings();
  }
  if (!logStore.isLoaded) {
    await logStore.initialize();
  }
  await nginxStore.checkStatus().catch(() => undefined);
});
</script>

<style scoped>
.app-shell {
  position: relative;
  width: 100%;
  height: 100vh;
  padding-top: 40px;
  background:
    radial-gradient(circle at top left, var(--app-bg-accent), transparent 28%),
    radial-gradient(circle at 82% 14%, rgba(191, 219, 254, 0.26), transparent 20%),
    var(--app-bg);
}

.app-shell::before {
  content: '';
  position: fixed;
  right: 48px;
  bottom: 72px;
  width: 220px;
  height: 220px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.18);
  filter: blur(28px);
  pointer-events: none;
}

.workspace {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  height: calc(100vh - 40px);
  padding: 12px;
  overflow: hidden;
  background: transparent;
}

.surface-card {
  background: var(--surface-bg);
  border: 1px solid var(--surface-border);
  box-shadow: var(--surface-shadow);
  backdrop-filter: blur(18px);
}

.sider-panel {
  width: 216px;
  min-width: 216px;
  height: 100%;
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sider-panel.collapsed {
  width: 72px;
  min-width: 72px;
}

.brand-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 14px 10px;
}

.brand-main {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  flex: 1;
}

.brand-mark {
  width: 36px;
  height: 36px;
  border-radius: 12px;
  background: var(--brand-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.brand-logo {
  width: 22px;
  height: 22px;
}

.brand-copy {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.brand-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.brand-subtitle {
  font-size: 12px;
  color: var(--text-secondary);
}

.collapse-icon {
  transition: transform 0.18s ease;
}

.collapse-icon.collapsed {
  transform: rotate(180deg);
}

.nav-menu {
  flex: 1;
  min-height: 0;
  padding: 4px 8px 12px;
  background: transparent;
}

.nav-menu :deep(.n-menu-item) {
  margin-bottom: 4px;
}

.nav-menu :deep(.n-menu-item-content) {
  min-height: 46px;
  padding: 0 12px !important;
  border-radius: 14px;
  display: flex;
  align-items: center;
  transition: color 0.18s ease, transform 0.18s ease;
}

.nav-menu :deep(.n-menu-item-content::before) {
  left: 4px !important;
  right: 4px !important;
  top: 3px !important;
  bottom: 3px !important;
  border-radius: 14px !important;
}

.nav-menu :deep(.n-menu-item-content__icon) {
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.sider-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 0 12px;
}

.collapse-button {
  flex-shrink: 0;
}

.main-panel {
  min-width: 0;
  height: 100%;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 12px;
  overflow: hidden;
  background: transparent;
}

.topbar {
  min-height: 68px;
  border-radius: var(--radius-xl);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
}

.topbar-copy h1 {
  margin: 0;
  font-size: 22px;
  line-height: 1.2;
  color: var(--text-primary);
}

.topbar-copy p {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.content-host {
  min-height: 0;
  display: flex;
  overflow: hidden;
  background: transparent;
}

.page-view {
  flex: 1 1 auto;
  min-width: 0;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: transparent;
}

.status-bar {
  min-height: 44px;
  display: flex;
  align-items: center;
  border-radius: var(--radius-lg);
  padding: 0 12px;
  background: var(--surface-bg);
  border: 1px solid var(--surface-border);
  backdrop-filter: blur(16px);
}

.status-strip {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  width: 100%;
}

.status-text {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.status-path {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: right;
}

.sider-panel.collapsed .brand-row {
  justify-content: center;
  padding-left: 8px;
  padding-right: 8px;
}

.sider-panel.collapsed .brand-main {
  justify-content: center;
}

.sider-panel.collapsed .nav-menu {
  padding-left: 6px;
  padding-right: 6px;
}

.sider-panel.collapsed :deep(.n-menu-item-content) {
  padding-left: 0 !important;
  padding-right: 0 !important;
  justify-content: center;
}

.sider-panel.collapsed :deep(.n-menu-item-content__icon) {
  margin-right: 0 !important;
  width: 100%;
}

.sider-panel.collapsed :deep(.n-menu-item-content::before) {
  left: 8px !important;
  right: 8px !important;
}

.sider-panel.collapsed :deep(.n-menu-item-content-header) {
  display: none;
}

@media (max-width: 980px) {
  .workspace {
    grid-template-columns: 72px minmax(0, 1fr);
  }

  .sider-panel {
    width: 72px;
    min-width: 72px;
  }

  .topbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .topbar-actions {
    width: 100%;
    justify-content: space-between;
  }

  .status-strip {
    gap: 8px;
  }
}
</style>
