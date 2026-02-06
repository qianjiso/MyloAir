import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useNotes } from '../hooks/useNotes';
import * as notesService from '../services/notes';
import { Button, Table, Modal, Form, Input, Select, Space, Tag, message } from 'antd';
import type { SecureRecord, SecureRecordGroup } from '../../shared/types';
import type { InputRef } from 'antd';

type NoteGroup = SecureRecordGroup;
type NoteRecord = SecureRecord;

const NoteManager: React.FC<{ onClose: () => void; selectedGroupId?: number; externalGroups?: NoteGroup[]; hideTopFilter?: boolean; createSignal?: number; openNoteId?: number; openSignal?: number; createTemplate?: string; templateSignal?: number }> = ({ onClose: _onClose, selectedGroupId: selectedGroupIdProp, externalGroups, hideTopFilter, createSignal, openNoteId, openSignal, createTemplate, templateSignal }) => {
  const { noteGroups, loadNoteGroups } = useNotes();
  const groups: NoteGroup[] = externalGroups ?? (noteGroups as any);
  const [notes, setNotes] = useState<NoteRecord[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<number | undefined>(selectedGroupIdProp);
  const [loading, setLoading] = useState(false);
  const [editVisible, setEditVisible] = useState(false);
  const [editingNote, setEditingNote] = useState<NoteRecord | null>(null);
  const [form] = Form.useForm();
  const [viewVisible, setViewVisible] = useState(false);
  const [viewText, setViewText] = useState('');
  const [selectedLineSet, setSelectedLineSet] = useState<Set<number>>(new Set());
  const titleInputRef = useRef<InputRef>(null);

  const loadGroups = useCallback(async () => {
    try {
      if (externalGroups && externalGroups.length >= 0) {
        return;
      }
      await loadNoteGroups();
    } catch (e) {
      message.error('加载分组失败');
    }
  }, [externalGroups, loadNoteGroups]);

  const loadNotes = useCallback(async (groupId?: number) => {
    setLoading(true);
    try {
      const res = await notesService.listNotes(groupId);
      setNotes(res || []);
    } catch (e) {
      message.error('加载便笺失败');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadGroups();
    loadNotes(selectedGroupIdProp);
  }, [loadGroups, loadNotes, selectedGroupIdProp]);

  useEffect(() => {
    if (typeof selectedGroupIdProp !== 'undefined') {
      setSelectedGroupId(selectedGroupIdProp);
      loadNotes(selectedGroupIdProp);
    }
  }, [selectedGroupIdProp, loadNotes]);

  

  useEffect(() => {
    if (createSignal) {
      handleAdd();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [createSignal]);

  useEffect(() => {
    if (typeof templateSignal !== 'undefined' && typeof createTemplate === 'string') {
      setEditingNote(null);
      setEditVisible(true);
      form.setFieldsValue({ title: '', content_ciphertext: createTemplate, group_id: selectedGroupId || undefined });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [templateSignal]);

  useEffect(() => {
    if (typeof openSignal !== 'undefined' && typeof openNoteId === 'number') {
      (async () => {
        try {
          const note = await notesService.getNote(openNoteId);
          if (note) {
            setEditingNote(note);
            setEditVisible(true);
            form.setFieldsValue({ title: note.title || '', content_ciphertext: note.content_ciphertext || '', group_id: note.group_id || undefined });
          }
        } catch (e) {
          message.error('加载便笺失败');
        }
      })();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openSignal]);

  useEffect(() => {
    if (editVisible) {
      setTimeout(() => {
        titleInputRef.current?.focus();
      }, 0);
    }
  }, [editVisible]);

  const handleAdd = () => {
    setEditingNote(null);
    form.resetFields();
    form.setFieldsValue({ group_id: selectedGroupId || undefined });
    setEditVisible(true);
  };

  const handleEdit = (record: NoteRecord) => {
    setEditingNote(record);
    form.setFieldsValue({ title: record.title || '', content_ciphertext: record.content_ciphertext, group_id: record.group_id || undefined });
    setEditVisible(true);
  };

  const handleDelete = async (id?: number) => {
    if (!id) return;
    try {
      const res = await notesService.removeNote(id);
      if (res.success) {
        message.success('删除成功');
        loadNotes(selectedGroupId);
      } else {
        message.error((res as any).error || '删除失败');
      }
    } catch (e) {
      message.error('删除失败');
    }
  };

  const handleSubmit = async (values: any) => {
    try {
      if (editingNote && editingNote.id) {
        const res = await notesService.updateNote(editingNote.id, values);
        if (res.success) message.success('更新成功'); else message.error(res.error || '更新失败');
      } else {
        const res = await notesService.createNote(values);
        if (res.success) message.success('添加成功'); else message.error(res.error || '添加失败');
      }
      setEditVisible(false);
      loadNotes(selectedGroupId);
    } catch (e) {
      message.error(editingNote ? '更新失败' : '添加失败');
    }
  };

  const columns = [
    { title: '标题', dataIndex: 'title', key: 'title' },
    { title: '分组', dataIndex: 'group_id', key: 'group_id', render: (gid: number) => {
      const g = groups.find(x => x.id === gid);
      return g ? <Tag color={g.color || '蓝色'}>{g.name}</Tag> : '-';
    } },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at', render: (t: string) => {
      const fmt = (s?: string) => {
        if (!s) return '-';
        const d = new Date(s);
        const y = d.getFullYear();
        const m = String(d.getMonth() + 1).padStart(2, '0');
        const day = String(d.getDate()).padStart(2, '0');
        const hh = String(d.getHours()).padStart(2, '0');
        const mm = String(d.getMinutes()).padStart(2, '0');
        return `${y}-${m}-${day} ${hh}:${mm}`;
      };
      return <span title={t}>{fmt(t)}</span>;
    } },
    { title: '更新时间', dataIndex: 'updated_at', key: 'updated_at', render: (t: string) => {
      const fmt = (s?: string) => {
        if (!s) return '-';
        const d = new Date(s);
        const y = d.getFullYear();
        const m = String(d.getMonth() + 1).padStart(2, '0');
        const day = String(d.getDate()).padStart(2, '0');
        const hh = String(d.getHours()).padStart(2, '0');
        const mm = String(d.getMinutes()).padStart(2, '0');
        return `${y}-${m}-${day} ${hh}:${mm}`;
      };
      return <span title={t}>{fmt(t)}</span>;
    } },
    { title: '操作', key: 'action', render: (_: any, record: NoteRecord) => (
      <Space>
        <Button type="link" onClick={() => {
          setViewText(record.content_ciphertext || '');
          setSelectedLineSet(new Set());
          setViewVisible(true);
        }}>查看</Button>
        <Button type="link" onClick={() => handleEdit(record)}>编辑</Button>
        <Button type="link" danger onClick={() => handleDelete(record.id)}>删除</Button>
      </Space>
    ) }
  ];

  return (
    <div>
      {!hideTopFilter && (
        <Space style={{ marginBottom: 12 }}>
          <Select placeholder="选择分组" allowClear style={{ width: 240 }} value={selectedGroupId} onChange={(v) => { setSelectedGroupId(v); loadNotes(v); }}>
            {groups.map(g => <Select.Option key={g.id} value={g.id!}>{g.name}</Select.Option>)}
          </Select>
          <Button type="primary" onClick={handleAdd}>新建便笺</Button>
        </Space>
      )}
      <Table columns={columns as any} dataSource={notes} rowKey="id" loading={loading} pagination={{ pageSize: 10 }} />

      <Modal title={editingNote ? '编辑便笺' : '新建便笺'} open={editVisible} onCancel={() => setEditVisible(false)} footer={null} width={900}>
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item name="title" label="标题">
            <Input placeholder="标题" ref={titleInputRef} />
          </Form.Item>
          <Form.Item name="content_ciphertext" label="正文" rules={[{ required: true, message: '请输入正文' }]}> 
            <Input.TextArea placeholder="自由文本" autoSize={{ minRows: 12, maxRows: 32 }} />
          </Form.Item>
          <Form.Item name="group_id" label="分组" rules={[{ required: true, message: '请选择分组' }]}> 
            <Select allowClear placeholder="选择分组">
              {groups.map(g => <Select.Option key={g.id} value={g.id!}>{g.name}</Select.Option>)}
            </Select>
          </Form.Item>
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">保存</Button>
              <Button onClick={() => setEditVisible(false)}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
      <Modal
        title="查看便笺"
        open={viewVisible}
        onCancel={() => setViewVisible(false)}
        footer={null}
        width={900}
      >
        <div style={{ marginBottom: 12, display: 'flex', gap: 8 }}>
          <Button onClick={async () => { await navigator.clipboard.writeText(viewText || ''); message.success('已复制全部'); }}>复制全部</Button>
          <Button type="primary" onClick={async () => {
            const lines = (viewText || '').split(/\r?\n/);
            const selected = Array.from(selectedLineSet).sort((a,b)=>a-b).map(i => lines[i] ?? '').join('\n');
            await navigator.clipboard.writeText(selected);
            message.success('已复制所选行');
          }} disabled={selectedLineSet.size===0}>复制所选行</Button>
        </div>
        <div style={{ maxHeight: 520, overflow: 'auto', border: '1px solid #f0f0f0', borderRadius: 6 }}>
          {(viewText || '').split(/\r?\n/).map((line, idx) => {
            const selected = selectedLineSet.has(idx);
            return (
              <div
                key={idx}
                onClick={() => {
                  const s = new Set(selectedLineSet);
                  if (s.has(idx)) s.delete(idx); else s.add(idx);
                  setSelectedLineSet(s);
                }}
                style={{
                  display: 'flex',
                  gap: 12,
                  padding: '6px 12px',
                  background: selected ? '#e6f4ff' : '#fff',
                  cursor: 'pointer',
                  borderBottom: '1px solid #f5f5f5',
                  fontFamily: 'monospace'
                }}
              >
                <span style={{ width: 40, color: '#999' }}>{String(idx+1).padStart(2,' ')}</span>
                <span style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{line}</span>
              </div>
            );
          })}
        </div>
      </Modal>
    </div>
  );
};

export default NoteManager;
