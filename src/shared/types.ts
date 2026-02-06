export interface PasswordItem {
  id?: number;
  title: string;
  username: string;
  password?: string | null;
  url?: string | null;
  notes?: string | null;
  group_id?: number | null;
  created_at?: string;
  updated_at?: string;
}

export interface PasswordHistory {
  id?: number;
  password_id: number;
  old_password: string;
  new_password: string;
  changed_at?: string;
  changed_reason?: string;
}

export interface Group {
  id?: number;
  name: string;
  parent_id?: number;
  color?: string;
  icon?: string;
  order_index?: number;
  sort?: number;
  created_at?: string;
  updated_at?: string;
}

export interface GroupWithChildren extends Group {
  children: GroupWithChildren[];
}

export interface SecureRecordGroup {
  id?: number;
  name: string;
  parent_id?: number | null;
  color?: string;
  sort_order?: number;
  created_at?: string;
  updated_at?: string;
}

export interface SecureRecord {
  id?: number;
  title?: string | null;
  content_ciphertext: string;
  group_id?: number | null;
  pinned?: boolean;
  archived?: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface UserSetting {
  id?: number;
  key: string;
  value: string;
  type?: 'string' | 'number' | 'boolean' | 'json';
  category?: string;
  description?: string;
  created_at?: string;
  updated_at?: string;
}

export interface UserSettingsCategory {
  category: string;
  description: string;
  settings: string[];
}

export type ExportOptions = {
  format: 'json' | 'encrypted_zip';
  includeHistory?: boolean;
  includeGroups?: boolean;
  includeSettings?: boolean;
  archivePassword?: string;
};

export type ImportOptions = {
  format: 'json';
  mergeStrategy: 'replace' | 'merge' | 'skip';
  validateIntegrity: boolean;
  dryRun: boolean;
};

export type ImportResult = {
  success: boolean;
  imported: number;
  skipped: number;
  errors: string[];
  warnings: string[];
};

export type IntegrityReport = {
  isValid: boolean;
  errors: string[];
  warnings: string[];
};

export type RepairResult = {
  repaired: string[];
  failed: string[];
};

export interface MasterPasswordState {
  hasMasterPassword: boolean;
  requireMasterPassword: boolean;
  hint?: string;
  autoLockMinutes: number;
  lastUnlockAt?: string;
}

export type AutoExportFrequency = 'every_minute' | 'daily' | 'weekly' | 'monthly';

export interface AutoExportConfig {
  enabled: boolean;
  frequency: AutoExportFrequency;
  directory: string;
  format: ExportOptions['format'];
  archivePassword?: string;
  /** 每日/每周/每月执行时间，格式 HH:mm */
  timeOfDay?: string;
  /** 每周执行的周几（1=周一 ... 7=周日） */
  dayOfWeek?: number;
  /** 每月执行的日期（1-31，超出当月天数时会自动调整为当月最后一天） */
  dayOfMonth?: number;
  /** “每分”模式下的执行间隔（单位：分钟），默认 60 */
  intervalMinutes?: number;
}
