// 轻量级发布-订阅事件总线（无外部依赖）
type Handler<T = unknown> = (payload: T) => void;

const _handlers = new Map<string, Set<Handler>>();

export const eventBus = {
  /** 订阅事件，返回取消订阅函数 */
  on<T>(event: string, handler: Handler<T>): () => void {
    if (!_handlers.has(event)) _handlers.set(event, new Set());
    _handlers.get(event)!.add(handler as Handler);
    return () => _handlers.get(event)?.delete(handler as Handler);
  },
  /** 发布事件 */
  emit<T>(event: string, payload?: T): void {
    _handlers.get(event)?.forEach(h => h(payload));
  },
};

// ---- 事件 payload 类型 ----

export interface NginxOperationResult {
  success: boolean;
  message: string;
  operation: 'start' | 'stop' | 'restart' | 'reload' | 'test';
}

// ---- 事件名常量 ----

export const EVENTS = {
  /** Nginx 操作（启动/停止/重启/重载/校验）完成 */
  NGINX_OPERATION_RESULT: 'nginx:operation-result',
  /** 应用设置加载完成 */
  SETTINGS_LOADED: 'settings:loaded',
  /** 配置文件加载完成 */
  CONFIG_LOADED: 'config:loaded',
} as const;
