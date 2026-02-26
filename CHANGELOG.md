# 更新日志

## 2026年2月26日（补丁）

### 🐛 修复
- 修复点击 Nginx 操作按钮后切换菜单卡住的问题：Tauri 2 中同步命令（`fn`）在主线程执行会阻塞 WebView 消息泵，将 `nginx.rs`、`config.rs`、`settings.rs`、`file_ops.rs` 中所有 `#[tauri::command]` 函数改为 `async fn`，使其在 tokio 线程池执行，主线程不再被阻塞
- 将 `nginx.rs` 中的 `thread::sleep` 替换为 `tokio::time::sleep(...).await`，彻底消除启动/停止等待期间的主线程阻塞

---

### ♻️ 重构
- 引入发布-订阅事件总线（`src/composables/useEventBus.ts`），解耦 Store 操作与 UI 消息提示
- `nginx store`：`start / stop / restart / reload / testConfig` 改为 fire-and-forget 模式，操作完成后通过 `NGINX_OPERATION_RESULT` 事件通知订阅方，状态刷新（`checkStatus`）异步触发，不再串行阻塞
- `settings store`：加载完成后 emit `SETTINGS_LOADED` 事件，新增 `isLoaded` 标志位
- `config store`：加载完成后 emit `CONFIG_LOADED` 事件
- `AppLayout.vue`：修复启动时双重调用 `loadSettings` 的问题；`init()` 改为并行触发设置加载与状态检查；主题初始化改用 `watch` 响应式驱动，移除冗余 `onMounted`
- `ProcessManager.vue`：移除 `onMounted` 中的 `await`，改用 `watch` 响应设置变化；所有操作 handler 改为同步 fire-and-forget，消息提示统一由事件订阅处理
- `ConfigManager.vue`：移除 `onMounted` 中的 `await`，改用 `watch` 响应配置路径变化；`handleReloadConfig` 改为 fire-and-forget

---

## 2025年11月1日

### ✨ 新功能
- 在配置管理页面添加"重载配置"按钮，方便用户在新增/编辑配置后直接重载

### 🎨 界面优化
- 页面美化，添加苹方字体

### 🐛 修复
- 配置管理页面重载配置功能异常（导致 Nginx 进程状态异常、创建多余进程）
- 配置管理sever卡片， location 搜索不到导致消失问题
- 日志顺序问题

### 🔒 安全性
- 实现应用单实例限制，防止同时打开多个应用实例

---

## v0.1.0 (2025-10-31)

### ✨ 新功能
- 实现 Nginx 进程管理功能
- 实现配置文件解析和管理
- 集成 Monaco Editor 文本编辑器
- 实现日志查看功能
- 实现应用设置页面
- 添加主题切换功能
- 添加"打开文件"功能

### 🔧 改进
- 优化配置文件解析逻辑
- 优化 UI 交互体验
- 优化日志显示格式
- 优化深色主题样式

### 🐛 修复
- 修复 Windows GBK 编码问题
- 修复配置文件路径解析问题
- 修复弹窗高度溢出问题
- 修复 Location 标签显示问题

