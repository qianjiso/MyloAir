import { useState, useCallback } from 'react';
import { message } from 'antd';
import * as pwdService from '../services/passwords';
import { reportError } from '../utils/logging';

export interface PasswordItem {
  id: number;
  title: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
  group_id?: number | null;
  created_at?: string;
  updated_at?: string;
}

export function usePasswords() {
  const [passwords, setPasswords] = useState<PasswordItem[]>([]);
  const [passwordHistory, setPasswordHistory] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const loadPasswords = useCallback(async (groupId?: number) => {
    setLoading(true);
    try {
      const result = await pwdService.listPasswords(groupId);
      setPasswords((result || []) as any);
    } catch (error) {
      message.error('加载密码失败');
      reportError('PASSWORDS_LOAD_FAILED', 'Load passwords error', error, { groupId });
    } finally {
      setLoading(false);
    }
  }, []);

  const loadRecentPasswords = useCallback(async () => {
    setLoading(true);
    try {
      const result = await pwdService.listPasswords();
      setPasswords((result || []) as any);
    } catch (error) {
      reportError('PASSWORDS_LOAD_RECENT_FAILED', 'Load recent passwords error', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadPasswordHistory = useCallback(async (passwordId: number) => {
    try {
      const result = await pwdService.listPasswordHistory(passwordId);
      setPasswordHistory(result as any);
    } catch (error) {
      message.error('加载密码历史失败');
      reportError('PASSWORD_HISTORY_LOAD_FAILED', 'Load password history error', error, { passwordId });
    }
  }, []);

  const createPassword = useCallback(async (payload: PasswordItem) => {
    return pwdService.createPassword(payload);
  }, []);

  const updatePassword = useCallback(async (id: number, payload: PasswordItem) => {
    return pwdService.updatePassword(id, payload);
  }, []);

  const removePassword = useCallback(async (id: number) => {
    return pwdService.removePassword(id);
  }, []);

  return {
    passwords,
    passwordHistory,
    loading,
    loadPasswords,
    loadRecentPasswords,
    loadPasswordHistory,
    createPassword,
    updatePassword,
    removePassword,
    setPasswords,
  };
}
