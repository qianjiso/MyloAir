import React, { useState } from 'react';
import {
  Modal,
  Form,
  Select,
  Input,
  Button,
  Space,
  Upload,
  message,
} from 'antd';
import * as backupService from '../services/backup';
import {
  DownloadOutlined,
  UploadOutlined,
  FileTextOutlined,
  LockOutlined,
  InboxOutlined,
} from '@ant-design/icons';
import { reportError } from '../utils/logging';

const { Option } = Select;
const { Dragger } = Upload;

interface ImportExportModalProps {
  visible: boolean;
  onClose: () => void;
}

const ImportExportModal: React.FC<ImportExportModalProps> = ({
  visible,
  onClose,
}) => {
  const [mode, setMode] = useState<'export' | 'import'>('export');
  const [exportForm] = Form.useForm();
  const [importForm] = Form.useForm();
  const [loading, setLoading] = useState(false);
  // 预览已取消
  const [uploadFile, setUploadFile] = useState<File | null>(null);

  // 导出数据
  const handleExport = async () => {
    try {
      setLoading(true);
      const values = await exportForm.validateFields();

      const data = await backupService.exportData(values as any);
      const isZip = values.format === 'encrypted_zip';
      const blob = new Blob([data as unknown as BlobPart], {
        type: isZip ? 'application/zip' : 'application/json',
      });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `passwords_backup_${new Date().toISOString().split('T')[0]}.${isZip ? 'zip' : 'json'}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      message.success('数据导出成功');
    } catch (error) {
      message.error('导出过程中发生错误');
      reportError('EXPORT_MODAL_EXPORT_FAILED', '导出过程中发生错误', error);
    } finally {
      setLoading(false);
    }
  };

  // 导入数据
  const handleImport = async () => {
    if (!uploadFile) {
      message.error('请选择要导入的文件');
      return;
    }

    try {
      setLoading(true);
      const fmt = 'json' as const;

      // 读取文件内容
      const arrayBuffer = await uploadFile.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      const result = await backupService.importData(uint8Array, {
        format: fmt,
        mergeStrategy: 'merge',
        validateIntegrity: false,
        dryRun: false,
      });
      message.success(`导入成功，共处理 ${result.imported || 0} 条记录`);
    } catch (error) {
      message.error('导入过程中发生错误');
      reportError('EXPORT_MODAL_IMPORT_FAILED', '导入过程中发生错误', error);
    } finally {
      setLoading(false);
    }
  };

  // 文件上传配置
  const uploadProps = {
    accept: '.json',
    maxCount: 1,
    beforeUpload: (file: File) => {
      setUploadFile(file);
      importForm.setFieldsValue({ format: 'json' });
      return false; // 阻止自动上传
    },
    onRemove: () => {
      setUploadFile(null);
      importForm.setFieldsValue({ format: 'json' });
    },
  };

  return (
    <Modal
      title={
        <Space>
          {mode === 'export' ? <DownloadOutlined /> : <UploadOutlined />}
          数据{mode === 'export' ? '导出' : '导入'}
        </Space>
      }
      open={visible}
      onCancel={onClose}
      width={600}
      footer={[
        <Button key="cancel" onClick={onClose}>
          取消
        </Button>,
        <Button
          key="execute"
          type="primary"
          loading={loading}
          onClick={mode === 'export' ? handleExport : handleImport}
        >
          {mode === 'export' ? '导出' : '导入'}
        </Button>,
      ]}
    >
      <div style={{ marginBottom: 16 }}>
        <Select
          value={mode}
          onChange={(v) => {
            setMode(v as any);
            setUploadFile(null);
          }}
          style={{ width: 160 }}
        >
          <Option value="export">导出数据</Option>
          <Option value="import">导入数据</Option>
        </Select>
      </div>

      {mode === 'export' ? (
        <Form
          form={exportForm}
          layout="vertical"
          initialValues={{
            format: 'json',
          }}
        >
          <Form.Item
            label="导出格式"
            name="format"
            tooltip="选择导出文件的格式"
          >
            <Select>
              <Option value="json">
                <Space>
                  <FileTextOutlined />
                  JSON格式（完整数据结构）
                </Space>
              </Option>
              <Option value="encrypted_zip">
                <Space>
                  <LockOutlined />
                  加密ZIP（AES-256加密）
                </Space>
              </Option>
            </Select>
          </Form.Item>

          {/* 导出密码 */}
          <Form.Item
            noStyle
            shouldUpdate={(prev, cur) => prev.format !== cur.format}
          >
            {({ getFieldValue }) =>
              ['encrypted_zip'].includes(getFieldValue('format')) ? (
                <Form.Item
                  label="备份包密码"
                  name="archivePassword"
                  rules={[
                    { required: true, message: '请设置备份包密码' },
                    { min: 4, message: '至少4位' },
                  ]}
                >
                  <Input.Password
                    style={{ width: '100%' }}
                    placeholder="请输入密码"
                  />
                </Form.Item>
              ) : null
            }
          </Form.Item>

          <Form.Item
            noStyle
            shouldUpdate={(prev, cur) => prev.format !== cur.format}
          >
            {({ getFieldValue }) =>
              getFieldValue('format') === 'zip' ? null : null
            }
          </Form.Item>

          {/* 已取消密码强度过滤与压缩级别选项 */}
        </Form>
      ) : (
        <Form
          form={importForm}
          layout="vertical"
          initialValues={{
            format: 'json',
          }}
        >
          <Form.Item label="选择文件" required>
            <Dragger {...uploadProps}>
              <p className="ant-upload-drag-icon">
                <InboxOutlined />
              </p>
              <p className="ant-upload-text">点击或拖拽文件到此区域上传</p>
              <p className="ant-upload-hint">支持 JSON 文件</p>
            </Dragger>
          </Form.Item>

          {/* 已选择文件提示取消 */}

          <Form.Item
            label="文件格式"
            name="format"
            tooltip="指定导入文件的格式"
          >
            <Select>
              <Option value="json">JSON格式</Option>
            </Select>
          </Form.Item>

          {/* 导入选项取消，默认智能合并 */}
        </Form>
      )}
    </Modal>
  );
};

export default ImportExportModal;
