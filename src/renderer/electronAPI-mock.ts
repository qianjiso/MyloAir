// Mock electronAPI for browser development

// In-memory store
const store = {
  passwords: [] as any[],
  groups: [] as any[],
  settings: [] as any[],
  history: [] as any[],
  notes: [] as any[],
  noteGroups: [] as any[],
  security: {
    hasMasterPassword: false,
    requireMasterPassword: false,
    hint: '',
    autoLockMinutes: 5,
    lastUnlockAt: ''
  }
};

let idCounter = 1;
const nextId = () => idCounter++;

const electronAPI = {
  // 密码管理相关
  getPasswords: (groupId?: number) => {
    return Promise.resolve(
      groupId 
        ? store.passwords.filter(p => p.group_id === groupId)
        : store.passwords
    );
  },
  getPassword: (id: number) => {
    const pw = store.passwords.find(p => p.id === id);
    return Promise.resolve(pw || null);
  },
  addPassword: (password: any) => {
    const newPw = { ...password, id: nextId(), created_at: new Date().toISOString() };
    store.passwords.push(newPw);
    console.log('[Mock] Added password:', newPw);
    return Promise.resolve({ success: true, id: newPw.id });
  },
  updatePassword: (id: number, password: any) => {
    const idx = store.passwords.findIndex(p => p.id === id);
    if (idx !== -1) {
      store.passwords[idx] = { ...store.passwords[idx], ...password, updated_at: new Date().toISOString() };
      return Promise.resolve({ success: true });
    }
    return Promise.resolve({ success: false, error: 'Not found' });
  },
  deletePassword: (id: number) => {
    const idx = store.passwords.findIndex(p => p.id === id);
    if (idx !== -1) {
      store.passwords.splice(idx, 1);
      return Promise.resolve({ success: true });
    }
    return Promise.resolve({ success: false });
  },
  searchPasswords: (keyword: string) => {
    if (!keyword) return Promise.resolve(store.passwords);
    const lower = keyword.toLowerCase();
    return Promise.resolve(
      store.passwords.filter(p => 
        (p.title && p.title.toLowerCase().includes(lower)) ||
        (p.username && p.username.toLowerCase().includes(lower))
      )
    );
  },
  
  // 密码历史记录
  getPasswordHistory: (_passwordId: number) => Promise.resolve([]),
  getPasswordsNeedingUpdate: () => Promise.resolve([]),
  
  // 分组管理
  getGroups: () => {
    console.log('[Mock] Getting groups:', store.groups);
    return Promise.resolve([...store.groups]); 
  },
  getGroupTree: (_parentId?: number) => {
     // Configurable flat list for now, or build tree if needed. 
     // Frontend usually handles tree building from flat list or expects pre-built
     // Let's return flat list as standard getGroups often used.
     // But strictly getGroupTree might expect recursive structure. 
     // For mock, returning empty or flat might be enough if frontend handles it.
     return Promise.resolve([...store.groups]); 
  },
  addGroup: (group: any) => {
    const newGroup = { ...group, id: nextId(), created_at: new Date().toISOString(), children: [] };
    store.groups.push(newGroup);
    console.log('[Mock] Added group:', newGroup, 'Total groups:', store.groups.length);
    return Promise.resolve({ success: true, id: newGroup.id });
  },
  updateGroup: (id: number, group: any) => {
    const idx = store.groups.findIndex(g => g.id === id);
    if (idx !== -1) {
      store.groups[idx] = { ...store.groups[idx], ...group };
      return Promise.resolve({ success: true });
    }
    return Promise.resolve({ success: false });
  },
  deleteGroup: (id: number) => {
    const idx = store.groups.findIndex(g => g.id === id);
    if (idx !== -1) {
      store.groups.splice(idx, 1);
      return Promise.resolve({ success: true });
    }
    return Promise.resolve({ success: false });
  },
  
  // 用户设置
  getUserSettings: (_category?: string) => Promise.resolve([]),
  getUserSetting: (_key: string) => Promise.resolve(null),
  setUserSetting: (_key: string, _value: string, _type?: string, _category?: string, _description?: string) => Promise.resolve({ success: true }),
  updateUserSetting: (_key: string, _value: string) => Promise.resolve({ success: true }),
  deleteUserSetting: (_key: string) => Promise.resolve({ success: true }),
  getUserSettingsCategories: () => Promise.resolve([]),
  
  // 密码生成
  generatePassword: (options: { 
    length?: number; 
    includeUppercase?: boolean;
    includeLowercase?: boolean;
    includeNumbers?: boolean;
    includeSymbols?: boolean;
  }) => {
    const length = options.length || 16;
    let charset = '';
    
    if (options.includeUppercase) charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    if (options.includeLowercase) charset += 'abcdefghijklmnopqrstuvwxyz';
    if (options.includeNumbers) charset += '0123456789';
    if (options.includeSymbols) charset += '!@#$%^&*()_+-=[]{}|;:,.<>?';
    
    if (!charset) charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    
    let password = '';
    for (let i = 0; i < length; i++) {
      password += charset.charAt(Math.floor(Math.random() * charset.length));
    }
    return Promise.resolve(password);
  },
  
  // 系统相关
  getVersion: () => Promise.resolve('1.0.0'),
  quit: () => {},
  
  // 文件操作
  exportData: () => Promise.resolve({ success: true, data: JSON.stringify(store) }),
  importData: (_data: number[], _options?: any) => Promise.resolve({ success: true }),
  
  // 安全相关
  getSecurityState: () => Promise.resolve({ ...store.security }),
  setMasterPassword: (password: string, _hint?: string) => {
    store.security.hasMasterPassword = true;
    return Promise.resolve({ success: true, state: store.security });
  },
  verifyMasterPassword: (_password: string) => Promise.resolve({ success: true, state: store.security }),
  updateMasterPassword: (_currentPassword: string, _newPassword: string, _hint?: string) => Promise.resolve({ success: true }),
  clearMasterPassword: (_currentPassword: string) => {
    store.security.hasMasterPassword = false;
    return Promise.resolve({ success: true });
  },
  setRequireMasterPassword: (require: boolean) => {
    store.security.requireMasterPassword = require;
    return Promise.resolve({ success: true });
  },

  // Note management
  getNoteGroups: () => Promise.resolve([...store.noteGroups]),
  getNoteGroupTree: () => Promise.resolve([...store.noteGroups]), 
  addNoteGroup: (group: any) => {
    const newG = { ...group, id: nextId() };
    store.noteGroups.push(newG);
    return Promise.resolve({ success: true, id: newG.id });
  },
  updateNoteGroup: (id: number, group: any) => { return Promise.resolve({ success: true }); },
  deleteNoteGroup: (id: number) => { return Promise.resolve({ success: true }); },
  getNotes: (groupId?: number) => {
     return Promise.resolve(
      groupId 
        ? store.notes.filter(n => n.group_id === groupId)
        : store.notes
    );
  },
  getNote: (id: number) => Promise.resolve(store.notes.find(n => n.id === id) || null),
  addNote: (note: any) => {
    const newN = { ...note, id: nextId(), created_at: new Date().toISOString() };
    store.notes.push(newN);
    return Promise.resolve({ success: true, id: newN.id });
  },
  updateNote: (id: number, note: any) => { return Promise.resolve({ success: true }); },
  deleteNote: (id: number) => { return Promise.resolve({ success: true }); },
  searchNotesTitle: (keyword: string) => Promise.resolve([]),
};

// 在浏览器环境中，将electronAPI挂载到window对象上
if (typeof window !== 'undefined') {
  (window as any).electronAPI = electronAPI;
  // Initialize default group for testing convenience
  store.groups.push({ id: 999, name: 'Default Group', parent_id: null, color: 'blue' });
}

export default electronAPI;
