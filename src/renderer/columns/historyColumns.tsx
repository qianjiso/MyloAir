import React from 'react';
import { Button, Space } from 'antd';
import { EyeOutlined, EyeInvisibleOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { formatTimestamp } from '../utils/time';

export interface PasswordHistoryRow {
  id?: number;
  old_password: string;
  new_password: string;
  changed_at?: string;
}

/**
 * 构建密码历史记录的列配置
 * @param visibleKeys 当前可见历史密码集合（键为 `old-<id>`/`new-<id>`）
 * @param toggle 可见性切换函数
 * @returns Antd Table 的列配置
 */
export function buildHistoryColumns(
  visibleKeys: Set<string>,
  toggle: (key: string) => void
): ColumnsType<PasswordHistoryRow> {
  return [
    {
      title: '旧密码',
      dataIndex: 'old_password',
      key: 'old_password',
      render: (text: string, record: PasswordHistoryRow) => {
        if (!text) return '-';
        const key = `old-${record.id}`;
        const isVisible = visibleKeys.has(key);
        return (
          <Space>
            <span style={{ fontFamily: isVisible ? 'monospace' : 'inherit' }}>
              {isVisible ? text : '••••••••'}
            </span>
            <Button type="link" size="small" icon={isVisible ? <EyeInvisibleOutlined /> : <EyeOutlined />} onClick={() => toggle(key)} />
          </Space>
        );
      },
    },
    {
      title: '新密码',
      dataIndex: 'new_password',
      key: 'new_password',
      render: (text: string, record: PasswordHistoryRow) => {
        if (!text) return '-';
        const key = `new-${record.id}`;
        const isVisible = visibleKeys.has(key);
        return (
          <Space>
            <span style={{ fontFamily: isVisible ? 'monospace' : 'inherit' }}>
              {isVisible ? text : '••••••••'}
            </span>
            <Button type="link" size="small" icon={isVisible ? <EyeInvisibleOutlined /> : <EyeOutlined />} onClick={() => toggle(key)} />
          </Space>
        );
      },
    },
    { title: '更改时间', dataIndex: 'changed_at', key: 'changed_at', render: (text: string) => formatTimestamp(text) },
    {
      title: '操作',
      key: 'action',
      render: (_: any, record: PasswordHistoryRow) => {
        const oldKey = `old-${record.id}`;
        const newKey = `new-${record.id}`;
        const oldVisible = visibleKeys.has(oldKey);
        const newVisible = visibleKeys.has(newKey);
        const rowVisible = oldVisible && newVisible;
        return (
          <Button
            type="link"
            size="small"
            icon={rowVisible ? <EyeInvisibleOutlined /> : <EyeOutlined />}
            onClick={() => {
              if (rowVisible) { toggle(oldKey); toggle(newKey); }
              else { toggle(oldKey); toggle(newKey); }
            }}
          >
            {rowVisible ? '隐藏' : '查看'}
          </Button>
        );
      }
    },
  ];
}

