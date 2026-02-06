import type { MasterPasswordState } from '../../shared/types';

export async function getSecurityState(): Promise<MasterPasswordState> {
  return window.electronAPI.getSecurityState();
}

export async function setMasterPassword(password: string, hint?: string) {
  return window.electronAPI.setMasterPassword(password, hint);
}

export async function verifyMasterPassword(password: string) {
  return window.electronAPI.verifyMasterPassword(password);
}

export async function updateMasterPassword(currentPassword: string, newPassword: string, hint?: string) {
  return window.electronAPI.updateMasterPassword(currentPassword, newPassword, hint);
}

export async function clearMasterPassword(currentPassword: string) {
  return window.electronAPI.clearMasterPassword(currentPassword);
}

export async function setRequireMasterPassword(require: boolean) {
  return window.electronAPI.setRequireMasterPassword(require);
}
