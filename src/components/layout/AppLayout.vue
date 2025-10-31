<template>
  <n-config-provider :theme="theme" :locale="zhCN" :date-locale="dateZhCN">
    <n-message-provider :container-style="{ top: '40px' }">
      <n-dialog-provider>
        <n-notification-provider :container-style="{ top: '40px' }">
          <!-- 自定义标题栏 -->
          <TitleBar :is-dark="isDark" />

          <n-layout has-sider style="height: 100vh; padding-top: 32px">
            <!-- 侧边栏 -->
            <n-layout-sider bordered collapse-mode="width" :collapsed-width="64" :width="200" show-trigger
              @collapse="collapsed = true" @expand="collapsed = false">
              <div class="logo-container">
                <n-icon :size="logoWidth">
                  <img :style="{ width: logoWidth + 'px', height: logoWidth + 'px' }" src="./../../assets/nginx-logo.svg" alt="" class="logo">
                </n-icon>
              </div>

              <n-menu v-model:value="activeKey" :collapsed="collapsed" :collapsed-width="64" :collapsed-icon-size="22"
                :options="menuOptions" @update:value="handleMenuSelect" />
            </n-layout-sider>

            <!-- 主内容区 -->
            <n-layout style="height: calc(100vh - 32px); display: flex; flex-direction: column">
              <n-layout-header bordered style="
                  height: 60px;
                  padding: 0 16px;
                  display: flex;
                  align-items: center;
                  justify-content: space-between;
                  flex-shrink: 0;
                ">
                <div class="header-title">
                  <h2 style="margin: 0">{{ currentPageTitle }}</h2>
                </div>
                <div class="header-actions">
                  <n-button quaternary circle @click="toggleTheme">
                    <n-icon :component="isDark ? SunnyOutline : MoonOutline" />
                  </n-button>
                </div>
              </n-layout-header>

              <n-layout position="absolute" style="top: 60px; bottom: 40px; width: 100%; height: calc(100vh - 132px);">
                <router-view style="padding: 16px;" />
              </n-layout>

              <n-layout-footer position="absolute" bordered style="
                  height: 40px;
                  padding: 0 16px;
                  display: flex;
                  align-items: center;
                  flex-shrink: 0;
                ">
                <n-space>
                  <span>Nginx运行状态:
                    <n-tag :bordered="false" :type="nginxStatus === '运行中' ? 'success' : 'error'">
                      {{ nginxStatus }}
                    </n-tag>
                  </span>
                  <n-divider vertical />
                </n-space>
              </n-layout-footer>
            </n-layout>
          </n-layout>
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, computed, h, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NLayout,
  NLayoutSider,
  NLayoutHeader,
  NLayoutFooter,
  NMenu,
  NIcon,
  NButton,
  NSpace,
  NDivider,
  NTag,
  darkTheme,
  zhCN,
  dateZhCN,
} from "naive-ui";
import {
  ServerOutline,
  SettingsOutline,
  DocumentTextOutline,
  ListOutline,
  MoonOutline,
  SunnyOutline,
} from "@vicons/ionicons5";
import TitleBar from "./TitleBar.vue";
import { useNginxStore } from "../../stores/nginx";
import { useSettingsStore } from "../../stores/settings";

const router = useRouter();
const nginxStore = useNginxStore();
const settingsStore = useSettingsStore();

// logo宽度
const logoWidth = computed(() => (collapsed.value ? 40 : 64));

// 状态
const collapsed = ref(false);
const activeKey = ref<string>("process");
const isDark = ref(false);

// 主题
const theme = computed(() => (isDark.value ? darkTheme : null));

// 切换主题
const toggleTheme = () => {
  isDark.value = !isDark.value;
  settingsStore.updateTheme(isDark.value ? "dark" : "light");
};

// 监听主题变化,同步到 body 的 class
watch(
  isDark,
  (dark) => {
    if (dark) {
      document.body.classList.add("dark");
    } else {
      document.body.classList.remove("dark");
    }
  },
  { immediate: true }
);

// 初始化时加载主题设置
onMounted(async () => {
  await settingsStore.loadSettings();
  const savedTheme = settingsStore.settings.theme;
  if (savedTheme === "dark") {
    isDark.value = true;
  } else if (savedTheme === "light") {
    isDark.value = false;
  } else {
    // auto: 根据系统主题
    isDark.value = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
});

// Nginx 状态
const nginxStatus = computed(() => {
  return nginxStore.status.isRunning ? "运行中" : "已停止";
});

// 当前页面标题
const currentPageTitle = computed(() => {
  const titles: Record<string, string> = {
    process: "进程管理",
    config: "配置管理",
    logs: "操作日志",
    settings: "应用设置",
  };
  return titles[activeKey.value] || "进程管理";
});

// 菜单选项
const menuOptions = computed(() => [
  {
    label: "进程管理",
    key: "process",
    icon: () => h(NIcon, null, { default: () => h(ServerOutline) }),
  },
  {
    label: "配置管理",
    key: "config",
    icon: () => h(NIcon, null, { default: () => h(DocumentTextOutline) }),
  },
  {
    label: "操作日志",
    key: "logs",
    icon: () => h(NIcon, null, { default: () => h(ListOutline) }),
  },
  {
    label: "应用设置",
    key: "settings",
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) }),
  },
]);

// 处理菜单选择
const handleMenuSelect = (key: string) => {
  activeKey.value = key;
  router.push(`/${key}`);
};

// 监听路由变化
router.afterEach((to) => {
  const path = to.path.slice(1);
  if (path) {
    activeKey.value = path;
  }
});

// 初始化
const init = async () => {
  await settingsStore.loadSettings();
  await nginxStore.checkStatus();

  // 设置初始主题
  isDark.value = settingsStore.settings.theme === "dark";
};

init();
</script>

<style scoped>
.logo-container {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--n-border-color);
}

.logo {
  transition: all 0.2s linear;
}

.header-title h2 {
  font-size: 20px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
