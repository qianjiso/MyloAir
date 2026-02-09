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
  geekblue: '#2f54eb',
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
      trigger={['contextMenu']}
      overlay={
        <Menu
          items={[
            {
              key: 'rename',
              label: '重命名',
              onClick: () => onEditGroup(group),
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
          ]}
        />
      }
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
                onDeleteGroup(group.id!);
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

  const buildGroupNodes = (groupsTree: GroupWithChildren[]): DataNode[] =>
    groupsTree.map((group) => ({
      key: group.id?.toString() || `temp-${group.name}`,
      title: renderGroupTitle(group as Group),
      children:
        group.children && group.children.length > 0
          ? buildGroupNodes(group.children)
          : undefined,
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
        console.log('[GroupTree] ========== 拖拽开始 ==========');
        console.log('[GroupTree] info:', {
          dragNode: { key: info.dragNode.key },
          node: { key: info.node.key, pos: info.node.pos },
          dropToGap: info.dropToGap,
          dropPosition: info.dropPosition,
        });
        try {
          const dragKey = parseInt(info.dragNode.key as string);
          const dropKey = parseInt(info.node.key as string);
          const dropToGap = info.dropToGap;

          console.log('[GroupTree] dragKey:', dragKey, 'dropKey:', dropKey);
          console.log(
            '[GroupTree] groups 数据:',
            groups.map((g) => ({
              id: g.id,
              name: g.name,
              parent_id: g.parent_id,
              sort_order: g.sort_order,
            }))
          );

          const findParent = (
            tree: any[],
            childId: number,
            parentId: number | null = null
          ): number | null => {
            for (const node of tree) {
              if (node.id === childId) return parentId;
              const p = findParent(
                node.children || [],
                childId,
                node.id ?? null
              );
              if (p !== null) return p;
            }
            return null;
          };

          const newParentId = dropToGap
            ? findParent(groupTree as any, dropKey)
            : dropKey;

          console.log(
            '[GroupTree] dropToGap:',
            dropToGap,
            'newParentId:',
            newParentId
          );
          console.log(
            '[GroupTree] groupTree:',
            JSON.stringify(groupTree, null, 2)
          );

          // Correctly find siblings by locating the parent node in the tree
          const getSiblings = (tree: any[], pid: number | null) => {
             if (pid === null) {
                 return tree.filter(n => (n.parent_id ?? null) === null);
             }
             const findNode = (nodes: any[], targetId: number): any => {
                 for (const n of nodes) {
                     if (n.id === targetId) return n;
                     if (n.children) {
                         const found = findNode(n.children, targetId);
                         if (found) return found;
                     }
                 }
                 return null;
             };
             const parentNode = findNode(tree, pid);
             return parentNode ? [...(parentNode.children || [])] : [];
          };

          const siblings = getSiblings(groupTree, newParentId);
          console.log(
            '[GroupTree] siblings under parent',
            newParentId,
            ':',
            siblings.map((s) => ({ id: s.id, name: s.name }))
          );

          const targetIndex = siblings.findIndex(
            (s: any) => (s.id as number) === dropKey
          );
          const insertIndex = dropToGap
            ? info.dropPosition < 0
              ? targetIndex
              : targetIndex + 1
            : siblings.length;

          console.log(
            '[GroupTree] targetIndex:',
            targetIndex,
            'insertIndex:',
            insertIndex
          );

          // 创建新的顺序数组
          const moved = siblings.filter(
            (s: any) => (s.id as number) !== dragKey
          );
          const newOrder = [
            ...moved.slice(0, insertIndex),
            { id: dragKey },
            ...moved.slice(insertIndex),
          ];

          console.log(
            '[GroupTree] newOrder:',
            newOrder.map((o) => o.id)
          );

          const findLocalGroup = (id: number) =>
            groups.find((g) => g.id === id);

          const dragGroup = findLocalGroup(dragKey);
          console.log('[GroupTree] dragGroup:', dragGroup);

          // 先更新被拖动的分组（更新 parent_id 和 sort_order）
          console.log('[GroupTree] 更新被拖动的分组:', dragKey, {
            name: dragGroup?.name || '',
            color: dragGroup?.color,
            parent_id: newParentId ?? undefined,
            sort_order: insertIndex,
          });

          // 显式构建 Group 对象，确保 sort 和 sort_order 字段都存在且为数字
          const dragGroupUpdate: any = {
            name: dragGroup?.name || '',
            parent_id: newParentId ?? undefined,
            sort: Number(insertIndex), // 确保是数字
            sort_order: Number(insertIndex), // 同时发送两种字段名
          };
          if (dragGroup?.color) {
            dragGroupUpdate.color = dragGroup.color;
          }
          console.log(
            '[GroupTree] 发送 updateGroup 请求:',
            dragKey,
            JSON.stringify(dragGroupUpdate, null, 2)
          );

          const result = await window.electronAPI.updateGroup(
            dragKey,
            dragGroupUpdate
          );

          console.log('[GroupTree] update dragGroup result:', result);

          if (!result.success) {
            message.error((result as any).error || '分组移动失败');
            return;
          }

          // 更新同级其他分组的排序
          console.log('[GroupTree] 开始更新同级分组排序...');
          for (let i = 0; i < newOrder.length; i++) {
            const id = (newOrder[i] as any).id as number;
            if (id === dragKey) continue; // 跳过已更新的拖动分组

            const g = findLocalGroup(id);
            const originalSortOrder = g?.sort_order;

            console.log(
              `[GroupTree] 更新分组 ${id} (${g?.name}): sort_order ${originalSortOrder} -> ${i}`
            );

            // 显式构建 Group 对象，同时发送 sort 和 sort_order
            const siblingUpdate: any = {
              name: g?.name || '',
              sort: Number(i), // 确保是数字
              sort_order: Number(i), // 同时发送两种字段名
              parent_id: g?.parent_id,
            };
            if (g?.color) {
              siblingUpdate.color = g.color;
            }
            console.log(
              `[GroupTree] 发送 updateGroup 请求:`,
              id,
              siblingUpdate
            );

            const updateResult = await window.electronAPI.updateGroup(
              id,
              siblingUpdate
            );

            console.log(`[GroupTree] 更新分组 ${id} 结果:`, updateResult);
          }

          console.log('[GroupTree] 刷新分组树...');
          const tree = await window.electronAPI.getGroupTree();
          console.log('[GroupTree] 新分组树:', tree);
          setGroupTree(tree || []);
          message.success('分组已移动');
        } catch (error) {
          console.error('[GroupTree] 拖拽失败:', error);
          message.error('分组移动失败');
        }
      }}
    />
  );
};

export default GroupTree;
