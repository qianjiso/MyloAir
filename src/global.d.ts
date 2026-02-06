/**
 * 全局类型声明
 * 扩展 Window 接口以支持 electronAPI
 */

import type { TauriAPI } from './renderer/api/tauriAPI';

declare global {
  interface Window {
    electronAPI: TauriAPI;
  }
}

export {};
