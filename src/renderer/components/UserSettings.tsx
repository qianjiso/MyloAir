import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Card,
  Form,
  Switch,
  Select,
  Button,
  Space,
  message,
  Typography,
  Row,
  Col,
  InputNumber,
  Tag,
  Modal,
  Alert,
  Input,
  Tabs,
  Collapse,
} from 'antd';
import {
  SaveOutlined,
  ReloadOutlined,
  SafetyCertificateOutlined,
  ToolOutlined,
  CheckCircleOutlined,
  ToolTwoTone,
  FolderOpenOutlined,
  CloudUploadOutlined,
  ApiOutlined,
} from '@ant-design/icons';
import type {
  BackupCloudTestInput,
  BackupConfig,
  BackupRunStatus,
  SaveBackupConfigInput,
  MasterPasswordState,
  UserSetting,
} from '../../shared/types';
import * as settingsService from '../services/settings';
import { useIntegrity } from '../hooks/useIntegrity';
import { reportError } from '../utils/logging';
import * as securityService from '../services/security';
import * as backupService from '../services/backup';

const { Title } = Typography;
const { Option } = Select;

interface UserSettingsProps {
  onClose?: () => void;
}

const UserSettings: React.FC<UserSettingsProps> = ({ onClose }) => {
  const [form] = Form.useForm();
  const autoExportEnabled = Form.useWatch('autoExportEnabled', form);
  const autoExportFrequency = Form.useWatch('autoExportFrequency', form);
  const autoExportDirectory = Form.useWatch('autoExportDirectory', form);
  const autoExportTimeOfDay = Form.useWatch('autoExportTimeOfDay', form);
  const autoExportDayOfWeek = Form.useWatch('autoExportDayOfWeek', form);
  const autoExportDayOfMonth = Form.useWatch('autoExportDayOfMonth', form);
  const autoExportIntervalMinutes = Form.useWatch(
    'autoExportIntervalMinutes',
    form
  );
  const targetMode = Form.useWatch('targetMode', form);
  const requireMasterPassword = Form.useWatch('requireMasterPassword', form);
  const autoLockMinutes = Form.useWatch('autoLockMinutes', form);
  const [initialAutoExportEnabled, setInitialAutoExportEnabled] = useState<
    boolean | null
  >(null);
  const [loading, setLoading] = useState(false);
  const { checking, repairing, report, repairResult, check, repair } =
    useIntegrity();
  const [securityState, setSecurityState] =
    useState<MasterPasswordState | null>(null);
  const [masterModalVisible, setMasterModalVisible] = useState(false);
  const [masterMode, setMasterMode] = useState<'set' | 'update' | 'remove' | 'enable' | 'disable'>(
    'set'
  );
  const [masterSaving, setMasterSaving] = useState(false);
  const [masterForm] = Form.useForm();
  const [selectingExportDirectory, setSelectingExportDirectory] =
    useState(false);
  const [cloudTesting, setCloudTesting] = useState(false);
  const [cloudUploading, setCloudUploading] = useState(false);
  const [savedBackupConfig, setSavedBackupConfig] = useState<BackupConfig | null>(
    null
  );
  const [activeTab, setActiveTab] = useState<'security' | 'ui' | 'data'>(
    'security'
  );
  const normalizeExportFormat = useCallback(
    (fmt?: string) => (fmt === 'encrypted_zip' ? 'encrypted_zip' : 'json'),
    []
  );
  const autoExportStatus =
    autoExportEnabled ?? initialAutoExportEnabled ?? false;
  const hasSavedArchivePassword = savedBackupConfig?.hasArchivePassword ?? false;
  const hasSavedSecretKey = savedBackupConfig?.hasSecretKey ?? false;
  const secretIdMasked = savedBackupConfig?.secretIdMasked ?? '';
  const savedManualRun = savedBackupConfig?.lastManualRun;
  const savedAutoRun = savedBackupConfig?.lastAutoRun;
  const failureCooldownMinutes =
    savedBackupConfig?.failureNotificationCooldownMinutes ?? 5;

  const autoExportSummary = useMemo(() => {
    if (!autoExportStatus) return '自动导出未开启';
    const targetText = targetMode === 'cos'
      ? `腾讯云 COS${form.getFieldValue('bucket') ? ` · ${form.getFieldValue('bucket')}` : ''}`
      : autoExportDirectory || '未选择目录';
    const weekMap: Record<number, string> = {
      1: '周一',
      2: '周二',
      3: '周三',
      4: '周四',
      5: '周五',
      6: '周六',
      7: '周日',
    };
    const timeText = autoExportTimeOfDay || '02:00';
    const dayOfWeekText = weekMap[Number(autoExportDayOfWeek)] || '周一';
    const dayOfMonthText = autoExportDayOfMonth || 1;
    if (autoExportFrequency === 'every_minute') {
      return `每 ${autoExportIntervalMinutes || 60} 分钟导出 · ${targetText}`;
    }
    if (autoExportFrequency === 'weekly') {
      return `每周${dayOfWeekText} ${timeText} · ${targetText}`;
    }
    if (autoExportFrequency === 'monthly') {
      return `每月${dayOfMonthText} 日 ${timeText} · ${targetText}`;
    }
    return `每日 ${timeText} · ${targetText}`;
  }, [
    form,
    autoExportDirectory,
    autoExportFrequency,
    autoExportIntervalMinutes,
    autoExportDayOfMonth,
    autoExportDayOfWeek,
    autoExportStatus,
    autoExportTimeOfDay,
    targetMode,
  ]);
  const securitySummary = useMemo(() => {
    const lock = Math.max(1, Number(autoLockMinutes || 5));
    if (securityState?.hasMasterPassword) {
      return requireMasterPassword
        ? `主密码已启用 · ${lock} 分钟自动锁定`
        : `主密码已设置 · ${lock} 分钟自动锁定`;
    }
    return `主密码未设置 · ${lock} 分钟自动锁定`;
  }, [autoLockMinutes, requireMasterPassword, securityState]);

  const formatRunSummary = useCallback((run?: BackupRunStatus | null) => {
    if (!run?.at) return '暂无记录';
    const resultText =
      run.result === 'success' ? '成功' : run.result === 'failed' ? '失败' : '未知';
    const targetText = run.target === 'cos' ? '云端' : run.target === 'local' ? '本地' : '未知目标';
    const errorText = run.error ? ` · ${run.error}` : '';
    return `${run.at} · ${targetText} · ${resultText}${errorText}`;
  }, []);

  const getErrorMessage = useCallback((error: unknown, fallback: string) => {
    if (error instanceof Error && error.message.trim()) {
      return error.message;
    }
    if (typeof error === 'string' && error.trim()) {
      return error;
    }
    if (error && typeof error === 'object' && 'message' in error) {
      const messageValue = (error as { message?: unknown }).message;
      if (typeof messageValue === 'string' && messageValue.trim()) {
        return messageValue;
      }
    }
    if (error && typeof error === 'object' && 'error' in error) {
      const errorValue = (error as { error?: unknown }).error;
      if (typeof errorValue === 'string' && errorValue.trim()) {
        return errorValue;
      }
    }
    return fallback;
  }, []);

  const buildBackupConfigInput = useCallback(
    (values: Record<string, any>): SaveBackupConfigInput => ({
      targetMode: (values.targetMode || 'local') as 'local' | 'cos',
      retentionCount: Number(values.retentionCount || 30),
      endpoint: values.endpoint || '',
      bucket: values.bucket || '',
      region: values.region || '',
      pathPrefix: values.pathPrefix || '',
      secretId: values.secretId || undefined,
      secretKey: values.secretKey || undefined,
      exportDefaultPassword: values.exportDefaultPassword || undefined,
    }),
    []
  );

  const mapSettingsToForm = useCallback(
    (
      settingsData: UserSetting[],
      secState: MasterPasswordState | null,
      backupConfig: BackupConfig | null
    ) => {
      const formData: Record<string, any> = {};
      settingsData.forEach((setting: UserSetting) => {
        let key = setting.key;
        if (key === 'security.auto_lock_timeout') {
          formData.autoLockMinutes = Math.max(
            1,
            Math.round(Number(setting.value) / 60)
          );
          return;
        }
        if (key === 'autoLockTime') {
          formData.autoLockMinutes =
            Number(setting.value) || formData.autoLockMinutes;
          return;
        }
        if (key === 'backup.auto_export_enabled' || key === 'backupEnabled')
          key = 'autoExportEnabled';
        if (key === 'backup.auto_export_frequency') key = 'autoExportFrequency';
        if (key === 'backup.auto_export_directory') key = 'autoExportDirectory';
        if (key === 'backup.auto_export_format') key = 'exportFormat';
        if (key === 'backup.auto_export_password')
          key = 'exportDefaultPassword';
        if (key === 'backup.auto_export_time_of_day')
          key = 'autoExportTimeOfDay';
        if (key === 'backup.auto_export_day_of_week')
          key = 'autoExportDayOfWeek';
        if (key === 'backup.auto_export_day_of_month')
          key = 'autoExportDayOfMonth';
        if (key === 'backup.auto_export_interval_minutes')
          key = 'autoExportIntervalMinutes';
        if (setting.type === 'boolean') {
          formData[key] = setting.value === 'true';
        } else if (setting.type === 'number') {
          formData[key] = Number(setting.value);
        } else {
          formData[key] = setting.value;
        }
      });
      const normalizedExportFormat = normalizeExportFormat(
        formData.exportFormat
      );
      return {
        // 先展开后端返回的所有设置项，保证未显式处理的键（如密码生成器相关）也能回填到表单
        ...formData,
        // 再按需覆盖 UI 与安全相关的默认值
        theme: formData.theme || 'auto',
        language: formData.language || 'zh-CN',
        uiListDensity: formData.uiListDensity || 'comfortable',
        uiFontSize: formData.uiFontSize || 'normal',
        exportDefaultPassword: '',
        exportFormat: backupConfig?.exportFormat || normalizedExportFormat,
        autoExportEnabled: formData.autoExportEnabled ?? false,
        autoExportFrequency: formData.autoExportFrequency || 'daily',
        autoExportDirectory: formData.autoExportDirectory || '',
        targetMode: backupConfig?.targetMode || 'local',
        retentionCount: backupConfig?.retentionCount ?? 30,
        endpoint: backupConfig?.endpoint || '',
        bucket: backupConfig?.bucket || '',
        region: backupConfig?.region || '',
        pathPrefix: backupConfig?.pathPrefix || '',
        secretId: '',
        secretKey: '',
        // 自动导出时间细节
        autoExportTimeOfDay: formData.autoExportTimeOfDay || '02:00',
        autoExportDayOfWeek: formData.autoExportDayOfWeek ?? 1,
        autoExportDayOfMonth: formData.autoExportDayOfMonth ?? 1,
        autoExportIntervalMinutes: formData.autoExportIntervalMinutes ?? 60,
        requireMasterPassword:
          secState?.requireMasterPassword ??
          formData.requireMasterPassword ??
          false,
        autoLockMinutes:
          secState?.autoLockMinutes ?? formData.autoLockMinutes ?? 5,
        // 密码生成器相关：如有存储值则使用存储值，否则退回到后端默认/内置默认
        'security.password_generator_length':
          formData['security.password_generator_length'] ?? 16,
        'security.password_generator_include_uppercase':
          formData['security.password_generator_include_uppercase'] ?? true,
        'security.password_generator_include_lowercase':
          formData['security.password_generator_include_lowercase'] ?? true,
        'security.password_generator_include_numbers':
          formData['security.password_generator_include_numbers'] ?? true,
        'security.password_generator_include_symbols':
          formData['security.password_generator_include_symbols'] ?? true,
      };
    },
    [normalizeExportFormat]
  );

  const reloadBackupConfig = useCallback(async () => {
    const backupConfig = await backupService.getBackupConfig();
    setSavedBackupConfig(backupConfig);
    form.setFieldsValue({
      targetMode: backupConfig.targetMode,
      retentionCount: backupConfig.retentionCount,
      endpoint: backupConfig.endpoint,
      bucket: backupConfig.bucket,
      region: backupConfig.region,
      pathPrefix: backupConfig.pathPrefix,
      exportFormat: backupConfig.exportFormat,
    });
    return backupConfig;
  }, [form]);

  const loadSecurityState = useCallback(async () => {
    const state = await securityService.getSecurityState();
    setSecurityState(state);
    form.setFieldsValue({
      requireMasterPassword: state.requireMasterPassword,
      autoLockMinutes: state.autoLockMinutes,
    });
  }, [form]);

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        const settingsData = await settingsService.listSettings();
        const secState = await securityService.getSecurityState();
        const backupConfig = await reloadBackupConfig();
        setSecurityState(secState);
        const mapped = mapSettingsToForm(settingsData, secState, backupConfig);
        form.setFieldsValue(mapped);
        setInitialAutoExportEnabled(!!mapped.autoExportEnabled);
      } catch (error) {
        message.error('加载设置失败');
        reportError('SETTINGS_LOAD_FAILED', '加载设置失败', error);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [form, mapSettingsToForm, reloadBackupConfig]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const bind = async () => {
      if (!window.electronAPI.onAutoExportDone) return;
      unlisten = await window.electronAPI.onAutoExportDone(async (payload) => {
        try {
          await reloadBackupConfig();
        } catch (error) {
          reportError('SETTINGS_RELOAD_BACKUP_STATUS_FAILED', '刷新备份状态失败', error);
        }
        if (payload.success) {
          message.success('自动备份执行成功');
        } else {
          message.error(payload.error || '自动备份执行失败');
        }
      });
    };
    bind();
    return () => {
      unlisten?.();
    };
  }, [reloadBackupConfig]);

  const handleSave = async () => {
    try {
      setLoading(true);
      const values = form.getFieldsValue();
      const exportFormat = normalizeExportFormat(values.exportFormat);
      const autoExportEnabled = !!values.autoExportEnabled;
      const nextTargetMode = values.targetMode || 'local';
      const hasArchivePasswordInput =
        !!values.exportDefaultPassword &&
        String(values.exportDefaultPassword).trim().length >= 4;
      if (
        autoExportEnabled &&
        exportFormat === 'encrypted_zip' &&
        !hasArchivePasswordInput &&
        !hasSavedArchivePassword
      ) {
        message.error('开启自动导出并选择加密ZIP时，请先设置至少4位的密码');
        setLoading(false);
        return;
      }
      if (
        autoExportEnabled &&
        nextTargetMode === 'local' &&
        (!values.autoExportDirectory ||
          !String(values.autoExportDirectory).trim())
      ) {
        message.error('开启本地自动导出时，请先选择导出目录');
        setLoading(false);
        return;
      }
      if (
        nextTargetMode === 'cos' &&
        (!values.endpoint || !values.bucket || !values.region)
      ) {
        message.error('启用云备份前，请先完整填写 Endpoint、Bucket 和 Region');
        setLoading(false);
        return;
      }

      if (values.autoLockMinutes != null) {
        const seconds = Math.max(1, Number(values.autoLockMinutes)) * 60;
        await settingsService.setSetting(
          'security.auto_lock_timeout',
          String(seconds),
          'number',
          'security',
          '自动锁定时间（秒）'
        );
      }
      // 主密码启停必须走专门的验证弹窗流程，不能在通用“保存设置”里顺带触发。

      // 保存其他设置项
      for (const [key, value] of Object.entries(values)) {
        if (
          [
            'autoLockMinutes',
            'requireMasterPassword',
            'autoExportEnabled',
            'autoExportFrequency',
            'autoExportDirectory',
            'exportFormat',
            'exportDefaultPassword',
            'backupEnabled',
            'autoExportTimeOfDay',
            'autoExportDayOfWeek',
            'autoExportDayOfMonth',
            'autoExportIntervalMinutes',
            'targetMode',
            'retentionCount',
            'endpoint',
            'bucket',
            'region',
            'pathPrefix',
            'secretId',
            'secretKey',
          ].includes(key)
        )
          continue;
        if (value === undefined) continue;
        await settingsService.setSetting(key, String(value));
      }
      await settingsService.setSetting(
        'backup.auto_export_format',
        exportFormat,
        'string',
        'backup',
        '自动导出格式'
      );
      await settingsService.setSetting(
        'backup.auto_export_directory',
        values.autoExportDirectory || '',
        'string',
        'backup',
        '自动导出目录'
      );
      await settingsService.setSetting(
        'backup.auto_export_enabled',
        String(autoExportEnabled),
        'boolean',
        'backup',
        '是否开启自动导出'
      );
      await settingsService.setSetting(
        'backup.auto_export_frequency',
        values.autoExportFrequency || 'daily',
        'string',
        'backup',
        '自动导出频率'
      );
      // 自动导出时间配置
      const timeOfDay = values.autoExportTimeOfDay || '02:00';
      await settingsService.setSetting(
        'backup.auto_export_time_of_day',
        timeOfDay,
        'string',
        'backup',
        '自动导出时间（每日/每周/每月，格式 HH:mm）'
      );
      const dayOfWeek = values.autoExportDayOfWeek ?? 1;
      await settingsService.setSetting(
        'backup.auto_export_day_of_week',
        String(dayOfWeek),
        'number',
        'backup',
        '自动导出周几（1=周一 ... 7=周日）'
      );
      const dayOfMonth = values.autoExportDayOfMonth ?? 1;
      await settingsService.setSetting(
        'backup.auto_export_day_of_month',
        String(dayOfMonth),
        'number',
        'backup',
        '自动导出日期（1-31）'
      );
      const intervalMinutes = values.autoExportIntervalMinutes ?? 60;
      await settingsService.setSetting(
        'backup.auto_export_interval_minutes',
        String(intervalMinutes),
        'number',
        'backup',
        '自动导出间隔（分钟，every_minute 模式）'
      );
      await backupService.saveBackupConfig(buildBackupConfigInput(values));
      await reloadBackupConfig();
      form.setFieldsValue({
        exportDefaultPassword: '',
        secretId: '',
        secretKey: '',
      });
      await loadSecurityState();
      message.success('设置保存成功');
      if (onClose) onClose();
    } catch (error) {
      const msg = getErrorMessage(error, '保存设置失败');
      message.error(msg);
      reportError('SETTINGS_SAVE_FAILED', '保存设置失败', error);
    } finally {
      setLoading(false);
    }
  };

  const handleReset = async () => {
    try {
      setLoading(true);
      const res = await settingsService.resetAllSettingsToDefault();
      if (!res.success) throw new Error(res.error || '重置失败');
      await securityService.setRequireMasterPassword(false);
      const settingsData = await settingsService.listSettings();
      const secState = await securityService.getSecurityState();
      const backupConfig = await reloadBackupConfig();
      setSecurityState(secState);
      const mapped = mapSettingsToForm(settingsData, secState, backupConfig);
      form.setFieldsValue(mapped);
      setInitialAutoExportEnabled(!!mapped.autoExportEnabled);
      await loadSecurityState();
      message.success('已重置为默认设置');
    } catch (error) {
      message.error('重置设置失败');
      reportError('SETTINGS_RESET_FAILED', '重置设置失败', error);
    } finally {
      setLoading(false);
    }
  };

  const handlePickExportDirectory = async () => {
    try {
      setSelectingExportDirectory(true);
      const currentPath = form.getFieldValue('autoExportDirectory');
      const picked = await backupService.pickExportDirectory({
        defaultPath: currentPath,
      });
      if (picked !== null) {
        form.setFieldsValue({ autoExportDirectory: picked });
      }
    } catch (error) {
      message.error('选择自动导出目录失败');
      reportError(
        'SETTINGS_PICK_EXPORT_DIRECTORY_FAILED',
        '选择自动导出目录失败',
        error
      );
    } finally {
      setSelectingExportDirectory(false);
    }
  };

  const handleCheckIntegrity = async () => {
    try {
      const r = await check();
      const errCount = r.errors.length;
      const warnCount = r.warnings.length;
      message.success(`完整性检查完成，错误 ${errCount}，警告 ${warnCount}`);
    } catch (error) {
      message.error('完整性检查失败');
      reportError('SETTINGS_CHECK_INTEGRITY_FAILED', '完整性检查失败', error);
    }
  };

  const handleRepairIntegrity = async () => {
    try {
      const r = await repair();
      message.success(
        `修复完成：${r.repaired.length} 条修复，${r.failed.length} 条失败`
      );
    } catch (error) {
      message.error('完整性修复失败');
      reportError('SETTINGS_REPAIR_INTEGRITY_FAILED', '完整性修复失败', error);
    }
  };

  const handleTestCloudConnection = async () => {
    try {
      const values = form.getFieldsValue();
      setCloudTesting(true);
      const payload: BackupCloudTestInput = {
        endpoint: values.endpoint || '',
        bucket: values.bucket || '',
        region: values.region || '',
        pathPrefix: values.pathPrefix || '',
        exportDefaultPassword: values.exportDefaultPassword || '',
      };
      const secretId = String(values.secretId || '').trim();
      if (secretId) {
        payload.secretId = secretId;
      }
      const secretKey = String(values.secretKey || '').trim();
      if (secretKey) {
        payload.secretKey = secretKey;
      }
      const result = await backupService.testBackupCloudConnection(payload);
      if (!result.success) {
        throw new Error(result.message);
      }
      await backupService.saveBackupConfig(buildBackupConfigInput(values));
      await reloadBackupConfig();
      form.setFieldsValue({
        exportDefaultPassword: '',
        secretId: '',
        secretKey: '',
      });
      const successText = result.warning
        ? `${result.message}（${result.warning}）`
        : result.message;
      message.success(`${successText}，云备份配置已保存`);
    } catch (error) {
      const msg = getErrorMessage(error, '连接测试失败');
      message.error(msg);
      reportError('SETTINGS_TEST_CLOUD_CONNECTION_FAILED', '测试云备份连接失败', error);
    } finally {
      setCloudTesting(false);
    }
  };

  const handleManualCloudBackup = async () => {
    try {
      setCloudUploading(true);
      const file = await backupService.triggerManualCloudBackup();
      await reloadBackupConfig();
      message.success(file ? `云备份上传成功：${file}` : '云备份上传成功');
    } catch (error) {
      const msg = getErrorMessage(error, '云备份上传失败');
      message.error(msg);
      reportError('SETTINGS_MANUAL_CLOUD_BACKUP_FAILED', '手动云备份失败', error);
    } finally {
      setCloudUploading(false);
    }
  };

  const handleMasterSubmit = async () => {
    setMasterSaving(true);
    try {
      const values = await masterForm.validateFields();
      let res;

      if (masterMode === 'enable') {
        // 开启主密码
        res = await window.electronAPI.setRequireMasterPassword(
          true,
          values.newPassword,
          values.hint
        );
        if (res.success && res.state) {
          message.success('主密码已开启,应用已锁定');
        }
      } else if (masterMode === 'disable') {
        // 关闭主密码
        res = await window.electronAPI.setRequireMasterPassword(
          false,
          undefined,
          undefined,
          values.currentPassword
        );
        if (res.success && res.state) {
          message.success('主密码已关闭,应用已解锁');
        }
      } else if (masterMode === 'update') {
        // 修改主密码
        res = await window.electronAPI.updateMasterPassword(
          values.currentPassword,
          values.newPassword,
          values.hint
        );
        if (res.success && res.state) {
          message.success('主密码已更新');
        }
      } else {
        // 兼容旧的 set/remove 模式(如果还有的话)
        if (masterMode === 'remove') {
          res = await window.electronAPI.clearMasterPassword(values.currentPassword);
        } else {
          res = await window.electronAPI.setMasterPassword(values.newPassword, values.hint);
        }
      }

      if (!res.success) throw new Error(res.error || '操作失败');
      setSecurityState(res.state || null);
      form.setFieldsValue({
        requireMasterPassword:
          res.state?.requireMasterPassword ??
          form.getFieldValue('requireMasterPassword'),
        autoLockMinutes:
          res.state?.autoLockMinutes ?? form.getFieldValue('autoLockMinutes'),
      });
      setMasterModalVisible(false);
      masterForm.resetFields();
    } catch (error) {
      const msg = error instanceof Error ? error.message : '操作失败';
      message.error(msg);
      reportError(
        'SETTINGS_MASTER_PASSWORD_OPERATION_FAILED',
        '主密码操作失败',
        error
      );
      throw error;
    } finally {
      setMasterSaving(false);
    }
  };

  const tabItems = [
    {
      key: 'security',
      label: '安全与密码',
      children: (
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Row gutter={16}>
            <Col span={14}>
              <Card
                size="small"
                title="访问控制"
                headStyle={{ fontWeight: 600 }}
              >
                <Row gutter={12}>
                  <Col span={12}>
                    <Form.Item
                      label="自动锁定时间（分钟）"
                      name="autoLockMinutes"
                      tooltip="应用闲置多长时间后自动锁定"
                      style={{ marginBottom: 12 }}
                    >
                      <InputNumber
                        min={1}
                        max={120}
                        style={{ width: '100%' }}
                        placeholder="5"
                      />
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="启用主密码"
                      name="requireMasterPassword"
                      valuePropName="checked"
                      tooltip="启用后立即锁定应用,需要输入主密码解锁"
                      style={{ marginBottom: 12 }}
                    >
                      <Switch
                        checked={securityState?.hasMasterPassword && securityState?.requireMasterPassword}
                        onChange={(checked) => {
                          if (checked) {
                            // 开启:弹出设置密码对话框
                            setMasterMode('enable');
                            masterForm.resetFields();
                            setMasterModalVisible(true);
                          } else {
                            // 关闭:弹出验证密码对话框
                            setMasterMode('disable');
                            masterForm.resetFields();
                            setMasterModalVisible(true);
                          }
                        }}
                      />
                    </Form.Item>
                  </Col>
                </Row>
                <div
                  style={{
                    marginTop: 12,
                    padding: '12px',
                    background: '#f5f5f5',
                    borderRadius: 8,
                  }}
                >
                  <Space
                    direction="vertical"
                    size={8}
                    style={{ width: '100%' }}
                  >
                    <Typography.Text strong>
                      {securityState?.hasMasterPassword
                        ? '主密码保护开启'
                        : '主密码未开启'}
                    </Typography.Text>
                    <Typography.Text
                      type="secondary"
                      style={{ display: 'block' }}
                    >
                      {securitySummary}
                    </Typography.Text>
                    <Space wrap style={{ marginTop: 4 }}>
                      {securityState?.hasMasterPassword && (
                      <Button
                        type="primary"
                        onClick={() => {
                          setMasterMode('update');
                          masterForm.resetFields();
                          setMasterModalVisible(true);
                        }}
                      >
                        修改主密码
                      </Button>
                    )}  </Space>
                  </Space>
                </div>
              </Card>
            </Col>
            <Col span={10}>
              <Card
                size="small"
                title="密码生成器"
                extra={<Tag color="blue">常用</Tag>}
                headStyle={{ fontWeight: 600 }}
              >
                <Row gutter={12}>
                  <Col span={24}>
                    <Form.Item
                      label="默认密码长度"
                      name="security.password_generator_length"
                      style={{ marginBottom: 12 }}
                    >
                      <InputNumber
                        min={4}
                        max={64}
                        style={{ width: '100%' }}
                        placeholder="16"
                      />
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="包含大写字母"
                      name="security.password_generator_include_uppercase"
                      valuePropName="checked"
                      style={{ marginBottom: 8 }}
                    >
                      <Switch />
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="包含小写字母"
                      name="security.password_generator_include_lowercase"
                      valuePropName="checked"
                      style={{ marginBottom: 8 }}
                    >
                      <Switch />
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="包含数字"
                      name="security.password_generator_include_numbers"
                      valuePropName="checked"
                      style={{ marginBottom: 8 }}
                    >
                      <Switch />
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="包含特殊字符"
                      name="security.password_generator_include_symbols"
                      valuePropName="checked"
                      style={{ marginBottom: 8 }}
                    >
                      <Switch />
                    </Form.Item>
                  </Col>
                </Row>
              </Card>
            </Col>
          </Row>
        </Space>
      ),
    },
    {
      key: 'ui',
      label: '界面与体验',
      children: (
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Card
            size="small"
            title="界面设置"
            extra={<Tag color="gold">清爽</Tag>}
            headStyle={{ fontWeight: 600 }}
          >
            <Row gutter={12}>
              <Col span={12}>
                <Form.Item
                  label="主题"
                  name="theme"
                  style={{ marginBottom: 12 }}
                >
                  <Select placeholder="选择主题">
                    <Option value="light">浅色主题</Option>
                    <Option value="dark">深色主题</Option>
                    <Option value="auto">跟随系统</Option>
                  </Select>
                </Form.Item>
              </Col>
              <Col span={12}>
                <Form.Item
                  label="语言"
                  name="language"
                  style={{ marginBottom: 12 }}
                >
                  <Select placeholder="选择语言">
                    <Option value="zh-CN">简体中文</Option>
                    <Option value="en-US">English</Option>
                  </Select>
                </Form.Item>
              </Col>
            </Row>

            <Row gutter={12}>
              <Col span={12}>
                <Form.Item
                  label="列表密度"
                  name="uiListDensity"
                  tooltip="影响表格、列表等组件的间距"
                  style={{ marginBottom: 12 }}
                >
                  <Select placeholder="选择列表密度">
                    <Option value="comfortable">标准</Option>
                    <Option value="compact">紧凑</Option>
                    <Option value="spacious">宽松</Option>
                  </Select>
                </Form.Item>
              </Col>
              <Col span={12}>
                <Form.Item
                  label="界面字体大小"
                  name="uiFontSize"
                  style={{ marginBottom: 12 }}
                >
                  <Select placeholder="选择字体大小">
                    <Option value="small">小</Option>
                    <Option value="normal">中</Option>
                    <Option value="large">大</Option>
                  </Select>
                </Form.Item>
              </Col>
            </Row>
          </Card>
        </Space>
      ),
    },
    {
      key: 'data',
      label: '备份与数据',
      children: (
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Row gutter={16}>
            <Col span={14}>
              <Card
                size="small"
                title="导出与自动备份"
                extra={
                  <Form.Item
                    name="autoExportEnabled"
                    valuePropName="checked"
                    style={{ marginBottom: 0 }}
                  >
                    <Switch
                      size="small"
                      checkedChildren="自动导出"
                      unCheckedChildren="自动导出"
                    />
                  </Form.Item>
                }
                headStyle={{ fontWeight: 600 }}
              >
                <Row gutter={12}>
                  <Col span={12}>
                    <Form.Item
                      label="备份目标"
                      name="targetMode"
                      style={{ marginBottom: 12 }}
                    >
                      <Select placeholder="选择备份目标">
                        <Option value="local">本地目录</Option>
                        <Option value="cos">腾讯云 COS</Option>
                      </Select>
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="保留份数"
                      name="retentionCount"
                      tooltip="仅保留最近 N 份备份，默认 30"
                      style={{ marginBottom: 12 }}
                    >
                      <InputNumber min={1} max={365} style={{ width: '100%' }} />
                    </Form.Item>
                  </Col>
                </Row>

                <Row gutter={12}>
                  <Col span={12}>
                    <Form.Item
                      label="默认导出格式"
                      name="exportFormat"
                      style={{ marginBottom: 12 }}
                    >
                      <Select placeholder="选择导出格式">
                        <Option value="json">JSON</Option>
                        <Option value="encrypted_zip">加密ZIP</Option>
                      </Select>
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="加密ZIP默认密码"
                      name="exportDefaultPassword"
                      tooltip="用于加密ZIP导出，至少4位。未设置则无法快速导出加密包。"
                      rules={[{ min: 4, message: '至少4位' }]}
                      style={{ marginBottom: 12 }}
                      extra={
                        hasSavedArchivePassword
                          ? '已保存加密密码。留空表示保持不变。'
                          : undefined
                      }
                    >
                      <Input.Password
                        placeholder={
                          hasSavedArchivePassword ? '留空表示保持当前密码' : '可选，至少4位'
                        }
                      />
                    </Form.Item>
                  </Col>
                </Row>

                <Row gutter={12}>
                  <Col span={12}>
                    <Form.Item
                      label="自动导出频率"
                      name="autoExportFrequency"
                      tooltip="每分钟/每日/每周/每月定期导出到指定目录"
                      style={{ marginBottom: 12 }}
                    >
                      <Select
                        placeholder="选择自动导出频率"
                        disabled={!autoExportEnabled}
                      >
                        <Option value="every_minute">每分钟</Option>
                        <Option value="daily">每日</Option>
                        <Option value="weekly">每周</Option>
                        <Option value="monthly">每月</Option>
                      </Select>
                    </Form.Item>
                  </Col>
                  <Col span={12}>
                    <Form.Item
                      label="自动导出目录"
                      name="autoExportDirectory"
                      tooltip="目标为本地目录时必须选择导出目录"
                      style={{ marginBottom: 12 }}
                    >
                      <Input
                        placeholder="选择自动导出保存目录"
                        readOnly
                        disabled={!autoExportEnabled || targetMode !== 'local'}
                        addonAfter={
                          <Button
                            size="small"
                            icon={<FolderOpenOutlined />}
                            onClick={handlePickExportDirectory}
                            loading={selectingExportDirectory}
                            disabled={!autoExportEnabled || targetMode !== 'local'}
                          >
                            选择
                          </Button>
                        }
                      />
                    </Form.Item>
                  </Col>
                </Row>

                {autoExportEnabled &&
                  autoExportFrequency === 'every_minute' && (
                    <Row gutter={12}>
                      <Col span={12}>
                        <Form.Item
                          label="导出间隔（分钟）"
                          name="autoExportIntervalMinutes"
                          tooltip="每隔多少分钟自动导出一次"
                          style={{ marginBottom: 12 }}
                        >
                          <InputNumber
                            min={1}
                            max={1440}
                            style={{ width: '100%' }}
                            placeholder="例如 60 表示每 60 分钟"
                          />
                        </Form.Item>
                      </Col>
                    </Row>
                  )}

                {autoExportEnabled && autoExportFrequency === 'daily' && (
                  <Row gutter={12}>
                    <Col span={12}>
                      <Form.Item
                        label="每日执行时间"
                        name="autoExportTimeOfDay"
                        tooltip="格式为 HH:mm，例如 18:00"
                        style={{ marginBottom: 12 }}
                      >
                        <Input placeholder="例如 18:00" />
                      </Form.Item>
                    </Col>
                  </Row>
                )}

                {autoExportEnabled && autoExportFrequency === 'weekly' && (
                  <Row gutter={12}>
                    <Col span={12}>
                      <Form.Item
                        label="每周执行日"
                        name="autoExportDayOfWeek"
                        tooltip="选择每周哪一天执行自动导出"
                        style={{ marginBottom: 12 }}
                      >
                        <Select placeholder="选择星期几">
                          <Option value={1}>周一</Option>
                          <Option value={2}>周二</Option>
                          <Option value={3}>周三</Option>
                          <Option value={4}>周四</Option>
                          <Option value={5}>周五</Option>
                          <Option value={6}>周六</Option>
                          <Option value={7}>周日</Option>
                        </Select>
                      </Form.Item>
                    </Col>
                    <Col span={12}>
                      <Form.Item
                        label="执行时间"
                        name="autoExportTimeOfDay"
                        tooltip="格式为 HH:mm，例如 18:00"
                        style={{ marginBottom: 12 }}
                      >
                        <Input placeholder="例如 18:00" />
                      </Form.Item>
                    </Col>
                  </Row>
                )}

                {autoExportEnabled && autoExportFrequency === 'monthly' && (
                  <Row gutter={12}>
                    <Col span={12}>
                      <Form.Item
                        label="每月执行日期"
                        name="autoExportDayOfMonth"
                        tooltip="1-31，超过当月天数时会自动调整为当月最后一天"
                        style={{ marginBottom: 12 }}
                      >
                        <InputNumber
                          min={1}
                          max={31}
                          style={{ width: '100%' }}
                          placeholder="例如 15 表示每月 15 日"
                        />
                      </Form.Item>
                    </Col>
                    <Col span={12}>
                      <Form.Item
                        label="执行时间"
                        name="autoExportTimeOfDay"
                        tooltip="格式为 HH:mm，例如 18:00"
                        style={{ marginBottom: 12 }}
                      >
                        <Input placeholder="例如 18:00" />
                      </Form.Item>
                    </Col>
                  </Row>
                )}

                {targetMode === 'cos' && (
                  <>
                    <Collapse
                      bordered={false}
                      defaultActiveKey={['cloud']}
                      items={[
                        {
                          key: 'cloud',
                          label: '云备份配置',
                          children: (
                            <>
                              <Alert
                                type="info"
                                showIcon
                                style={{ marginBottom: 12 }}
                                message="云端仅上传加密 ZIP，AK/SK 将加密存储到本地数据库。"
                              />
                              <Row gutter={12}>
                                <Col span={12}>
                                  <Form.Item
                                    label="Endpoint"
                                    name="endpoint"
                                    style={{ marginBottom: 12 }}
                                  >
                                    <Input placeholder="例如 https://cos.ap-shanghai.myqcloud.com" />
                                  </Form.Item>
                                </Col>
                                <Col span={12}>
                                  <Form.Item
                                    label="Bucket"
                                    name="bucket"
                                    style={{ marginBottom: 12 }}
                                  >
                                    <Input placeholder="例如 my-bucket-1234567890" />
                                  </Form.Item>
                                </Col>
                              </Row>
                              <Row gutter={12}>
                                <Col span={12}>
                                  <Form.Item
                                    label="Region"
                                    name="region"
                                    style={{ marginBottom: 12 }}
                                  >
                                    <Input placeholder="例如 ap-shanghai" />
                                  </Form.Item>
                                </Col>
                                <Col span={12}>
                                  <Form.Item
                                    label="Path Prefix"
                                    name="pathPrefix"
                                    style={{ marginBottom: 12 }}
                                  >
                                    <Input placeholder="例如 backups/myloair" />
                                  </Form.Item>
                                </Col>
                              </Row>
                              <Row gutter={12}>
                                <Col span={12}>
                                  <Form.Item
                                    label="SecretId (AK)"
                                    name="secretId"
                                    style={{ marginBottom: 12 }}
                                    extra={
                                      secretIdMasked
                                        ? `已保存：${secretIdMasked}；输入新值将覆盖`
                                        : undefined
                                    }
                                  >
                                    <Input placeholder="输入新的 AK，留空保持不变" />
                                  </Form.Item>
                                </Col>
                                <Col span={12}>
                                  <Form.Item
                                    label="SecretKey (SK)"
                                    name="secretKey"
                                    style={{ marginBottom: 12 }}
                                    extra={
                                      hasSavedSecretKey
                                        ? '已保存 SK；输入新值将覆盖，当前不会明文回显'
                                        : undefined
                                    }
                                  >
                                    <Input.Password placeholder="输入新的 SK，留空保持不变" />
                                  </Form.Item>
                                </Col>
                              </Row>
                              <Space wrap>
                                <Button
                                  icon={<ApiOutlined />}
                                  onClick={handleTestCloudConnection}
                                  loading={cloudTesting}
                                >
                                  测试连接
                                </Button>
                                <Button
                                  type="primary"
                                  icon={<CloudUploadOutlined />}
                                  onClick={handleManualCloudBackup}
                                  loading={cloudUploading}
                                >
                                  立即上传云备份
                                </Button>
                              </Space>
                            </>
                          ),
                        },
                      ]}
                    />
                    <div
                      style={{
                        marginTop: 12,
                        padding: '12px',
                        background: '#f7f8fa',
                        borderRadius: 8,
                      }}
                    >
                      <Typography.Text strong style={{ display: 'block' }}>
                        最近备份状态
                      </Typography.Text>
                      <Typography.Text type="secondary" style={{ display: 'block', marginTop: 6 }}>
                        手动备份：{formatRunSummary(savedManualRun)}
                      </Typography.Text>
                      <Typography.Text type="secondary" style={{ display: 'block', marginTop: 6 }}>
                        自动备份：{formatRunSummary(savedAutoRun)}
                      </Typography.Text>
                      <Typography.Text type="secondary" style={{ display: 'block', marginTop: 6 }}>
                        自动失败通知冷却：同类错误 {failureCooldownMinutes} 分钟内不重复提醒
                      </Typography.Text>
                    </div>
                  </>
                )}
              </Card>
            </Col>
            <Col span={10}>
              <Card
                size="small"
                title="数据健康"
                extra={
                  <Space size="small">
                    <Tag color="green">快照</Tag>
                    <Button
                      size="small"
                      icon={<SafetyCertificateOutlined />}
                      onClick={handleCheckIntegrity}
                      loading={checking}
                    >
                      检查
                    </Button>
                    <Button
                      size="small"
                      icon={<ToolOutlined />}
                      onClick={handleRepairIntegrity}
                      loading={repairing}
                    >
                      修复
                    </Button>
                  </Space>
                }
                headStyle={{ fontWeight: 600 }}
              >
                <Space
                  direction="vertical"
                  size="small"
                  style={{ width: '100%' }}
                >
                  <div>
                    <Typography.Text strong>自动导出</Typography.Text>
                    <br />
                    <Typography.Text
                      style={{
                        color: autoExportStatus ? '#389e0d' : '#8c8c8c',
                      }}
                    >
                      {autoExportSummary}
                    </Typography.Text>
                  </div>
                  <Collapse
                    ghost
                    defaultActiveKey={report || repairResult ? ['report'] : []}
                    items={[
                      {
                        key: 'report',
                        label: (
                          <Space size={8}>
                            <CheckCircleOutlined />
                            <span>
                              检查结果
                              {report
                                ? `（错误 ${report.errors.length} · 警告 ${report.warnings.length}）`
                                : ''}
                            </span>
                          </Space>
                        ),
                        children: (
                          <div
                            style={{
                              maxHeight: 180,
                              overflow: 'auto',
                              padding: '4px 4px 0',
                            }}
                          >
                            {report ? (
                              <div>
                                <div style={{ marginBottom: 6 }}>
                                  错误 {report.errors.length}，警告{' '}
                                  {report.warnings.length}
                                </div>
                                {report.errors.map((e, idx) => (
                                  <div
                                    key={`err-${idx}`}
                                    style={{ color: '#cf1322' }}
                                  >
                                    {e}
                                  </div>
                                ))}
                                {report.warnings.map((w, idx) => (
                                  <div
                                    key={`warn-${idx}`}
                                    style={{ color: '#faad14' }}
                                  >
                                    {w}
                                  </div>
                                ))}
                              </div>
                            ) : (
                              <div>尚未执行检查</div>
                            )}
                          </div>
                        ),
                      },
                      {
                        key: 'repair',
                        label: (
                          <Space size={8}>
                            <ToolTwoTone twoToneColor="#52c41a" />
                            <span>
                              修复结果
                              {repairResult
                                ? `（已修复 ${repairResult.repaired.length} · 失败 ${repairResult.failed.length}）`
                                : ''}
                            </span>
                          </Space>
                        ),
                        children: (
                          <div
                            style={{
                              maxHeight: 180,
                              overflow: 'auto',
                              padding: '4px 4px 0',
                            }}
                          >
                            {repairResult ? (
                              <div>
                                <div style={{ marginBottom: 6 }}>
                                  已修复 {repairResult.repaired.length}，失败{' '}
                                  {repairResult.failed.length}
                                </div>
                                {repairResult.repaired.map((r, idx) => (
                                  <div
                                    key={`rep-${idx}`}
                                    style={{ color: '#52c41a' }}
                                  >
                                    {r}
                                  </div>
                                ))}
                                {repairResult.failed.map((f, idx) => (
                                  <div
                                    key={`fail-${idx}`}
                                    style={{ color: '#cf1322' }}
                                  >
                                    {f}
                                  </div>
                                ))}
                              </div>
                            ) : (
                              <div>尚未执行修复</div>
                            )}
                          </div>
                        ),
                      },
                    ]}
                  />
                </Space>
              </Card>
            </Col>
          </Row>
        </Space>
      ),
    },
  ];

  return (
    <div style={{ padding: '8px 0' }}>
      <Card
        bordered={false}
        style={{
          borderRadius: 12,
          background: 'linear-gradient(120deg, #f3f6ff 0%, #ffffff 100%)',
          boxShadow: '0 10px 30px rgba(0,0,0,0.04)',
        }}
        bodyStyle={{ padding: '12px 16px' }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <div>
            <Title level={4} style={{ margin: 0 }}>
              用户设置
            </Title>
            <Typography.Text type="secondary">
              分栏视图让你无需下拉即可完成常用操作
            </Typography.Text>
          </div>
          <Space size="small">
            <Tag color={securityState?.hasMasterPassword ? 'green' : 'orange'}>
              {securityState?.hasMasterPassword
                ? '主密码已开启'
                : '主密码未开启'}
            </Tag>
            <Tag color={autoExportStatus ? 'blue' : 'default'}>
              {autoExportStatus ? '自动导出开启' : '自动导出关闭'}
            </Tag>
          </Space>
        </div>
      </Card>

      <Form
        form={form}
        layout="vertical"
        size="middle"
        style={{ marginTop: 16 }}
      >
        <Tabs
          activeKey={activeTab}
          onChange={(key) => setActiveTab(key as 'security' | 'ui' | 'data')}
          items={tabItems}
        />
      </Form>

      <div
        style={{
          marginTop: 16,
          display: 'flex',
          justifyContent: 'flex-end',
          gap: 12,
        }}
      >
        <Button
          icon={<ReloadOutlined />}
          onClick={handleReset}
          loading={loading}
        >
          重置为默认
        </Button>
        <Button
          type="primary"
          icon={<SaveOutlined />}
          onClick={handleSave}
          loading={loading}
        >
          保存设置
        </Button>
      </div>

      <Modal
        title={
          masterMode === 'enable'
            ? '设置主密码'
            : masterMode === 'disable'
              ? '关闭主密码'
              : masterMode === 'update'
                ? '修改主密码'
                : masterMode === 'remove'
                  ? '关闭主密码'
                  : '设置主密码'
        }
        open={masterModalVisible}
        onCancel={() => {
          setMasterModalVisible(false);
          masterForm.resetFields();
        }}
        onOk={async () => {
          try {
            await masterForm.validateFields();
            await handleMasterSubmit();
          } catch {
            /* no-op */
          }
        }}
        okButtonProps={{ loading: masterSaving }}
        destroyOnClose
      >
        <Form layout="vertical" form={masterForm}>
          {/* 需要输入当前密码的模式: disable, update, remove */}
          {(masterMode === 'disable' || masterMode === 'update' || masterMode === 'remove') && (
            <Form.Item
              label="当前主密码"
              name="currentPassword"
              rules={[{ required: true, message: '请输入当前主密码' }]}
            >
              <Input.Password />
            </Form.Item>
          )}
          {/* 需要输入新密码的模式: enable, update, set */}
          {(masterMode === 'enable' || masterMode === 'update' || masterMode === 'set') && (
            <>
              <Form.Item
                label="新主密码"
                name="newPassword"
                rules={[
                  { required: true, message: '请输入新主密码' },
                  { min: 6, message: '至少6位字符' },
                ]}
              >
                <Input.Password placeholder="至少6位，建议包含字母和数字" />
              </Form.Item>
              <Form.Item
                label="确认新主密码"
                name="confirmPassword"
                dependencies={['newPassword']}
                rules={[
                  { required: true, message: '请再次输入新主密码' },
                  ({ getFieldValue }) => ({
                    validator(_, value) {
                      if (!value || getFieldValue('newPassword') === value) {
                        return Promise.resolve();
                      }
                      return Promise.reject(
                        new Error('两次输入的主密码不一致')
                      );
                    },
                  }),
                ]}
              >
                <Input.Password />
              </Form.Item>
              <Form.Item label="主密码提示（可选）" name="hint">
                <Input placeholder="仅自己能懂的提示" maxLength={100} />
              </Form.Item>
            </>
          )}
          {/* 警告提示 */}
          {(masterMode === 'disable' || masterMode === 'remove') && (
            <Alert
              type="warning"
              message="⚠️ 关闭主密码后，应用将不再需要密码解锁，请确认已备份数据。"
            />
          )}
        </Form>
      </Modal>
    </div>
  );
};

export default UserSettings;
