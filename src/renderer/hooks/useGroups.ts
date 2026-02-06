import { useState, useCallback } from 'react';
import { message } from 'antd';
import * as groupService from '../services/groups';
import type { Group, GroupWithChildren } from '../../shared/types';
import { reportError } from '../utils/logging';

export function useGroups() {
  const [groups, setGroups] = useState<Group[]>([]);
  const [groupTree, setGroupTree] = useState<GroupWithChildren[]>([]);

  const loadGroups = useCallback(async () => {
    try {
      const list = await groupService.listGroups();
      const tree = await groupService.getGroupTree();
      setGroups(list || []);
      setGroupTree(tree || []);
    } catch (error) {
      message.error('加载分组失败');
      reportError('GROUPS_LOAD_FAILED', 'Load groups error', error);
    }
  }, []);

  const createGroup = useCallback(async (payload: Group) => {
    return groupService.createGroup(payload);
  }, []);

  const updateGroup = useCallback(async (id: number, payload: Group) => {
    return groupService.updateGroup(id, payload);
  }, []);

  const removeGroup = useCallback(async (id: number) => {
    return groupService.removeGroup(id);
  }, []);

  return {
    groups,
    groupTree,
    loadGroups,
    createGroup,
    updateGroup,
    removeGroup,
    setGroups,
    setGroupTree,
  };
}
