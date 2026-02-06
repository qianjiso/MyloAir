import type { IntegrityReport, RepairResult } from '../../shared/types';

export async function check(): Promise<IntegrityReport> {
  const res = await window.electronAPI.checkDataIntegrity();
  if (!res.success || !res.data) throw new Error(res.error || 'check failed');
  return res.data as IntegrityReport;
}

export async function repair(): Promise<RepairResult> {
  const res = await window.electronAPI.repairDataIntegrity();
  if (!res.success || !res.data) throw new Error(res.error || 'repair failed');
  return res.data as RepairResult;
}
