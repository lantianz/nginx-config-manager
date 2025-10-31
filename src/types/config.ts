// Nginx 配置文件相关类型定义

/**
 * 配置指令
 */
export interface Directive {
  name: string;
  value: string;
  line: number;
}

/**
 * Location 配置块
 */
export interface LocationBlock {
  id: string;
  path: string;
  modifier?: string; // =, ~, ~*, ^~
  directives: Directive[];
  rawContent: string;
  startLine: number;
  endLine: number;
}

/**
 * Server 配置块
 */
export interface ServerBlock {
  id: string;
  listen: string[];
  serverName: string[];
  locations: LocationBlock[];
  directives: Directive[];
  rawContent: string;
  startLine: number;
  endLine: number;
}

/**
 * Nginx 配置结构
 */
export interface NginxConfig {
  servers: ServerBlock[];
  globalDirectives: Directive[];
  filePath: string;
  rawContent: string;
}

/**
 * 配置编辑表单数据 - Server
 */
export interface ServerFormData {
  listen: string[];
  serverName: string[];
  root?: string;
  index?: string;
  enableSSL: boolean;
  enableHTTP2: boolean;
  enableGzip: boolean;
  customDirectives: string;
}

/**
 * 配置编辑表单数据 - Location
 */
export interface LocationFormData {
  path: string;
  modifier?: string;
  root?: string;
  alias?: string;
  proxyPass?: string;
  index?: string;
  customDirectives: string;
}

