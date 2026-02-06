/**
 * Tauri API 适配层
 *
 * 将 Electron 的 window.electronAPI 接口适配为 Tauri invoke 调用，
 * 保持前端代码的兼容性。
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import mockAPI from '../electronAPI-mock';

// 类型定义（从 shared/types 导入）
import type {
  Group,
  GroupWithChildren,
  PasswordHistory,
  UserSetting,
  UserSettingsCategory,
  SecureRecord,
  SecureRecordGroup,
  MasterPasswordState,
} from '../../shared/types';

/**
 * 密码条目（与 Rust 模型对应）
 */
interface Password {
  id?: number;
  title: string;
  username?: string;
  password?: string;
  url?: string;
  notes?: string;
  groupId?: number;
  createdAt?: string;
  updatedAt?: string;
  lastUsedAt?: string;
  useCount?: number;
  favorite?: boolean;
  tags?: string;
}

/**
 * Tauri API 接口定义
 * 保持与原 ElectronAPI 接口兼容
 */
export interface TauriAPI {
  // 密码管理
  getPasswords(groupId?: number): Promise<Password[]>;
  getPassword(id: number): Promise<Password | null>;
  addPassword(password: Password): Promise<{ success: boolean; id: number }>;
  updatePassword(
    id: number,
    password: Password
  ): Promise<{ success: boolean; error?: string }>;
  deletePassword(id: number): Promise<{ success: boolean }>;
  searchPasswords(keyword: string): Promise<Password[]>;
  advancedSearch(options: any): Promise<Password[]>;

  // 密码历史
  updatePasswordWithHistory(
    id: number,
    newPassword: string,
    reason?: string
  ): Promise<{ success: boolean; error?: string }>;
  getPasswordHistory(passwordId: number): Promise<PasswordHistory[]>;
  getPasswordsNeedingUpdate(): Promise<Password[]>;
  addPasswordHistory(
    history: PasswordHistory
  ): Promise<{ success: boolean; id: number; error?: string }>;
  getHistoryById(id: number): Promise<PasswordHistory | undefined>;
  deleteHistory(id: number): Promise<{ success: boolean; error?: string }>;
  cleanOldHistory(
    daysToKeep?: number
  ): Promise<{ success: boolean; count: number; error?: string }>;

  // 分组管理
  getGroups(): Promise<Group[]>;
  getGroupTree(parentId?: number): Promise<GroupWithChildren[]>;
  getGroupById(id: number): Promise<Group | undefined>;
  getGroupByName(name: string, parentId?: number): Promise<Group | undefined>;
  addGroup(group: Group): Promise<{ success: boolean; id: number }>;
  updateGroup(id: number, group: Group): Promise<{ success: boolean }>;
  deleteGroup(id: number): Promise<{ success: boolean }>;

  // 用户设置
  getUserSettings(category?: string): Promise<UserSetting[]>;
  getUserSetting(key: string): Promise<UserSetting | null>;
  setUserSetting(
    key: string,
    value: string,
    type?: string,
    category?: string,
    description?: string
  ): Promise<{ success: boolean; error?: string }>;
  updateUserSetting(
    key: string,
    value: string
  ): Promise<{ success: boolean; error?: string }>;
  deleteUserSetting(key: string): Promise<{ success: boolean; error?: string }>;
  getUserSettingsCategories(): Promise<UserSettingsCategory[]>;
  resetSettingToDefault(
    key: string
  ): Promise<{ success: boolean; error?: string }>;
  resetAllSettingsToDefault(): Promise<{
    success: boolean;
    count: number;
    error?: string;
  }>;
  importSettings(
    settings: UserSetting[]
  ): Promise<{ success: boolean; count: number; error?: string }>;
  exportSettings(
    categories?: string[]
  ): Promise<{ success: boolean; data?: string; error?: string }>;

  // 密码生成
  generatePassword(options: {
    length?: number;
    includeUppercase?: boolean;
    includeLowercase?: boolean;
    includeNumbers?: boolean;
    includeSymbols?: boolean;
  }): Promise<string>;

  // 系统相关
  getVersion(): Promise<string>;
  quit(): void;
  minimizeWindow(): Promise<void>;
  toggleMaximizeWindow(): Promise<void>;
  closeWindow(): Promise<void>;
  openExternal(url: string): Promise<void>;
  reportError?(payload: {
    code?: string;
    message: string;
    context?: Record<string, unknown>;
    stack?: string;
    source?: string;
  }): Promise<void>;

  // 数据完整性
  checkDataIntegrity(): Promise<{
    success: boolean;
    data?: { isValid: boolean; errors: string[]; warnings: string[] };
    error?: string;
  }>;
  repairDataIntegrity(): Promise<{
    success: boolean;
    data?: { repaired: string[]; failed: string[] };
    error?: string;
  }>;

  // 文件操作
  exportData: (options: {
    format: 'json' | 'encrypted_zip';
    includeHistory?: boolean;
    includeGroups?: boolean;
    includeSettings?: boolean;
    archivePassword?: string;
  }) => Promise<{ success: boolean; data?: number[]; error?: string }>;
  exportDataToFile: (options: {
    format: 'json' | 'encrypted_zip';
    includeHistory?: boolean;
    includeGroups?: boolean;
    includeSettings?: boolean;
    archivePassword?: string;
    filePath: string;
  }) => Promise<{ success: boolean; filePath?: string | null; error?: string }>;
  pickExportPath: (options: {
    defaultPath?: string;
    format: 'json' | 'encrypted_zip';
  }) => Promise<{ success: boolean; filePath?: string | null; error?: string }>;
  pickExportDirectory?: (options: { defaultPath?: string }) => Promise<{
    success: boolean;
    directory?: string | null;
    error?: string;
  }>;

  importData: (
    data: number[],
    options: {
      format: 'json';
      mergeStrategy: 'replace' | 'merge' | 'skip';
      validateIntegrity: boolean;
      dryRun: boolean;
    }
  ) => Promise<{ success: boolean; data?: any; error?: string }>;

  onDataImported: (
    handler: (payload: { imported: number; skipped: number }) => void
  ) => void;
  onAutoExportDone?: (
    handler: (payload: {
      success: boolean;
      filePath?: string;
      error?: string;
    }) => void
  ) => void;

  // 笔记相关
  getNoteGroups(): Promise<SecureRecordGroup[]>;
  getNoteGroupTree(parentId?: number): Promise<SecureRecordGroup[]>;
  addNoteGroup(
    group: SecureRecordGroup
  ): Promise<{ success: boolean; id: number; error?: string }>;
  updateNoteGroup(
    id: number,
    group: SecureRecordGroup
  ): Promise<{ success: boolean; error?: string }>;
  deleteNoteGroup(id: number): Promise<{ success: boolean; error?: string }>;
  getNotes(groupId?: number): Promise<SecureRecord[]>;
  getNote(id: number): Promise<SecureRecord | null>;
  addNote(
    note: SecureRecord
  ): Promise<{ success: boolean; id: number; error?: string }>;
  updateNote(
    id: number,
    note: SecureRecord
  ): Promise<{ success: boolean; error?: string }>;
  deleteNote(id: number): Promise<{ success: boolean; error?: string }>;
  searchNotesTitle(keyword: string): Promise<any[]>;

  // 安全相关
  getSecurityState(): Promise<MasterPasswordState>;
  setMasterPassword(
    password: string,
    hint?: string
  ): Promise<{ success: boolean; state?: MasterPasswordState; error?: string }>;
  verifyMasterPassword(
    password: string
  ): Promise<{ success: boolean; state?: MasterPasswordState; error?: string }>;
  updateMasterPassword(
    currentPassword: string,
    newPassword: string,
    hint?: string
  ): Promise<{ success: boolean; state?: MasterPasswordState; error?: string }>;
  clearMasterPassword(
    currentPassword: string
  ): Promise<{ success: boolean; state?: MasterPasswordState; error?: string }>;
  setRequireMasterPassword(
    require: boolean
  ): Promise<{ success: boolean; state?: MasterPasswordState; error?: string }>;
}

/**
 * Tauri API 实现
 */
const realTauriAPI: TauriAPI = {
  // 密码管理
  getPasswords: (groupId) => invoke('get_passwords', { groupId }),
  getPassword: (id) => invoke('get_password', { id }),
  addPassword: (password) => invoke('add_password', { password }),
  updatePassword: (id, password) => invoke('update_password', { id, password }),
  deletePassword: (id) => invoke('delete_password', { id }),
  searchPasswords: (keyword) => invoke('search_passwords', { keyword }),
  advancedSearch: (options) => invoke('advanced_search', { options }),

  // 密码历史（TODO: 实现）
  updatePasswordWithHistory: (id, newPassword, reason) =>
    invoke('update_password_with_history', { id, newPassword, reason }),
  getPasswordHistory: (passwordId) =>
    invoke('get_password_history', { passwordId }),
  getPasswordsNeedingUpdate: () => invoke('get_passwords_needing_update', {}),
  addPasswordHistory: (history) => invoke('add_password_history', { history }),
  getHistoryById: (id) => invoke('get_history_by_id', { id }),
  deleteHistory: (id) => invoke('delete_history', { id }),
  cleanOldHistory: (daysToKeep) => invoke('clean_old_history', { daysToKeep }),

  // 分组管理
  getGroups: () => invoke('get_groups', {}),
  getGroupTree: (parentId) => invoke('get_group_tree', { parentId }),
  getGroupById: (id) => invoke('get_group_by_id', { id }),
  getGroupByName: (name, parentId) =>
    invoke('get_group_by_name', { name, parentId }),
  addGroup: (group) => invoke('add_group', { group }),
  updateGroup: (id, group) => invoke('update_group', { id, group }),
  deleteGroup: (id) => invoke('delete_group', { id }),

  // 用户设置（TODO: 实现）
  getUserSettings: (category) => invoke('get_user_settings', { category }),
  getUserSetting: (key) => invoke('get_user_setting', { key }),
  setUserSetting: (key, value, type, category, description) =>
    invoke('set_user_setting', { key, value, type, category, description }),
  updateUserSetting: (key, value) =>
    invoke('update_user_setting', { key, value }),
  deleteUserSetting: (key) => invoke('delete_user_setting', { key }),
  getUserSettingsCategories: () => invoke('get_user_settings_categories', {}),
  resetSettingToDefault: (key) => invoke('reset_setting_to_default', { key }),
  resetAllSettingsToDefault: () => invoke('reset_all_settings_to_default', {}),
  importSettings: (settings) => invoke('import_settings', { settings }),
  exportSettings: (categories) => invoke('export_settings', { categories }),

  // 密码生成
  generatePassword: (options) => invoke('generate_password', { options }),

  // 系统相关
  getVersion: () => invoke('get_version', {}),
  quit: () => {
    invoke('quit', {});
  },
  minimizeWindow: () => invoke('minimize_window', {}),
  toggleMaximizeWindow: () => invoke('toggle_maximize_window', {}),
  closeWindow: () => invoke('close_window', {}),
  openExternal: (url) => invoke('open_external', { url }),
  reportError: (payload) => invoke('report_error', { payload }),

  // 数据完整性
  checkDataIntegrity: () => invoke('check_data_integrity', {}),
  repairDataIntegrity: () => invoke('repair_data_integrity', {}),

  // 文件操作
  exportData: (options) => invoke('export_data', { options }),
  exportDataToFile: (options) => invoke('export_data_to_file', { options }),
  pickExportPath: (options) => invoke('pick_export_path', { options }),
  pickExportDirectory: (options) =>
    invoke('pick_export_directory', { options }),
  importData: (data, options) => invoke('import_data', { data, options }),

  onDataImported: (handler) => {
    listen('data-imported', (event) =>
      handler(event.payload as { imported: number; skipped: number })
    );
  },
  onAutoExportDone: (handler) => {
    listen('auto-export-done', (event) =>
      handler(
        event.payload as { success: boolean; filePath?: string; error?: string }
      )
    );
  },

  // 笔记相关
  getNoteGroups: () => invoke('get_note_groups', {}),
  getNoteGroupTree: (parentId) => invoke('get_note_group_tree', { parentId }),
  addNoteGroup: (group) => invoke('add_note_group', { group }),
  updateNoteGroup: (id, group) => invoke('update_note_group', { id, group }),
  deleteNoteGroup: (id) => invoke('delete_note_group', { id }),
  getNotes: (groupId) => invoke('get_notes', { groupId }),
  getNote: (id) => invoke('get_note', { id }),
  addNote: (note) => invoke('add_note', { note }),
  updateNote: (id, note) => invoke('update_note', { id, note }),
  deleteNote: (id) => invoke('delete_note', { id }),
  searchNotesTitle: (keyword) => invoke('search_notes_title', { keyword }),

  // 安全相关
  getSecurityState: () => invoke('security_get_state', {}),
  setMasterPassword: (password, hint) =>
    invoke('security_set_master_password', { password, hint }),
  verifyMasterPassword: (password) =>
    invoke('security_verify_master_password', { password }),
  updateMasterPassword: (currentPassword, newPassword, hint) =>
    invoke('security_update_master_password', {
      currentPassword,
      newPassword,
      hint,
    }),
  clearMasterPassword: (currentPassword) =>
    invoke('security_clear_master_password', { currentPassword }),
  setRequireMasterPassword: (require) =>
    invoke('security_set_require_master_password', { require }),
};

/**
 * 决定使用真实API还是Mock API
 */
// 简单判断: 检查 window.__TAURI_INTERNALS__ 或其他 Tauri 特有对象
// 这里假设非 Tauri 环境下 invoke 会失败，或者我们显式检查 window.__TAURI__
const isTauri =
  typeof window !== 'undefined' &&
  (window as any).__TAURI_INTERNALS__ !== undefined;

export const tauriAPI: TauriAPI = isTauri
  ? realTauriAPI
  : (mockAPI as unknown as TauriAPI);

/**
 * 兼容层：将 tauriAPI 暴露为 window.electronAPI
 * 这样前端代码无需修改即可使用
 */
export function initElectronAPICompat(): void {
  (window as any).electronAPI = tauriAPI;

  if (!isTauri) {
    console.warn('[TauriAPI] Running in browser mode. Using Mock API.');
    console.log('[TauriAPI] Mock API:', tauriAPI);
  }
}

// 自动初始化兼容层
initElectronAPICompat();

export default tauriAPI;
