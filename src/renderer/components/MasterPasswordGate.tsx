import React, { useMemo, useState } from 'react';
import { Alert, Button, Card, Form, Input, Space, Typography } from 'antd';
import { LockOutlined, SafetyCertificateOutlined } from '@ant-design/icons';
import type { MasterPasswordState } from '../../shared/types';

interface MasterPasswordGateProps {
  visible: boolean;
  state?: MasterPasswordState | null;
  loading?: boolean;
  onUnlock: (password: string) => Promise<void> | void;
  onSetup: (password: string, hint?: string) => Promise<void> | void;
}

const MasterPasswordGate: React.FC<MasterPasswordGateProps> = ({
  visible,
  state,
  loading,
  onUnlock,
  onSetup,
}) => {
  const [form] = Form.useForm();
  const [error, setError] = useState<string>('');
  const hasMaster = state?.hasMasterPassword;

  const title = useMemo(() => {
    if (!hasMaster) return '设置主密码';
    return '输入主密码解锁';
  }, [hasMaster]);

  const description = useMemo(() => {
    if (!hasMaster) return '主密码用于锁定应用和保护数据，请务必牢记。';
    return '已启用主密码访问控制，输入后解锁应用。';
  }, [hasMaster]);

  const handleFinish = async (values: any) => {
    try {
      setError('');
      if (hasMaster) {
        await onUnlock(values.password);
      } else {
        if (values.password !== values.confirmPassword) {
          setError('两次输入的主密码不一致');
          return;
        }
        await onSetup(values.password, values.hint);
      }
      form.resetFields();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '操作失败';
      setError(msg);
    }
  };

  if (!visible) return null;

  return (
    <div className="master-lock-overlay">
      <Card
        className="master-lock-card"
        title={(
          <Space>
            <LockOutlined />
            <span>{title}</span>
          </Space>
        )}
        extra={<SafetyCertificateOutlined />}
      >
        <Typography.Paragraph style={{ marginBottom: 12 }}>{description}</Typography.Paragraph>
        {state?.hint && hasMaster && (
          <Alert type="info" message={`提示：${state.hint}`} style={{ marginBottom: 12 }} />
        )}
        {error && <Alert type="error" message={error} style={{ marginBottom: 12 }} />}
        <Form form={form} layout="vertical" onFinish={handleFinish}>
          <Form.Item
            label="主密码"
            name="password"
            rules={[{ required: true, message: '请输入主密码' }, { min: 6, message: '至少6位字符' }]}
          >
            <Input.Password placeholder={hasMaster ? '输入主密码以解锁' : '设置主密码'} autoFocus />
          </Form.Item>
          {!hasMaster && (
            <>
              <Form.Item
                label="确认主密码"
                name="confirmPassword"
                rules={[{ required: true, message: '请再次输入主密码' }]}
              >
                <Input.Password placeholder="再次输入主密码" />
              </Form.Item>
              <Form.Item label="密码提示（可选）" name="hint">
                <Input placeholder="例如：常用短语或记忆提示" maxLength={100} />
              </Form.Item>
            </>
          )}
          <Form.Item style={{ marginBottom: 0 }}>
            <Button type="primary" htmlType="submit" block loading={loading}>
              {hasMaster ? '解锁' : '保存主密码并解锁'}
            </Button>
          </Form.Item>
        </Form>
        {state?.requireMasterPassword && (
          <Typography.Text type="secondary" style={{ fontSize: 12 }}>
            自动锁定：{state.autoLockMinutes} 分钟无操作后需要重新输入主密码
          </Typography.Text>
        )}
      </Card>
    </div>
  );
};

export default MasterPasswordGate;
