import type { Group, GroupWithChildren } from '../../shared/types';

/** 获取分组列表 */
export async function listGroups(): Promise<Group[]> {
  return window.electronAPI.getGroups();
}

/** 获取分组树 */
export async function getGroupTree(parentId?: number): Promise<GroupWithChildren[]>{
  return window.electronAPI.getGroupTree(parentId);
}

/** 新建分组 */
export async function createGroup(group: Group): Promise<{ success: boolean; id?: number; error?: string }>{
  return window.electronAPI.addGroup(group);
}

/** 更新分组 */
export async function updateGroup(id: number, group: Group): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.updateGroup(id, group);
}

/** 删除分组 */
export async function removeGroup(id: number): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.deleteGroup(id);
}
