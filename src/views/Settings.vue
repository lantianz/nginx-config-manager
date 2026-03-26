<template>
  <div class="settings-page">
    <div class="settings-scroll">
      <section class="settings-card">
        <div class="section-header">
          <h3>基本设置</h3>
        </div>

        <RuntimeEnvironmentSection :show-descriptions="false" />

        <div class="section-divider" />

        <div class="preference-row">
          <div class="preference-title">主题模式</div>
          <n-radio-group v-model:value="themeModel" size="medium" name="theme-mode" class="theme-selector">
            <n-radio-button value="light">浅色</n-radio-button>
            <n-radio-button value="dark">深色</n-radio-button>
            <n-radio-button value="auto">跟随系统</n-radio-button>
          </n-radio-group>
        </div>

        <div class="section-divider" />

        <div class="preference-row preference-row-start">
          <div class="preference-title">日志保留</div>
          <div class="preference-content">
            <n-radio-group
              :value="logRetentionMode"
              size="medium"
              name="log-retention-mode"
              class="theme-selector"
              @update:value="handleRetentionModeChange"
            >
              <n-radio-button value="7">7 天</n-radio-button>
              <n-radio-button value="30">30 天</n-radio-button>
              <n-radio-button value="custom">自定义</n-radio-button>
            </n-radio-group>

            <div class="custom-retention-row">
              <n-input-number
                :value="customRetentionDays"
                :min="1"
                :max="365"
                :precision="0"
                :disabled="logRetentionMode !== 'custom'"
                class="retention-input"
                @update:value="handleCustomRetentionChange"
              />
              <span class="retention-hint">支持 1-365 天，默认保留 30 天。</span>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-card">
        <div class="section-header">
          <h3>关于应用</h3>
        </div>

        <div class="about-list">
          <div class="about-row">
            <span class="about-label">版本</span>
            <div class="about-value">
              <n-tag type="info" round>v{{ appVersion }}</n-tag>
            </div>
          </div>

          <div class="about-row">
            <span class="about-label">技术栈</span>
            <div class="about-value">
              <n-space>
                <n-tag size="small" round>Vue 3</n-tag>
                <n-tag size="small" round>Tauri 2</n-tag>
                <n-tag size="small" round>Naive UI</n-tag>
                <n-tag size="small" round>TypeScript</n-tag>
                <n-tag size="small" round>Rust</n-tag>
              </n-space>
            </div>
          </div>

          <div class="about-row">
            <span class="about-label">开发者</span>
            <div class="about-value">
              <n-button text tag="a" href="https://github.com/lantianz" target="_blank" @click="openGitHub" class="link-button">
                <template #icon>
                  <n-icon :component="LogoGithub" />
                </template>
                lantianz
              </n-button>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { NButton, NIcon, NInputNumber, NRadioButton, NRadioGroup, NSpace, NTag } from 'naive-ui';
import { LogoGithub } from '@vicons/ionicons5';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import RuntimeEnvironmentSection from '@/components/settings/RuntimeEnvironmentSection.vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();
const appVersion = ref('');
const logRetentionMode = ref<'7' | '30' | 'custom'>('30');
const customRetentionDays = ref(30);

const sanitizeRetentionDays = (days: number) => {
  if (!Number.isFinite(days)) {
    return 30;
  }

  return Math.min(365, Math.max(1, Math.round(days)));
};

const themeModel = computed({
  get: () => settingsStore.settings.theme,
  set: (value: 'light' | 'dark' | 'auto') => {
    settingsStore.updateTheme(value);
  },
});

const syncRetentionState = (days: number) => {
  const normalized = sanitizeRetentionDays(days);
  if (normalized === 7) {
    logRetentionMode.value = '7';
    return;
  }

  if (normalized === 30) {
    logRetentionMode.value = '30';
    return;
  }

  logRetentionMode.value = 'custom';
  customRetentionDays.value = normalized;
};

const handleRetentionModeChange = (value: '7' | '30' | 'custom') => {
  logRetentionMode.value = value;

  if (value === '7') {
    settingsStore.updateLogRetentionDays(7);
    return;
  }

  if (value === '30') {
    settingsStore.updateLogRetentionDays(30);
    return;
  }

  customRetentionDays.value = sanitizeRetentionDays(settingsStore.settings.logRetentionDays);
  settingsStore.updateLogRetentionDays(customRetentionDays.value);
};

const handleCustomRetentionChange = (value: number | null) => {
  if (value === null) {
    return;
  }

  customRetentionDays.value = sanitizeRetentionDays(value);

  if (logRetentionMode.value === 'custom') {
    settingsStore.updateLogRetentionDays(customRetentionDays.value);
  }
};

onMounted(async () => {
  if (!settingsStore.isLoaded) {
    await settingsStore.loadSettings();
  }
  syncRetentionState(settingsStore.settings.logRetentionDays);
  appVersion.value = await getVersion();
});

const openGitHub = async (event: Event) => {
  event.preventDefault();
  try {
    await openUrl('https://github.com/lantianz');
  } catch (error) {
    console.error('打开链接失败:', error);
  }
};
</script>

<style scoped>
.settings-page {
  height: 100%;
  overflow: hidden;
}

.settings-scroll {
  height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 4px;
  scrollbar-gutter: stable;
}

.settings-card {
  background: var(--surface-bg);
  border: 1px solid var(--surface-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--surface-shadow);
  backdrop-filter: blur(18px);
  padding: 18px;
}

.section-header {
  margin-bottom: 14px;
}

.section-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

.section-divider {
  height: 1px;
  margin: 16px 0;
  background: var(--surface-border);
}

.preference-row {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr);
  gap: 16px;
  align-items: center;
}

.preference-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.theme-selector {
  justify-self: start;
}

.preference-row-start {
  align-items: start;
}

.preference-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.custom-retention-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.retention-input {
  width: 160px;
}

.retention-hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.about-list {
  display: flex;
  flex-direction: column;
}

.about-row {
  display: grid;
  grid-template-columns: 100px minmax(0, 1fr);
  gap: 16px;
  align-items: center;
  padding: 12px 0;
  border-top: 1px solid var(--surface-border);
}

.about-row:first-child {
  padding-top: 0;
  border-top: none;
}

.about-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.about-value {
  min-width: 0;
  color: var(--text-primary);
}

.link-button {
  padding: 0;
  font-size: 14px;
}

@media (max-width: 860px) {
  .preference-row,
  .about-row {
    grid-template-columns: 1fr;
    gap: 10px;
  }
}
</style>
