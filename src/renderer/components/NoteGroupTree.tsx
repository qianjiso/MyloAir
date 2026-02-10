import React from 'react';
import { Tree, Dropdown, Menu, Button, Popconfirm, message } from 'antd';
import type { DataNode } from 'antd/es/tree';
import { EditOutlined, DeleteOutlined } from '@ant-design/icons';

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
  groups: Array<{
    id?: number;
    name: string;
    parent_id?: number | null;
    color?: string;
    sort_order?: number;
    children?: any[];
  }>;
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
  groups,
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
      overlay={
        <Menu
          items={[
            {
              key: 'rename',
              label: '重命名',
              onClick: () => onEditGroup(group),
            },
            {
              key: 'new-child',
              label: '新建子分组',
              onClick: () => {
                /* 在外部通过弹窗完成 */ onEditGroup({
                ...group,
                parent_id: group.id,
              });
              },
            },
            {
              key: 'move',
              label: '移动到分组',
              onClick: () => {
                onEditGroup(group);
              },
            },
            {
              key: 'delete',
              danger: true,
              label: '删除',
              onClick: () => onDeleteGroup(group.id!),
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
      draggable
      onDrop={async (info) => {
        console.log('[NoteGroupTree] ========== 拖拽开始 ==========');
        console.log('[NoteGroupTree] info:', {
          dragNode: { key: info.dragNode.key },
          node: { key: info.node.key, pos: info.node.pos },
          dropToGap: info.dropToGap,
          dropPosition: info.dropPosition,
        });

        try {
          const dragKey = parseInt(info.dragNode.key as string);
          const dropKey = parseInt(info.node.key as string);
          const dropToGap = info.dropToGap;

          console.log('[NoteGroupTree] dragKey:', dragKey, 'dropKey:', dropKey);
          console.log(
            '[NoteGroupTree] groups 数据:',
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
            '[NoteGroupTree] dropToGap:',
            dropToGap,
            'newParentId:',
            newParentId
          );
          console.log(
            '[NoteGroupTree] groupTree:',
            JSON.stringify(groupTree, null, 2)
          );

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

          const siblings = getSiblings(groupTree as any, newParentId);
          console.log(
            '[NoteGroupTree] siblings under parent',
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
            '[NoteGroupTree] targetIndex:',
            targetIndex,
            'insertIndex:',
            insertIndex
          );

          const moved = siblings.filter(
            (s: any) => (s.id as number) !== dragKey
          );
          const newOrder = [
            ...moved.slice(0, insertIndex),
            { id: dragKey },
            ...moved.slice(insertIndex),
          ];

          console.log(
            '[NoteGroupTree] newOrder:',
            newOrder.map((o) => o.id)
          );

          const findLocalNoteGroup = (id: number) =>
            groups.find((g) => g.id === id);
          const dragNoteGroup = findLocalNoteGroup(dragKey);

          console.log('[NoteGroupTree] dragNoteGroup:', dragNoteGroup);
          console.log('[NoteGroupTree] 更新被拖动的分组:', dragKey, {
            name: dragNoteGroup?.name || '',
            color: dragNoteGroup?.color,
            parent_id: newParentId,
            sort_order: insertIndex,
          });

          // 显式构建对象，同时发送 sort 和 sort_order 字段
          const dragNoteGroupUpdate: any = {
            name: dragNoteGroup?.name || '',
            parent_id: newParentId,
            sort: Number(insertIndex), // 确保是数字
            sort_order: Number(insertIndex), // 同时发送两种字段名
          };
          if (dragNoteGroup?.color) {
            dragNoteGroupUpdate.color = dragNoteGroup.color;
          }
          console.log(
            '[NoteGroupTree] 发送 updateNoteGroup 请求:',
            dragKey,
            dragNoteGroupUpdate
          );

          const result = await window.electronAPI.updateNoteGroup(
            dragKey,
            dragNoteGroupUpdate
          );

          console.log('[NoteGroupTree] update dragGroup result:', result);

          if (!result.success) {
            message.error(result.error || '分组移动失败');
            return;
          }

          // 批量更新同级分组的排序（只更新真正变化的分组）
          console.log('[NoteGroupTree] 开始批量更新排序...');

          // 计算需要更新的分组
          const updates = newOrder
            .map((item, index) => {
              const group = findLocalNoteGroup(item.id as number);
              return {
                id: item.id as number,
                group,
                newSortOrder: index,
                oldSortOrder: group?.sort_order,
              };
            })
            .filter(item => {
              // 只更新 sort_order 真正变化的分组（排除已经更新过的 dragKey）
              return item.id !== dragKey && item.oldSortOrder !== item.newSortOrder;
            });

          console.log('[NoteGroupTree] 需要更新的分组:', updates.map(u => ({
            id: u.id,
            name: u.group?.name,
            oldSort: u.oldSortOrder,
            newSort: u.newSortOrder
          })));

          // 并发更新所有需要更新的分组
          if (updates.length > 0) {
            await Promise.all(
              updates.map(item =>
                window.electronAPI.updateNoteGroup(item.id, {
                  name: item.group?.name || '',
                  parent_id: item.group?.parent_id,
                  color: item.group?.color,
                  sort_order: item.newSortOrder,
                })
              )
            );
            console.log('[NoteGroupTree] 批量更新完成');
          } else {
            console.log('[NoteGroupTree] 无需更新其他分组');
          }

          console.log('[NoteGroupTree] 刷新分组树...');
          const tree = await window.electronAPI.getNoteGroupTree();
          const list = await window.electronAPI.getNoteGroups();
          console.log('[NoteGroupTree] 新分组树:', tree);
          console.log('[NoteGroupTree] 新分组列表:', list);
          setGroupTree(tree || []);
          setGroups(list || []);
          message.success('分组已移动');
        } catch (error) {
          console.error('[NoteGroupTree] 拖拽失败:', error);
          message.error('分组移动失败');
        }
      }}
    />
  );
};

export default NoteGroupTree;
