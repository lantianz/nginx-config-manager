<script setup lang="ts">
import AppLayout from "./components/layout/AppLayout.vue";
</script>

<template>
  <AppLayout />
</template>

<style>
:root {
  color: #1e3a5f;
  font-family: "PingFang SC", "Microsoft YaHei", "Segoe UI", sans-serif;
  line-height: 1.5;
  font-weight: 400;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  --app-bg: #eef4fb;
  --app-bg-accent: rgba(147, 197, 253, 0.24);
  --surface-bg: rgba(255, 255, 255, 0.78);
  --surface-bg-strong: rgba(255, 255, 255, 0.92);
  --surface-bg-soft: rgba(248, 250, 252, 0.8);
  --surface-border: rgba(148, 163, 184, 0.18);
  /* --surface-shadow: 0 14px 28px rgba(15, 23, 42, 0.08); */
  --surface-shadow: none;
  --text-primary: #1e3a5f;
  --text-secondary: #64748b;
  --text-muted: #94a3b8;
  --brand-color: #3b82f6;
  --brand-soft: rgba(59, 130, 246, 0.12);
  --success-soft: rgba(16, 185, 129, 0.12);
  --warning-soft: rgba(245, 158, 11, 0.12);
  --danger-soft: rgba(244, 63, 94, 0.12);
  --radius-xl: 20px;
  --radius-lg: 16px;
  --radius-md: 12px;
  --radius-sm: 10px;
  --page-gap: 16px;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
}

body {
  overflow: hidden;
  background: var(--app-bg);
  color: var(--text-primary);
  user-select: none;
  -webkit-user-select: none;
}

body.dark {
  --app-bg: #0f172a;
  --app-bg-accent: rgba(59, 130, 246, 0.12);
  --surface-bg: rgba(15, 23, 42, 0.78);
  --surface-bg-strong: rgba(15, 23, 42, 0.92);
  --surface-bg-soft: rgba(30, 41, 59, 0.72);
  --surface-border: rgba(148, 163, 184, 0.16);
  /* --surface-shadow: 0 18px 36px rgba(2, 6, 23, 0.34); */
  --surface-shadow: none;
  --text-primary: #e2e8f0;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
  background: var(--app-bg);
  color: var(--text-primary);
}

#app {
  user-select: none;
  -webkit-user-select: none;
}

input,
textarea,
select,
[contenteditable="true"],
.allow-select,
.allow-select *,
.n-input input,
.n-input textarea,
.n-base-selection-input,
.monaco-editor,
.monaco-editor * {
  user-select: text;
  -webkit-user-select: text;
}

::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.5);
  border-radius: 999px;
  border: 2px solid transparent;
  background-clip: padding-box;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(100, 116, 139, 0.68);
}

body.dark ::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.34);
}

body.dark ::-webkit-scrollbar-thumb:hover {
  background: rgba(148, 163, 184, 0.5);
}

*::-webkit-scrollbar-button {
  display: none !important;
}
</style>
