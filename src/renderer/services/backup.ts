import type { ExportOptions, ImportOptions, ImportResult } from '../../shared/types';

export async function exportData(options: ExportOptions): Promise<Uint8Array> {
  const res = await window.electronAPI.exportData(options);
  if (!res.success || !res.data) throw new Error(res.error || 'export failed');
  return new Uint8Array(res.data);
}

export async function exportDataToFile(options: ExportOptions & { filePath: string }): Promise<string | null> {
  const res = await window.electronAPI.exportDataToFile(options);
  if (!res.success) throw new Error(res.error || 'export failed');
  return res.filePath ?? null;
}

export async function pickExportPath(options: { defaultPath?: string; format: ExportOptions['format'] }): Promise<string | null> {
  const res = await window.electronAPI.pickExportPath(options);
  if (!res.success) throw new Error(res.error || 'pick path failed');
  return res.filePath ?? null;
}

export async function pickExportDirectory(options: { defaultPath?: string }): Promise<string | null> {
  if (!window.electronAPI.pickExportDirectory) return null;
  const res = await window.electronAPI.pickExportDirectory(options);
  if (!res.success) throw new Error(res.error || 'pick directory failed');
  return res.directory ?? null;
}

export async function importData(data: Uint8Array, options: ImportOptions): Promise<ImportResult> {
  const res = await window.electronAPI.importData(Array.from(data), options);
  if (!res.success || !res.data) throw new Error(res.error || 'import failed');
  return res.data as ImportResult;
}
