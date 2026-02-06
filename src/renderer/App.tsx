import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Layout, Button, Table, Modal, Form, Input, message, Space, Select, Tabs, Segmented } from 'antd';
import { PlusOutlined, SettingOutlined, FolderOutlined, FolderAddOutlined, DownloadOutlined, KeyOutlined } from '@ant-design/icons';
const PasswordGenerator = React.lazy(() => import('./components/PasswordGenerator'));
import PasswordDetailModal from './components/PasswordDetailModal';
const UserSettings = React.lazy(() => import('./components/UserSettings'));
const ImportExportModal = React.lazy(() => import('./components/ImportExportModal'));
const NoteManager = React.lazy(() => import('./components/NoteManager'));
import GroupTree from './components/GroupTree';
import NoteGroupTree from './components/NoteGroupTree';
import './styles/global.css';
import { buildPasswordColumns } from './columns/passwordColumns';
import { buildHistoryColumns } from './columns/historyColumns';
import MasterPasswordGate from './components/MasterPasswordGate';

// 从preload导入类型
import type { Group, MasterPasswordState } from '../shared/types';
import { usePasswords } from './hooks/usePasswords';
import { useGroups } from './hooks/useGroups';
import { useNotes } from './hooks/useNotes';
import * as securityService from './services/security';
import { reportError } from './utils/logging';

// 在浏览器环境中导入mock
if (typeof window !== 'undefined' && !window.electronAPI) {
  import('./electronAPI-mock');
}

const { Header, Content, Sider } = Layout;
const { Option } = Select;

// 颜色映射已迁移到分组树组件

interface Password {
  id: number;
  title: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
  group_id?: number;
  created_at?: string;
  updated_at?: string;
}

const App: React.FC = () => {
  const {
    passwords,
    passwordHistory,
    loading,
    loadPasswords,
    loadRecentPasswords,
    loadPasswordHistory,
    createPassword,
    updatePassword,
    removePassword,
  } = usePasswords();
  const {
    groups,
    groupTree,
    loadGroups,
    createGroup,
    updateGroup,
    removeGroup,
    setGroupTree,
  } = useGroups();
  const [selectedGroupId, setSelectedGroupId] = useState<number | undefined>();
  
  const [modalVisible, setModalVisible] = useState(false);
  const [groupModalVisible, setGroupModalVisible] = useState(false);
  const [historyModalVisible, setHistoryModalVisible] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [importExportVisible, setImportExportVisible] = useState(false);
  const [currentModule, setCurrentModule] = useState<'password' | 'notes'>('password');
  const {
    noteGroups,
    noteGroupTree,
    loadNoteGroups,
    createNoteGroup,
    updateNoteGroup,
    removeNoteGroup,
    setNoteGroups,
    setNoteGroupTree,
  } = useNotes();
  const [selectedNoteGroupId, setSelectedNoteGroupId] = useState<number | undefined>();
  const [noteGroupModalVisible, setNoteGroupModalVisible] = useState(false);
  const [editingNoteGroup, setEditingNoteGroup] = useState<any | null>(null);
  const [noteGroupForm] = Form.useForm();
  const [noteCreateSignal, setNoteCreateSignal] = useState(0);
  const [cmdPaletteVisible, setCmdPaletteVisible] = useState(false);
  const [globalSearchVisible, setGlobalSearchVisible] = useState(false);
  const [globalSearchPasswords, setGlobalSearchPasswords] = useState<any[]>([]);
  const [globalSearchNotes, setGlobalSearchNotes] = useState<any[]>([]);
  const [noteOpenId, setNoteOpenId] = useState<number | undefined>(undefined);
  const [noteOpenSignal, setNoteOpenSignal] = useState(0);
  const [globalSearchActiveTab, setGlobalSearchActiveTab] = useState<'pw' | 'nt'>('pw');
  const [selectedPwIndex, setSelectedPwIndex] = useState(0);
  const [selectedNoteIndex, setSelectedNoteIndex] = useState(0);
  
  const [editingPassword, setEditingPassword] = useState<Password | null>(null);
  const [editingGroup, setEditingGroup] = useState<Group | null>(null);
  const [generatorVisible, setGeneratorVisible] = useState(false);
  const [passwordDetailMode, setPasswordDetailMode] = useState<'view' | 'edit' | 'create'>('view');
  const [visiblePasswords, setVisiblePasswords] = useState<Set<string>>(new Set());
  const [visibleHistoryPasswords, setVisibleHistoryPasswords] = useState<Set<string>>(new Set());
  const [form] = Form.useForm();
  const [treeKey, setTreeKey] = useState(0);
  const [groupForm] = Form.useForm();
  const [expandedKeys, setExpandedKeys] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [securityState, setSecurityState] = useState<MasterPasswordState | null>(null);
  const [locked, setLocked] = useState(true);
  const [checkingSecurity, setCheckingSecurity] = useState(true);
  const [securityLoading, setSecurityLoading] = useState(false);
  const lockTimerRef = useRef<NodeJS.Timeout | null>(null);
  const settingsVisibleRef = useRef(settingsVisible);

  useEffect(() => {
    const initSecurity = async () => {
      try {
        setCheckingSecurity(true);
        const state = await securityService.getSecurityState();
        setSecurityState(state);
        const shouldLock = state.requireMasterPassword;
        setLocked(shouldLock);
        if (!shouldLock) {
          await Promise.all([
            loadGroups(),
            loadRecentPasswords(),
          ]);
        }
      } catch (err) {
        reportError('APP_LOAD_SECURITY_STATE_FAILED', '加载安全状态失败', err);
        setLocked(false);
        await Promise.all([
          loadGroups(),
          loadRecentPasswords(),
        ]);
      } finally {
        setCheckingSecurity(false);
      }
    };
    initSecurity();
  }, [loadGroups, loadRecentPasswords]);

  useEffect(() => {
    if (window.electronAPI?.onDataImported) {
      window.electronAPI.onDataImported(async () => {
        if (locked) return;
        await loadGroups();
        if (selectedGroupId) {
          await loadPasswords(selectedGroupId);
        } else {
          await loadRecentPasswords();
        }
      });
    }
  }, [locked, selectedGroupId, loadGroups, loadPasswords, loadRecentPasswords]);

  useEffect(() => {
    if (locked || checkingSecurity) return;
    if (selectedGroupId) {
      loadPasswords(selectedGroupId);
      setSearchQuery('');
    } else if (!searchQuery) {
      loadRecentPasswords();
    }
  }, [selectedGroupId, searchQuery, loadPasswords, loadRecentPasswords, locked, checkingSecurity]);

  const resetAutoLockTimer = useCallback(() => {
    if (lockTimerRef.current) {
      clearTimeout(lockTimerRef.current);
    }
    if (!securityState?.requireMasterPassword || !securityState.hasMasterPassword) return;
    const minutes = Math.max(1, securityState.autoLockMinutes || 5);
    lockTimerRef.current = setTimeout(() => {
      setLocked(true);
    }, minutes * 60 * 1000);
  }, [securityState]);

  useEffect(() => {
    if (locked || !securityState?.requireMasterPassword || !securityState.hasMasterPassword) return;
    const reset = () => resetAutoLockTimer();
    window.addEventListener('mousemove', reset);
    window.addEventListener('keydown', reset);
    window.addEventListener('click', reset);
    reset();
    return () => {
      window.removeEventListener('mousemove', reset);
      window.removeEventListener('keydown', reset);
      window.removeEventListener('click', reset);
      if (lockTimerRef.current) {
        clearTimeout(lockTimerRef.current);
      }
    };
  }, [locked, securityState, resetAutoLockTimer]);

  const handleUnlock = useCallback(async (password: string) => {
    setSecurityLoading(true);
    const res = await securityService.verifyMasterPassword(password);
    setSecurityLoading(false);
    if (!res.success) {
      const msg = res.error || '主密码不正确';
      message.error(msg);
      throw new Error(msg);
    }
    setSecurityState(res.state || securityState);
    setLocked(false);
    resetAutoLockTimer();
    await loadGroups();
    await loadRecentPasswords();
  }, [loadGroups, loadRecentPasswords, resetAutoLockTimer, securityState]);

  const handleSetupMaster = useCallback(async (password: string, hint?: string) => {
    setSecurityLoading(true);
    const res = await securityService.setMasterPassword(password, hint);
    setSecurityLoading(false);
    if (!res.success) {
      const msg = res.error || '设置主密码失败';
      message.error(msg);
      throw new Error(msg);
    }
    setSecurityState(res.state || securityState);
    message.success('主密码已设置并启用');
    setLocked(false);
    resetAutoLockTimer();
    await loadGroups();
    await loadRecentPasswords();
  }, [loadGroups, loadRecentPasswords, resetAutoLockTimer, securityState]);

  useEffect(() => {
    const refreshState = async () => {
      try {
        const state = await securityService.getSecurityState();
        setSecurityState(state);
        const nextLocked = state.requireMasterPassword ? locked : false;
        setLocked(nextLocked);
        if (state.requireMasterPassword && !nextLocked) {
          resetAutoLockTimer();
        }
        if (!state.requireMasterPassword && lockTimerRef.current) {
          clearTimeout(lockTimerRef.current);
        }
      } catch (err) {
        reportError('APP_REFRESH_SECURITY_STATE_FAILED', '刷新安全状态失败', err);
      }
    };
    if (settingsVisibleRef.current && !settingsVisible) {
      refreshState();
    }
    settingsVisibleRef.current = settingsVisible;
  }, [settingsVisible, locked, resetAutoLockTimer]);

  

  const handleAdd = useCallback(() => {
    setEditingPassword(selectedGroupId ? ({ group_id: selectedGroupId } as any) : null);
    setPasswordDetailMode('create');
    setModalVisible(true);
    form.resetFields();
  }, [selectedGroupId, form]);

  const handleEdit = async (record: Password) => {
    try {
      const full = await window.electronAPI.getPassword(record.id);
      const pw = full || record;
      setEditingPassword(pw as any);
      setPasswordDetailMode('edit');
      setModalVisible(true);
      form.setFieldsValue(pw as any);
    } catch {
      setEditingPassword(record);
      setPasswordDetailMode('edit');
      setModalVisible(true);
      form.setFieldsValue(record);
    }
  };

  const handleView = async (record: Password) => {
    try {
      const full = await window.electronAPI.getPassword(record.id);
      const pw = full || record;
      setEditingPassword(pw as any);
    } catch {
      setEditingPassword(record);
    }
    setPasswordDetailMode('view');
    setModalVisible(true);
  };


  const handleDelete = async (id: number) => {
    try {
      const result = await removePassword(id);
      if (result.success) {
        message.success('删除成功');
        loadPasswords(selectedGroupId);
      } else {
        message.error('删除失败');
      }
    } catch (error) {
      message.error('删除失败');
      reportError('APP_DELETE_PASSWORD_FAILED', 'Delete password error', error, { passwordId: id });
    }
  };

  const handleViewHistory = async (record: Password) => {
    if (record.id) {
      await loadPasswordHistory(record.id);
      setHistoryModalVisible(true);
    }
  };

  const handleSubmit = async (values: any) => {
    try {
      if (editingPassword && passwordDetailMode !== 'create') {
        const result = await updatePassword(editingPassword.id, values);
        if (result.success) {
          message.success('更新成功');
        } else {
          message.error((result as any).error || '更新失败');
        }
      } else {
        const result = await createPassword(values);
        if (result.success) {
          message.success('添加成功');
        } else {
          message.error((result as any).error || '添加失败');
        }
      }
      setModalVisible(false);
      loadPasswords(selectedGroupId);
    } catch (error) {
      message.error(passwordDetailMode === 'create' ? '添加失败' : '更新失败');
      reportError('APP_SUBMIT_PASSWORD_FAILED', 'Submit password form failed', error, {
        mode: passwordDetailMode,
        hasEditingPassword: !!editingPassword,
      });
    }
  };

  const handleAddGroup = () => {
    setEditingGroup(null);
    setGroupModalVisible(true);
    groupForm.resetFields();
  };

  const handleEditGroup = (group: Group) => {
    setEditingGroup(group);
    setGroupModalVisible(true);
    groupForm.setFieldsValue(group);
  };

  const handleDeleteGroup = async (id: number) => {
    try {
      const result = await removeGroup(id);
      if (result.success) {
        message.success('删除分组成功');
        loadGroups();
        if (selectedGroupId === id) {
          setSelectedGroupId(undefined);
        }
      } else {
        message.error('删除分组失败');
      }
    } catch (error) {
      message.error('删除分组失败');
      reportError('APP_DELETE_GROUP_FAILED', 'Delete group error', error, { groupId: id });
    }
  };

  

  const handleSubmitGroup = async (values: any) => {
    try {
      if (editingGroup && editingGroup.id) {
        const result = await updateGroup(editingGroup.id, values);
        if (result.success) {
          message.success('更新分组成功');
        } else {
          message.error((result as any).error || '更新分组失败');
        }
      } else {
        const result = await createGroup(values);
        if (result.success) {
          message.success('添加分组成功');
        } else {
          message.error((result as any).error || '添加分组失败');
        }
      }
      setGroupModalVisible(false);
      await loadGroups();
      // 强制刷新Tree组件
      setTreeKey(prev => prev + 1);
    } catch (error: any) {
      message.error(error.message || (editingGroup ? '更新分组失败' : '添加分组失败'));
      reportError('APP_SUBMIT_GROUP_FAILED', 'Submit group form failed', error, {
        hasEditingGroup: !!editingGroup,
      });
    }
  };

  const handleGeneratePassword = (password: string) => {
    form.setFieldsValue({ password });
    setGeneratorVisible(false);
    message.success('密码已生成');
  };

  const togglePasswordVisibility = (passwordId: string) => {
    setVisiblePasswords(prev => {
      const newSet = new Set(prev);
      if (newSet.has(passwordId)) {
        newSet.delete(passwordId);
      } else {
        newSet.add(passwordId);
      }
      return newSet;
    });
  };

  const toggleHistoryPasswordVisibility = (historyId: string) => {
    setVisibleHistoryPasswords(prev => {
      const newSet = new Set(prev);
      if (newSet.has(historyId)) {
        newSet.delete(historyId);
      } else {
        newSet.add(historyId);
      }
      return newSet;
    });
  };

  // 渲染与树数据构建逻辑迁移到独立组件

  const handleGroupSelect = (selectedKeys: React.Key[], info: any) => {
    if (selectedKeys.length > 0) {
      const selectedKey = selectedKeys[0] as string;
      setSelectedGroupId(parseInt(selectedKey));
      const hasChildren = info.node && info.node.children && info.node.children.length > 0;
      if (hasChildren && !expandedKeys.includes(selectedKey)) {
        setExpandedKeys(prev => [...prev, selectedKey]);
      }
    } else {
      setSelectedGroupId(undefined);
    }
  };

  const handleNoteGroupSelect = (selectedKeys: React.Key[]) => {
    if (selectedKeys.length > 0) {
      const selectedKey = selectedKeys[0] as string;
      setSelectedNoteGroupId(parseInt(selectedKey));
    } else {
      setSelectedNoteGroupId(undefined);
    }
  };

  const handleAddNoteGroup = () => {
    setEditingNoteGroup(null);
    setNoteGroupModalVisible(true);
    noteGroupForm.resetFields();
  };

  const handleEditNoteGroup = (group: any) => {
    setEditingNoteGroup(group);
    setNoteGroupModalVisible(true);
    noteGroupForm.setFieldsValue(group);
  };

  const handleDeleteNoteGroup = async (id: number) => {
    try {
      const result = await removeNoteGroup(id);
      if (result.success) {
        message.success('删除便笺分组成功');
        await loadNoteGroups();
      } else {
        message.error((result as any).error || '删除便笺分组失败');
      }
    } catch (error) {
      message.error('删除便笺分组失败');
    }
  };

  const handleSubmitNoteGroup = async (values: any) => {
    try {
      const payload = { name: values.name, parent_id: values.parent_id || null, color: values.color || 'blue' };
      if (editingNoteGroup && editingNoteGroup.id) {
        const result = await updateNoteGroup(editingNoteGroup.id, payload);
        if (result.success) message.success('更新便笺分组成功'); else message.error((result as any).error || '更新便笺分组失败');
      } else {
        const result = await createNoteGroup(payload);
        if (result.success) message.success('添加便笺分组成功'); else message.error((result as any).error || '添加便笺分组失败');
      }
      setNoteGroupModalVisible(false);
      await loadNoteGroups();
    } catch (error: any) {
      message.error(error.message || (editingNoteGroup ? '更新便笺分组失败' : '添加便笺分组失败'));
    }
  };

  // 便笺分组排序功能已由 NoteGroupTree 组件承载

  useEffect(() => {
    if (currentModule === 'notes') {
      loadNoteGroups();
    }
  }, [currentModule, loadNoteGroups]);

  useEffect(() => {
    if (!globalSearchVisible) return;
    const handler = async (e: KeyboardEvent) => {
      const k = e.key;
      if (k === 'ArrowDown') {
        if (globalSearchActiveTab === 'pw') setSelectedPwIndex(i => Math.min(i + 1, Math.max(0, globalSearchPasswords.length - 1)));
        else setSelectedNoteIndex(i => Math.min(i + 1, Math.max(0, globalSearchNotes.length - 1)));
        e.preventDefault();
      } else if (k === 'ArrowUp') {
        if (globalSearchActiveTab === 'pw') setSelectedPwIndex(i => Math.max(i - 1, 0));
        else setSelectedNoteIndex(i => Math.max(i - 1, 0));
        e.preventDefault();
      } else if (k === 'Enter') {
        if (globalSearchActiveTab === 'pw' && globalSearchPasswords.length) {
          const row = globalSearchPasswords[selectedPwIndex];
          setCurrentModule('password');
          try {
            const full = await window.electronAPI.getPassword(row.id);
            setEditingPassword((full || row) as any);
          } catch {
            setEditingPassword(row as any);
          }
          setPasswordDetailMode('view');
          setModalVisible(true);
          setGlobalSearchVisible(false);
        } else if (globalSearchActiveTab === 'nt' && globalSearchNotes.length) {
          const row = globalSearchNotes[selectedNoteIndex];
          setCurrentModule('notes');
          setGlobalSearchVisible(false);
          setNoteOpenId(row.id);
          setNoteOpenSignal(s => s + 1);
        }
        e.preventDefault();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [globalSearchVisible, globalSearchActiveTab, globalSearchPasswords, globalSearchNotes, selectedPwIndex, selectedNoteIndex]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      const k = e.key.toLowerCase();
      if (meta && k === '1') { setCurrentModule('password'); e.preventDefault(); }
      if (meta && k === '2') { setCurrentModule('notes'); e.preventDefault(); }
      if (meta && k === 'f') { const el = document.querySelector('input[placeholder*="搜索"]') as HTMLInputElement; if (el) { el.focus(); el.select(); } e.preventDefault(); }
      if (meta && k === 'n') {
        if (currentModule === 'password') { handleAdd(); } else { setNoteCreateSignal((s) => s + 1); }
        e.preventDefault();
      }
      if (meta && k === 'k') { setCmdPaletteVisible(true); e.preventDefault(); }
      if (e.key === 'Escape') { setCmdPaletteVisible(false); }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [currentModule, handleAdd]);

  const columns = buildPasswordColumns(
    groups as any,
    visiblePasswords,
    togglePasswordVisibility,
    (row) => handleViewHistory(row as any),
    (row) => handleEdit(row as any),
    (id) => handleDelete(id),
    passwords as any
  ) as any;

  const historyColumns = buildHistoryColumns(
    visibleHistoryPasswords,
    (key) => toggleHistoryPasswordVisibility(key)
  ) as any;

  return (
    <>
    <Layout style={{ minHeight: '100vh' }}>
      <Header className="header">
        <div className="app-toolbar">
          <div className="toolbar-left">
            <div className="logo"><KeyOutlined /> Mylo</div>
            <Segmented
              value={currentModule}
              onChange={(v) => setCurrentModule(v as any)}
              options={[
                { label: '密码', value: 'password' },
                { label: '便笺', value: 'notes' }
              ]}
            />
          </div>
          <div className="toolbar-center">
            <Input.Search 
              allowClear 
              placeholder={currentModule === 'password' ? '搜索密码：标题/用户名/URL' : '搜索便笺标题'} 
              className="header-search"
              onSearch={async (value) => {
                try {
                  if (!value || value.trim() === '') { return; }
                  setSearchQuery(value);
                  const [pw, nt] = await Promise.all([
                    window.electronAPI.searchPasswords(value),
                    window.electronAPI.searchNotesTitle(value)
                  ]);
                  setGlobalSearchPasswords(pw || []);
                  setGlobalSearchNotes(nt || []);
                  setSelectedPwIndex(0);
                  setSelectedNoteIndex(0);
                  setGlobalSearchActiveTab(currentModule === 'password' ? 'pw' : 'nt');
                  setGlobalSearchVisible(true);
                } catch { message.error('搜索失败'); }
              }}
            />
          </div>
          <div className="toolbar-right header-actions">
            <Button icon={<DownloadOutlined />} onClick={() => setImportExportVisible(true)}>导入导出</Button>
            <Button icon={<SettingOutlined />} onClick={() => setSettingsVisible(true)}>设置</Button>
          </div>
        </div>
      </Header>
      <Layout>
        <Sider width={250} style={{ background: '#fff', borderRight: '1px solid #f0f0f0' }}>
          <div style={{ padding: '16px' }}>
            <Button icon={<FolderAddOutlined />} onClick={currentModule === 'password' ? handleAddGroup : handleAddNoteGroup} style={{ width: '100%', marginBottom: '16px' }}>新建分组</Button>
            <div style={{ marginBottom: '8px', fontSize: '14px', fontWeight: 'bold', color: '#666' }}>分组列表</div>
            {currentModule === 'password' ? (
              <GroupTree
                groups={groups}
                groupTree={groupTree}
                selectedGroupId={selectedGroupId}
                expandedKeys={expandedKeys}
                onExpanded={(keys) => setExpandedKeys(keys as string[])}
                onSelect={handleGroupSelect}
                setGroupTree={(tree) => setGroupTree(tree)}
                onEditGroup={handleEditGroup}
                onDeleteGroup={handleDeleteGroup}
                treeKey={treeKey}
              />
            ) : (
              <NoteGroupTree
                groups={noteGroups as any}
                groupTree={noteGroupTree as any}
                selectedGroupId={selectedNoteGroupId}
                onSelect={handleNoteGroupSelect}
                setGroupTree={(tree) => setNoteGroupTree(tree)}
                setGroups={(list) => setNoteGroups(list)}
                onEditGroup={handleEditNoteGroup}
                onDeleteGroup={handleDeleteNoteGroup}
              />
            )}
          </div>
        </Sider>
        
        <Layout style={{ padding: '24px' }}>
          <Content style={{ background: '#fff', padding: '22px', borderRadius: '8px' }}>
            {currentModule === 'password' ? (
              <>
                <div style={{ marginBottom: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <h2 style={{ margin: 0, whiteSpace: 'nowrap' }}>
                    {searchQuery ? '搜索结果' : selectedGroupId ? groups.find(g => g.id === selectedGroupId)?.name : '最新记录'}
                  </h2>
                  <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>添加密码</Button>
                </div>
                <Table 
                  columns={columns} 
                  dataSource={passwords} 
                  rowKey="id" 
                  loading={loading} 
                  tableLayout="fixed"
                  scroll={{ x: 'max-content' }}
                  pagination={{ total: passwords.length, pageSize: 10, showSizeChanger: true, showQuickJumper: true, showTotal: (total) => `共 ${total} 条记录` }} 
                />
              </>
            ) : (
              <>
                <div style={{ marginBottom: '16px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <h2 style={{ margin: 0, whiteSpace: 'nowrap' }}>
                    {selectedNoteGroupId ? noteGroups.find(g => g.id === selectedNoteGroupId)?.name || '便笺' : '最新便笺'}
                  </h2>
                  <Button type="primary" icon={<PlusOutlined />} onClick={() => setNoteCreateSignal(s => s + 1)}>添加便笺</Button>
                </div>
                <React.Suspense fallback={<div>正在加载便笺模块...</div>}>
                  <NoteManager
                    onClose={() => {}}
                    selectedGroupId={selectedNoteGroupId}
                    externalGroups={noteGroups as any}
                    hideTopFilter
                    createSignal={noteCreateSignal}
                    openNoteId={noteOpenId}
                    openSignal={noteOpenSignal}
                    createTemplate={undefined}
                    templateSignal={0}
                  />
                </React.Suspense>
              </>
            )}
          </Content>
        </Layout>
       </Layout>

      <PasswordDetailModal
        visible={modalVisible}
        password={editingPassword}
        groups={groups}
        mode={passwordDetailMode}
        onEdit={handleEdit}
        onClose={() => setModalVisible(false)}
        onSave={handleSubmit}
        onDelete={handleDelete}
      />

      <Modal
        title={editingGroup ? '编辑分组' : '新建分组'}
        open={groupModalVisible}
        onCancel={() => setGroupModalVisible(false)}
        footer={null}
      >
        <Form
          form={groupForm}
          layout="vertical"
          onFinish={handleSubmitGroup}
        >
          <Form.Item
            name="name"
            label="分组名称"
            rules={[{ required: true, message: '请输入分组名称' }]}
          >
            <Input placeholder="分组名称" />
          </Form.Item>
          
          <Form.Item
            name="parent_id"
            label="父级分组"
          >
            <Select placeholder="选择父级分组" allowClear>
              {groups.filter(g => !editingGroup || g.id !== editingGroup.id).map(group => (
                <Option key={group.id} value={group.id}>
                  {group.icon === 'folder' ? <FolderOutlined /> : null} {group.name}
                </Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="color"
            label="颜色"
            initialValue="blue"
          >
            <Select>
              <Option value="blue">蓝色</Option>
              <Option value="green">绿色</Option>
              <Option value="red">红色</Option>
              <Option value="orange">橙色</Option>
              <Option value="purple">紫色</Option>
              <Option value="cyan">青色</Option>
              <Option value="magenta">洋红色</Option>
              <Option value="yellow">黄色</Option>
              <Option value="pink">粉色</Option>
              <Option value="geekblue">极客蓝</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="sort"
            label="排序"
            tooltip="数字越小越靠前"
          >
            <Input type="number" placeholder="请输入排序数字" />
          </Form.Item>
          
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                {editingGroup ? '更新' : '添加'}
              </Button>
              <Button onClick={() => setGroupModalVisible(false)}>
                取消
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="密码历史"
        open={historyModalVisible}
        onCancel={() => setHistoryModalVisible(false)}
        footer={[
          <Button key="close" onClick={() => setHistoryModalVisible(false)}>
            关闭
          </Button>
        ]}
        width={800}
      >
        <Table
          columns={historyColumns}
          dataSource={passwordHistory}
          rowKey="id"
          pagination={false}
          locale={{ emptyText: '暂无历史记录' }}
        />
      </Modal>

      <React.Suspense fallback={null}>
        <PasswordGenerator
          visible={generatorVisible}
          onClose={() => setGeneratorVisible(false)}
          onGenerate={handleGeneratePassword}
        />
      </React.Suspense>

      <Modal
        title="用户设置"
        open={settingsVisible}
        onCancel={() => setSettingsVisible(false)}
        footer={null}
        width={1000}
        style={{ top: 32 }}
        destroyOnHidden
      >
        <React.Suspense fallback={<div>正在加载设置...</div>}>
          <UserSettings onClose={() => setSettingsVisible(false)} />
        </React.Suspense>
      </Modal>

      <React.Suspense fallback={null}>
        <ImportExportModal
          visible={importExportVisible}
          onClose={() => setImportExportVisible(false)}
        />
      </React.Suspense>

      
      <Modal title={editingNoteGroup ? '编辑便笺分组' : '新建便笺分组'} open={noteGroupModalVisible} onCancel={() => setNoteGroupModalVisible(false)} footer={null}>
        <Form form={noteGroupForm} layout="vertical" onFinish={handleSubmitNoteGroup}>
          <Form.Item name="name" label="分组名称" rules={[{ required: true, message: '请输入分组名称' }]}>
            <Input placeholder="分组名称" />
          </Form.Item>
          <Form.Item name="parent_id" label="父级分组">
            <Select placeholder="选择父级分组" allowClear>
              {(noteGroups || []).filter((g) => !editingNoteGroup || g.id !== editingNoteGroup.id).map(group => (
                <Option key={group.id} value={group.id as number}>{group.name}</Option>
              ))}
            </Select>
          </Form.Item>
          <Form.Item name="color" label="颜色" initialValue="blue">
            <Select>
              <Option value="blue">蓝色</Option>
              <Option value="green">绿色</Option>
              <Option value="red">红色</Option>
              <Option value="orange">橙色</Option>
              <Option value="purple">紫色</Option>
              <Option value="cyan">青色</Option>
              <Option value="magenta">洋红色</Option>
              <Option value="yellow">黄色</Option>
              <Option value="pink">粉色</Option>
              <Option value="geekblue">极客蓝</Option>
            </Select>
          </Form.Item>
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">{editingNoteGroup ? '更新' : '添加'}</Button>
              <Button onClick={() => setNoteGroupModalVisible(false)}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>

      

      <Modal title="全局搜索" open={globalSearchVisible} onCancel={() => setGlobalSearchVisible(false)} footer={null} width={900}>
        <Tabs activeKey={globalSearchActiveTab} onChange={(k) => setGlobalSearchActiveTab(k as any)} items={[
          { key: 'pw', label: `密码（${globalSearchPasswords.length}）`, children: (
            <Table size="small" pagination={{ pageSize: 10 }} rowKey="id" dataSource={globalSearchPasswords} columns={[
              { title: '标题', dataIndex: 'title' },
              { title: '用户名', dataIndex: 'username' },
              { title: '分组', dataIndex: 'group_id', render: (gid: number) => { const g = groups.find(x => x.id === gid); return g ? g.name : '未分组'; } },
              { title: '操作', render: (_: any, row: any) => (<Button size="small" onClick={() => { setCurrentModule('password'); handleView(row); setGlobalSearchVisible(false); }}>打开</Button>) }
            ]} />
          ) },
          { key: 'nt', label: `便笺（${globalSearchNotes.length}）`, children: (
            <Table size="small" pagination={{ pageSize: 10 }} rowKey="id" dataSource={globalSearchNotes} columns={[
              { title: '标题', dataIndex: 'title' },
              { title: '分组', dataIndex: 'group_id', render: (gid: number) => { const g = noteGroups.find(x => x.id === gid); return g ? g.name : '未分组'; } },
              { title: '更新时间', dataIndex: 'updated_at' },
              { title: '操作', render: (_: any, row: any) => (<Button size="small" onClick={() => { setCurrentModule('notes'); setGlobalSearchVisible(false); setNoteOpenId(row.id); setNoteOpenSignal(s => s + 1); }}>打开</Button>) }
            ]} />
          ) }
        ]} />
      </Modal>

      <Modal title="命令面板" open={cmdPaletteVisible} onCancel={() => setCmdPaletteVisible(false)} footer={null}>
        <Space direction="vertical" style={{ width: '100%' }}>
          <Button onClick={() => { setCurrentModule('password'); setCmdPaletteVisible(false); }}>切换到密码模块</Button>
          <Button onClick={() => { setCurrentModule('notes'); setCmdPaletteVisible(false); }}>切换到便笺模块</Button>
          <Button onClick={() => { if (currentModule === 'password') { handleAdd(); } else { setNoteCreateSignal((s) => s + 1); } setCmdPaletteVisible(false); }}>快速新建当前模块条目</Button>
        </Space>
        <div style={{ marginTop: 8, color: '#999' }}>快捷键：⌘1/⌘2 切模块 · ⌘F 搜索 · ⌘N 新建 · ⌘K 打开此面板</div>
      </Modal>
    </Layout>
    <MasterPasswordGate
      visible={!checkingSecurity && locked}
      state={securityState}
      loading={securityLoading}
      onUnlock={handleUnlock}
      onSetup={handleSetupMaster}
    />
    </>
  );
};

export default App;
