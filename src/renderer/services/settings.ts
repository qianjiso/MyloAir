import type { UserSetting } from '../../shared/types';

/** 获取设置列表（可选类别） */
export async function listSettings(category?: string): Promise<UserSetting[]>{
  return window.electronAPI.getUserSettings(category);
}

/** 设置或更新单个设置项 */
export async function setSetting(key: string, value: string, type?: string, category?: string, description?: string): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.setUserSetting(key, value, type, category, description);
}

/** 重置到默认 */
export async function resetSettingToDefault(key: string): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.resetSettingToDefault(key);
}

/** 重置全部到默认 */
export async function resetAllSettingsToDefault(): Promise<{ success: boolean; count?: number; error?: string }>{
  return window.electronAPI.resetAllSettingsToDefault();
}
