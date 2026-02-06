import { useState, useCallback } from 'react';
import * as integrityService from '../services/integrity';
import type { IntegrityReport, RepairResult } from '../../shared/types';

export function useIntegrity() {
  const [checking, setChecking] = useState(false);
  const [repairing, setRepairing] = useState(false);
  const [report, setReport] = useState<IntegrityReport | null>(null);
  const [repairResult, setRepairResult] = useState<RepairResult | null>(null);

  const check = useCallback(async () => {
    setChecking(true);
    try {
      const r = await integrityService.check();
      setReport(r);
      return r;
    } finally {
      setChecking(false);
    }
  }, []);

  const repair = useCallback(async () => {
    setRepairing(true);
    try {
      const r = await integrityService.repair();
      setRepairResult(r);
      return r;
    } finally {
      setRepairing(false);
    }
  }, []);

  return { checking, repairing, report, repairResult, check, repair };
}
