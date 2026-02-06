import type { SecureRecordGroup, SecureRecord } from '../../shared/types';

/** 获取便笺分组列表 */
export async function listNoteGroups(): Promise<SecureRecordGroup[]>{
  return window.electronAPI.getNoteGroups();
}

/** 获取便笺分组树 */
export async function getNoteGroupTree(parentId?: number): Promise<SecureRecordGroup[]>{
  return window.electronAPI.getNoteGroupTree(parentId);
}

/** 新建便笺分组 */
export async function createNoteGroup(group: SecureRecordGroup): Promise<{ success: boolean; id?: number; error?: string }>{
  return window.electronAPI.addNoteGroup(group);
}

/** 更新便笺分组 */
export async function updateNoteGroup(id: number, group: SecureRecordGroup): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.updateNoteGroup(id, group);
}

/** 删除便笺分组 */
export async function removeNoteGroup(id: number): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.deleteNoteGroup(id);
}

/** 获取便笺列表 */
export async function listNotes(groupId?: number): Promise<SecureRecord[]>{
  return window.electronAPI.getNotes(groupId);
}

export async function getNote(id: number): Promise<SecureRecord | null>{
  return window.electronAPI.getNote(id);
}

export async function createNote(note: SecureRecord): Promise<{ success: boolean; id?: number; error?: string }>{
  return window.electronAPI.addNote(note);
}

export async function updateNote(id: number, note: SecureRecord): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.updateNote(id, note);
}

export async function removeNote(id: number): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.deleteNote(id);
}
