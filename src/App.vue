<script setup lang="ts">
import AppLayout from './components/layout/AppLayout.vue';
</script>

<template>
  <AppLayout />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'PingFang SC', 'Microsoft YaHei', '微软雅黑', 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

#app {
  width: 100%;
  height: 100vh;
  overflow: hidden;
}

/* 自定义滚动条样式 - 亮色模式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 4px;
}

::-webkit-scrollbar-corner {
  background: rgba(0, 0, 0, 0);
}

::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
  transition: background 0.3s;
}

::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}

/* 自定义滚动条样式 - 暗色模式 */
.dark ::-webkit-scrollbar-track {
  background: transparent;
}

.dark ::-webkit-scrollbar-thumb {
  background: #4a4a4a;
}

.dark ::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}

/* Firefox 滚动条样式 */
* {
  /* scrollbar-width: thin;
  scrollbar-color: #c1c1c1 #f1f1f1; */
}
.dark * {
  /* scrollbar-color: #4a4a4a #2c2c2c; */
}
*::-webkit-scrollbar-button {
  display: none !important;
}
*::-webkit-scrollbar-track {
  background: transparent; /* 轨道透明 */
}

/* 文字不可选中样式 - 提升用户体验 */
/* 按钮文字不可选中 */
button,
.n-button,
.n-button__content {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 标签文字不可选中 */
.n-tag {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 卡片标题不可选中 */
.n-card-header__main,
.n-card-header {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 侧边栏菜单不可选中 */
.n-menu-item,
.n-menu-item-content,
.n-menu-item-content__icon,
.n-menu-item-content-header {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 表单标签不可选中 */
.n-form-item-label,
.n-form-item-label__text {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 时间线标题不可选中 */
.n-timeline-item-header,
.n-timeline-item-content__title {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 描述列表标签不可选中 */
.n-descriptions-table-wrapper .n-descriptions-table-content__label {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 图标不可选中 */
.n-icon {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* Alert 标题不可选中 */
.n-alert__title {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* Empty 描述不可选中 */
.n-empty__description {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}
</style>