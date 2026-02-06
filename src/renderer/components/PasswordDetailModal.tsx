import React, { useState, useEffect } from 'react';
import { 
  Modal, 
  Form, 
  Input, 
  Button, 
  Select, 
  Space, 
  message,
  Typography,
  Row,
  Col
} from 'antd';
import { 
  CopyOutlined, 
  EditOutlined,
  DeleteOutlined
} from '@ant-design/icons';
import PasswordGenerator from './PasswordGenerator';
import { reportError } from '../utils/logging';

const { Title } = Typography;
const { TextArea } = Input;
const { Option } = Select;
 

interface PasswordDetailModalProps {
  visible: boolean;
  password: any;
  groups: any[];
  onClose: () => void;
  onSave: (password: any) => void;
  onDelete: (id: number) => void;
  onEdit?: (password: any) => void;
  mode: 'view' | 'edit' | 'create';
}

const PasswordDetailModal: React.FC<PasswordDetailModalProps> = ({
  visible,
  password,
  groups,
  onClose,
  onSave,
  onDelete,
  onEdit,
  mode
}) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [passwordGeneratorVisible, setPasswordGeneratorVisible] = useState(false);

  useEffect(() => {
    if (visible && password) {
      form.setFieldsValue({
        title: password.title || '',
        username: password.username || '',
        password: password.password || '',
        url: password.url || '',
        notes: password.notes || '',
        group_id: password.group_id || null
      });
      setShowPassword(false);
    }
  }, [visible, password, form]);

  useEffect(() => {
    if (visible && mode === 'create') {
      form.resetFields();
      const gid = password?.group_id ?? null;
      if (gid !== null && typeof gid !== 'undefined') {
        form.setFieldsValue({ group_id: gid });
      }
      setShowPassword(false);
    }
  }, [visible, mode, form, password?.group_id]);

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      setLoading(true);

      const passwordData = {
        ...values,
      };

      if (mode === 'create') {
        await onSave(passwordData);
      } else {
        await onSave({ ...passwordData, id: password.id });
      }
      onClose();
    } catch (error) {
      reportError('PASSWORD_DETAIL_SAVE_FAILED', '保存失败', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = async (text: string, fieldName: string) => {
    try {
      await navigator.clipboard.writeText(text);
      message.success(`${fieldName}已复制到剪贴板`);
    } catch (error) {
      message.error('复制失败');
    }
  };

  const handleGeneratePassword = (generatedPassword: string) => {
    form.setFieldsValue({ password: generatedPassword });
    setPasswordGeneratorVisible(false);
    setShowPassword(true);
  };

  const handleDelete = () => {
    Modal.confirm({
      title: '确认删除',
      content: '确定要删除这个密码吗？此操作不可恢复。',
      okText: '删除',
      okType: 'danger',
      cancelText: '取消',
      onOk: () => {
        onDelete(password.id);
        onClose();
      }
    });
  };

  const isReadonly = mode === 'view';
  const readonlyInputClass = isReadonly ? 'readonly-input' : undefined;

  return (
    <>
      <Modal
        title={
          <Title level={4} style={{ margin: 0 }}>
            {mode === 'create' ? '创建密码' : mode === 'edit' ? '编辑密码' : '密码详情'}
          </Title>
        }
        open={visible}
        onCancel={onClose}
        width={800}
        footer={
          <Space>
            <Button onClick={onClose}>
              {isReadonly ? '关闭' : '取消'}
            </Button>
            {!isReadonly && (
              <Button type="primary" onClick={handleSave} loading={loading}>
                保存
              </Button>
            )}
            {mode === 'view' && (
              <Button 
                type="default" 
                icon={<EditOutlined />}
                onClick={() => {
                  if (password && onEdit) {
                    onEdit(password);
                  }
                }}
              >
                编辑
              </Button>
            )}
            {mode === 'view' && (
              <Button 
                danger 
                icon={<DeleteOutlined />}
                onClick={handleDelete}
              >
                删除
              </Button>
            )}
          </Space>
        }
      >
        <Form
          form={form}
          layout="vertical"
        >
              <Row gutter={16}>
                <Col span={24}>
                  <Form.Item
                    label="标题"
                    name="title"
                    rules={[{ required: true, message: '请输入标题' }]}
                  >
                    <Input
                      placeholder="请输入密码标题"
                      readOnly={isReadonly}
                      className={readonlyInputClass}
                    />
                  </Form.Item>
                </Col>
              </Row>

              <Row gutter={16}>
                <Col span={24}>
                  <Form.Item
                    label="用户名"
                    name="username"
                    rules={[{ required: true, message: '请输入用户名' }]}
                  >
                    <Input 
                      placeholder="请输入用户名"
                      readOnly={isReadonly}
                      className={readonlyInputClass}
                      onClick={() => {
                        if (!isReadonly) return;
                        const value = form.getFieldValue('username');
                        if (value) handleCopy(value, '用户名');
                      }}
                      suffix={
                        <Button
                          type="text"
                          icon={<CopyOutlined />}
                          onClick={() => {
                            const value = form.getFieldValue('username');
                            if (value) handleCopy(value, '用户名');
                          }}
                        />
                      }
                    />
                  </Form.Item>
                </Col>
              </Row>

              <Row gutter={16}>
                <Col span={24}>
                  <Form.Item
                    label="密码"
                    name="password"
                    rules={[{ required: true, message: '请输入密码' }]}
                  >
                    <Input.Password
                      placeholder="请输入密码"
                      readOnly={isReadonly}
                      className={readonlyInputClass}
                      onClick={() => {
                        if (!isReadonly) return;
                        const value = form.getFieldValue('password');
                        if (value) handleCopy(value, '密码');
                      }}
                      visibilityToggle={{
                        visible: showPassword,
                        onVisibleChange: setShowPassword
                      }}
                      suffix={
                        <Space>
                          <Button
                            type="text"
                            icon={<CopyOutlined />}
                            onClick={() => {
                              const value = form.getFieldValue('password');
                              if (value) handleCopy(value, '密码');
                            }}
                          />
                          {!isReadonly && (
                            <Button type="text" onClick={() => setPasswordGeneratorVisible(true)}>生成</Button>
                          )}
                        </Space>
                      }
                    />
                  </Form.Item>
                </Col>
              </Row>

              

              <Row gutter={16}>
                <Col span={24}>
                  <Form.Item
                    label="URL"
                    name="url"
                  >
                    <Input 
                      placeholder="请输入网址"
                      readOnly={isReadonly}
                      className={readonlyInputClass}
                      onClick={() => {
                        if (!isReadonly) return;
                        const value = form.getFieldValue('url');
                        if (value) handleCopy(value, 'URL');
                      }}
                      suffix={
                        <Button
                          type="text"
                          icon={<CopyOutlined />}
                          onClick={() => {
                            const value = form.getFieldValue('url');
                            if (value) handleCopy(value, 'URL');
                          }}
                        />
                      }
                    />
                  </Form.Item>
                </Col>
              </Row>

              <Row gutter={16}>
                <Col span={24}>
                  <Form.Item
                    label="分组"
                    name="group_id"
                    rules={[{ required: true, message: '请选择分组' }]}
                  >
                    <Select 
                      placeholder="请选择分组" 
                      disabled={isReadonly}
                      allowClear
                      showSearch
                      filterOption={(input, option) =>
                        (option?.children as unknown as string)?.toLowerCase().includes(input.toLowerCase()) ?? false
                      }
                    >
                      {groups.map(group => (
                        <Option key={group.id} value={group.id}>
                          {group.name}
                        </Option>
                      ))}
                    </Select>
                  </Form.Item>
                </Col>
              </Row>

              <Row gutter={16}>
                <Col span={24}>
                  <Form.Item
                    label="备注"
                    name="notes"
                  >
                    <TextArea 
                      placeholder="请输入备注信息" 
                      rows={3}
                      readOnly={isReadonly}
                      className={readonlyInputClass}
                      maxLength={1000}
                      showCount
                    />
                  </Form.Item>
                </Col>
              </Row>
        </Form>
      </Modal>

      <PasswordGenerator
        visible={passwordGeneratorVisible}
        onClose={() => setPasswordGeneratorVisible(false)}
        onGenerate={handleGeneratePassword}
      />
    </>
  );
};

export default PasswordDetailModal;
