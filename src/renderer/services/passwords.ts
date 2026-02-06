/** 密码数据服务（渲染层轻适配器） */
export type PasswordItem = {
  id?: number;
  title: string;
  username: string;
  password?: string;
  url?: string;
  notes?: string;
  group_id?: number | null;
  created_at?: string;
  updated_at?: string;
};

export type PasswordHistory = {
  id?: number;
  password_id: number;
  old_password: string;
  new_password: string;
  changed_at?: string;
  changed_reason?: string;
};

/** 获取密码列表（可选分组过滤） */
export async function listPasswords(groupId?: number): Promise<PasswordItem[]> {
  const res = await window.electronAPI.getPasswords(groupId);
  return (res || []) as any;
}

/** 新建密码 */
export async function createPassword(payload: PasswordItem): Promise<{ success: boolean; id?: number; error?: string }>{
  return window.electronAPI.addPassword(payload);
}

/** 更新密码 */
export async function updatePassword(id: number, payload: PasswordItem): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.updatePassword(id, payload);
}

/** 删除密码 */
export async function removePassword(id: number): Promise<{ success: boolean; error?: string }>{
  return window.electronAPI.deletePassword(id);
}

/** 获取密码历史 */
export async function listPasswordHistory(passwordId: number): Promise<PasswordHistory[]>{
  return window.electronAPI.getPasswordHistory(passwordId);
}
