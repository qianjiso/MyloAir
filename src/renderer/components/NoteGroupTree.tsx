import React from 'react';
import { Tree, Dropdown, Button, Popconfirm, message } from 'antd';
import type { DataNode } from 'antd/es/tree';
import { EditOutlined, DeleteOutlined } from '@ant-design/icons';
import { buildDropPlan } from '../utils/treeDnd';

const groupColorMap: Record<string, string> = {
  blue: '#1677ff',
  green: '#52c41a',
  red: '#f5222d',
  yellow: '#fadb14',
  purple: '#722ed1',
  orange: '#fa8c16',
  pink: '#eb2f96',
  gray: '#8c8c8c',
  cyan: '#13c2c2',
  teal: '#08979c',
  magenta: '#eb2f96',
  geekblue: '#2f54eb',
};

const getGroupColor = (color?: string) => {
  if (!color) return '#1677ff';
  return groupColorMap[color] || color;
};

export interface NoteGroupTreeProps {
  groupTree: any[];
  selectedGroupId?: number;
  onSelect: (selectedKeys: React.Key[]) => void;
  setGroupTree: (tree: any[]) => void;
  setGroups: (groups: any[]) => void;
  onEditGroup: (group: any) => void;
  onDeleteGroup: (id: number) => void;
}

/**
 * 分组树组件（便笺分组）
 * 负责渲染树与拖拽排序，并提供右键菜单操作
 */
const NoteGroupTree: React.FC<NoteGroupTreeProps> = ({
  groupTree,
  selectedGroupId,
  onSelect,
  setGroupTree,
  setGroups,
  onEditGroup,
  onDeleteGroup,
}) => {
  const renderNoteGroupTitle = (group: any) => (
    <Dropdown
      trigger={['contextMenu']}
      menu={{
        items: [
          {
            key: 'rename',
            label: '重命名',
            onClick: () => onEditGroup(group),
          },
          {
            key: 'new-child',
            label: '新建子分组',
            onClick: () =>
              onEditGroup({
                ...group,
                parent_id: group.id,
              }),
          },
          {
            key: 'move',
            label: '移动到分组',
            onClick: () => onEditGroup(group),
          },
          {
            key: 'delete',
            danger: true,
            label: '删除',
            onClick: () => group.id && onDeleteGroup(group.id),
          },
        ],
      }}
    >
      <div className="group-tree-node">
        <div className="group-tree-node__info">
          <span
            className="group-color-dot"
            style={{ backgroundColor: getGroupColor(group.color) }}
          />
          <span className="group-tree-node__name">{group.name}</span>
        </div>
        <div className="group-tree-node__actions">
          <Button
            type="text"
            size="small"
            icon={<EditOutlined />}
            onClick={(e) => {
              e.stopPropagation();
              onEditGroup(group);
            }}
          />
          {group.id && (
            <Popconfirm
              title="确定要删除这个分组吗？"
              onConfirm={(e) => {
                e?.stopPropagation();
                onDeleteGroup(group.id);
              }}
              okText="确定"
              cancelText="取消"
            >
              <Button
                type="text"
                size="small"
                danger
                icon={<DeleteOutlined />}
                onClick={(e) => e.stopPropagation()}
              />
            </Popconfirm>
          )}
        </div>
      </div>
    </Dropdown>
  );

  const buildNoteGroupNodes = (nodes: any[]): DataNode[] =>
    nodes.map((group) => ({
      key: group.id?.toString() || `note-${group.name}`,
      title: renderNoteGroupTitle(group),
      children:
        group.children && group.children.length > 0
          ? buildNoteGroupNodes(group.children)
          : undefined,
    }));

  const treeData: DataNode[] = buildNoteGroupNodes(groupTree as any);

  return (
    <Tree
      showLine
      treeData={treeData}
      onSelect={onSelect}
      selectedKeys={selectedGroupId ? [selectedGroupId.toString()] : []}
      style={{ background: 'transparent' }}
      defaultExpandAll={true}
      expandAction="click"
      blockNode
      draggable={{ icon: false, nodeDraggable: () => true }}
      allowDrop={() => true}
      onDrop={async (info) => {
        try {
          const dragKey = parseInt(info.dragNode.key as string);
          const dropKey = parseInt(info.node.key as string);
          if (!Number.isFinite(dragKey) || !Number.isFinite(dropKey)) {
            message.error('分组移动失败');
            return;
          }
          const plan = buildDropPlan(groupTree as any, dragKey, dropKey, info as any);
          const result = await window.electronAPI.reorderNoteGroup({
            dragId: dragKey,
            newParentId: plan.newParentId,
            insertIndex: plan.insertIndex,
          });
          if (!result.success) {
            message.error(result.error || '分组移动失败');
            return;
          }
          const [tree, list] = await Promise.all([
            window.electronAPI.getNoteGroupTree(),
            window.electronAPI.getNoteGroups(),
          ]);
          setGroupTree(tree || []);
          setGroups(list || []);
          message.success('分组已移动');
        } catch (error) {
          message.error((error as Error)?.message || '分组移动失败');
        }
      }}
    />
  );
};

export default NoteGroupTree;
