import { useState, useCallback } from 'react';
import { message } from 'antd';
import * as notesService from '../services/notes';

export function useNotes() {
  const [noteGroups, setNoteGroups] = useState<any[]>([]);
  const [noteGroupTree, setNoteGroupTree] = useState<any[]>([]);

  const loadNoteGroups = useCallback(async () => {
    try {
      const list = await notesService.listNoteGroups();
      const tree = await notesService.getNoteGroupTree();
      setNoteGroups(list || []);
      setNoteGroupTree(tree || []);
    } catch (error) {
      message.error('加载便笺分组失败');
    }
  }, []);

  const createNoteGroup = useCallback(async (payload: any) => {
    return notesService.createNoteGroup(payload);
  }, []);

  const updateNoteGroup = useCallback(async (id: number, payload: any) => {
    return notesService.updateNoteGroup(id, payload);
  }, []);

  const removeNoteGroup = useCallback(async (id: number) => {
    return notesService.removeNoteGroup(id);
  }, []);

  return {
    noteGroups,
    noteGroupTree,
    loadNoteGroups,
    createNoteGroup,
    updateNoteGroup,
    removeNoteGroup,
    setNoteGroups,
    setNoteGroupTree,
  };
}

