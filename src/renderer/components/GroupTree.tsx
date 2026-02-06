import React from 'react';
import { Tree, Dropdown, Menu, Button, Popconfirm, message } from 'antd';
import type { DataNode } from 'antd/es/tree';
import { EditOutlined, DeleteOutlined } from '@ant-design/icons';
import type { Group, GroupWithChildren } from '../../shared/types';

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
  geekblue: '#2f54eb'
};

const getGroupColor = (color?: string) => {
  if (!color) return '#1677ff';
  return groupColorMap[color] || color;
};

export interface GroupTreeProps {
  groups: Group[];
  groupTree: GroupWithChildren[];
  selectedGroupId?: number;
  expandedKeys: string[];
  onExpanded: (keys: string[]) => void;
  onSelect: (selectedKeys: React.Key[], info: any) => void;
  setGroupTree: (tree: GroupWithChildren[]) => void;
  onEditGroup: (group: Group) => void;
  onDeleteGroup: (id: number) => void;
  treeKey?: number;
}

/**
 * 分组树组件（密码分组）
 * 负责渲染树形结构与拖拽排序，并提供右键菜单操作
 */
const GroupTree: React.FC<GroupTreeProps> = ({
  groups,
  groupTree,
  selectedGroupId,
  expandedKeys,
  onExpanded,
  onSelect,
  setGroupTree,
  onEditGroup,
  onDeleteGroup,
  treeKey,
}) => {
  const renderGroupTitle = (group: Group) => (
    <Dropdown
      trigger={["contextMenu"]}
      overlay={(
        <Menu
          items={[
            { key: 'rename', label: '重命名', onClick: () => onEditGroup(group) },
            { key: 'move', label: '移动到分组', onClick: () => onEditGroup(group) },
            { key: 'delete', danger: true, label: '删除', onClick: () => group.id && onDeleteGroup(group.id) }
          ]}
        />
      )}
    >
      <div className="group-tree-node">
        <div className="group-tree-node__info">
          <span className="group-color-dot" style={{ backgroundColor: getGroupColor(group.color) }} />
          <span className="group-tree-node__name">{group.name}</span>
        </div>
        <div className="group-tree-node__actions">
          <Button type="text" size="small" icon={<EditOutlined />} onClick={(e) => { e.stopPropagation(); onEditGroup(group); }} />
          {group.id && (
            <Popconfirm title="确定要删除这个分组吗？" onConfirm={(e) => { e?.stopPropagation(); onDeleteGroup(group.id!); }} okText="确定" cancelText="取消">
              <Button type="text" size="small" danger icon={<DeleteOutlined />} onClick={(e) => e.stopPropagation()} />
            </Popconfirm>
          )}
        </div>
      </div>
    </Dropdown>
  );

  const buildGroupNodes = (groupsTree: GroupWithChildren[]): DataNode[] =>
    groupsTree.map(group => ({
      key: group.id?.toString() || `temp-${group.name}`,
      title: renderGroupTitle(group as Group),
      children: group.children && group.children.length > 0 ? buildGroupNodes(group.children) : undefined,
    }));

  const treeData: DataNode[] = buildGroupNodes(groupTree);

  return (
    <Tree
      key={treeKey}
      showLine
      treeData={treeData}
      onSelect={onSelect}
      selectedKeys={selectedGroupId ? [selectedGroupId.toString()] : []}
      style={{ background: 'transparent' }}
      defaultExpandAll={true}
      expandAction="click"
      expandedKeys={expandedKeys}
      onExpand={(keys) => onExpanded(keys as string[])}
      draggable
      onDrop={async (info) => {
        try {
          const dragKey = parseInt((info.dragNode.key as string));
          const dropKey = parseInt((info.node.key as string));
          const dropToGap = info.dropToGap;
          const findParent = (tree: any[], childId: number, parentId: number | null = null): number | null => {
            for (const node of tree) {
              if (node.id === childId) return parentId;
              const p = findParent(node.children || [], childId, node.id ?? null);
              if (p !== null) return p;
            }
            return null;
          };
          const newParentId = dropToGap ? findParent(groupTree as any, dropKey) : dropKey;
          const collectSiblings = (tree: any[], parentId: number | null) => {
            const res: any[] = [];
            const walk = (nodes: any[], pid: number | null) => {
              for (const n of nodes) {
                const p = n.parent_id ?? null;
                if (p === pid) res.push(n);
                if (n.children && n.children.length) walk(n.children, n.id ?? null);
              }
            };
            walk(tree as any, parentId);
            return res;
          };
          const siblings = collectSiblings(groupTree as any, newParentId);
          const targetIndex = siblings.findIndex((s: any) => (s.id as number) === dropKey);
          const insertIndex = dropToGap ? (info.dropPosition < 0 ? targetIndex : targetIndex + 1) : siblings.length;
          const moved = siblings.filter((s: any) => (s.id as number) !== dragKey);
          const newOrder = [ ...moved.slice(0, insertIndex), { id: dragKey }, ...moved.slice(insertIndex) ];
          const parentIdToSend = newParentId ?? undefined;
          const findLocalGroup = (id: number) => groups.find(g => g.id === id);
          const dragGroup = findLocalGroup(dragKey);
          const result = await window.electronAPI.updateGroup(dragKey, { name: dragGroup?.name || '', color: dragGroup?.color, parent_id: parentIdToSend, sort: insertIndex } as any);
          if (!result.success) { message.error((result as any).error || '分组移动失败'); return; }
          for (let i = 0; i < newOrder.length; i++) {
            const id = (newOrder[i] as any).id as number;
            const g = findLocalGroup(id);
            await window.electronAPI.updateGroup(id, { name: g?.name || '', color: g?.color, sort: i, parent_id: parentIdToSend } as any);
          }
          const tree = await window.electronAPI.getGroupTree();
          setGroupTree(tree || []);
          message.success('分组已移动');
        } catch {
          message.error('分组移动失败');
        }
      }}
    />
  );
};

export default GroupTree;
