import { useState, useCallback } from 'react';
import * as backupService from '../services/backup';
import type { ExportOptions, ImportOptions, ImportResult } from '../../shared/types';

export function useBackup() {
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [lastExport, setLastExport] = useState<Uint8Array | null>(null);
  const [lastImportResult, setLastImportResult] = useState<ImportResult | null>(null);

  const exportData = useCallback(async (options: ExportOptions) => {
    setExporting(true);
    try {
      const data = await backupService.exportData(options);
      setLastExport(data);
      return data;
    } finally {
      setExporting(false);
    }
  }, []);

  const importData = useCallback(async (data: Uint8Array, options: ImportOptions) => {
    setImporting(true);
    try {
      const result = await backupService.importData(data, options);
      setLastImportResult(result);
      return result;
    } finally {
      setImporting(false);
    }
  }, []);

  const exportDataToFile = useCallback(async (options: ExportOptions & { filePath: string }) => {
    setExporting(true);
    try {
      return await backupService.exportDataToFile(options);
    } finally {
      setExporting(false);
    }
  }, []);

  return { exporting, importing, lastExport, lastImportResult, exportData, exportDataToFile, importData };
}
