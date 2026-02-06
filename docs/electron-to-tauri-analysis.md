# MyloAir 密码管理器：Electron 到 Tauri 2.0 迁移分析

## 项目概览

**项目名称**: MyloAir
**当前技术栈**: Electron 27 + React 18 + TypeScript + SQLite  
**目标技术栈**: Tauri 2.0 + React 18 + TypeScript + SQLite  
**开发环境**: M1 Mac (arm64)

## 一、核心依赖分析

### 1.1 Node.js 原生模块

| 模块名称                 | 用途          | Tauri 替代方案            | 迁移难度      |
| ------------------------ | ------------- | ------------------------- | ------------- |
| `better-sqlite3`         | SQLite 数据库 | `tauri-plugin-sql`        | ⭐⭐⭐ 中等   |
| `archiver`               | ZIP 压缩      | Rust `zip` crate 或前端库 | ⭐⭐ 简单     |
| `archiver-zip-encrypted` | 加密 ZIP      | 手写 Rust 或 `zip-crypto` | ⭐⭐⭐⭐ 困难 |
| `electron-store`         | 持久化配置    | `tauri-plugin-store`      | ⭐ 简单       |
| `electron-updater`       | 自动更新      | `tauri-plugin-updater`    | ⭐⭐ 简单     |
| `node-stream-zip`        | ZIP 解压      | Rust `zip` crate          | ⭐⭐ 简单     |

### 1.2 Electron 特定 API

| API                   | 用途         | Tauri 替代方案          |
| --------------------- | ------------ | ----------------------- |
| `BrowserWindow`       | 窗口管理     | `tauri::Window` API     |
| `ipcMain/ipcRenderer` | 进程通信     | Tauri Commands + Events |
| `dialog`              | 文件对话框   | `tauri-plugin-dialog`   |
| `shell.openExternal`  | 打开外部链接 | `tauri-plugin-shell`    |
| `app.getPath()`       | 系统路径     | `tauri::api::path`      |
| `Menu`                | 应用菜单     | Tauri Menu API          |

## 二、IPC 通信接口分析

### 2.1 IPC 通信域划分

项目采用模块化 IPC 设计，共 7 个通信域：

#### 1. 密码管理 (passwords.ts)

```typescript
// 主要接口
- get-passwords: 获取密码列表
- add-password: 添加密码
- update-password: 更新密码
- delete-password: 删除密码
- search-passwords: 搜索密码
- advanced-search: 高级搜索
- get-password-history: 获取密码历史
- update-password-with-history: 更新密码并记录历史
```

#### 2. 分组管理 (groups.ts)

```typescript
- get-groups: 获取所有分组
- get-group-tree: 获取分组树
- add-group: 添加分组
- update-group: 更新分组
- delete-group: 删除分组
```

#### 3. 笔记管理 (notes.ts)

```typescript
- get-note-groups: 获取笔记分组
- get-notes: 获取笔记列表
- add-note: 添加笔记
- update-note: 更新笔记
- delete-note: 删除笔记
- search-notes-title: 搜索笔记标题
```

#### 4. 设置管理 (settings.ts)

```typescript
- get-user-settings: 获取用户设置
- set-user-setting: 设置用户配置
- update-user-setting: 更新设置
- delete-user-setting: 删除设置
- reset-setting-to-default: 重置为默认值
- export-settings: 导出设置
- import-settings: 导入设置
```

#### 5. 备份管理 (backup.ts)

```typescript
- export-data: 导出数据（返回字节数组）
- export-data-to-file: 导出数据到文件
- pick-export-path: 选择导出路径
- pick-export-directory: 选择导出目录
- import-data: 导入数据
- check-data-integrity: 检查数据完整性
- repair-data-integrity: 修复数据完整性
```

#### 6. 安全管理 (security.ts)

```typescript
- security-get-state: 获取主密码状态
- security-set-master-password: 设置主密码
- security-verify-master-password: 验证主密码
- security-update-master-password: 更新主密码
- security-clear-master-password: 清除主密码
- security-set-require-master-password: 设置是否需要主密码
```

#### 7. 日志管理 (logging.ts)

```typescript
- renderer-report-error: 渲染进程错误上报
```

#### 8. 窗口管理 (main.ts)

```typescript
- window-minimize: 最小化窗口
- window-toggle-maximize: 切换最大化
- window-close: 关闭窗口
- open-external: 打开外部链接
```

### 2.2 事件通信

```typescript
// 主进程 -> 渲染进程事件
- data-imported: 数据导入完成通知
- auto-export-done: 自动导出完成通知
```

## 三、核心服务架构

### 3.1 服务层设计

```
DatabaseService (主协调器)
├── GroupService (分组管理)
├── PasswordService (密码管理)
├── NoteService (笔记管理)
├── SettingsService (设置管理)
├── BackupService (备份管理)
├── IntegrityService (数据完整性)
├── SecurityService (安全管理)
└── AutoExportService (自动导出)
```

### 3.2 数据库表结构

```sql
-- 核心表
- passwords: 密码记录
- password_history: 密码历史
- groups: 分组
- secure_records: 安全笔记
- secure_record_groups: 笔记分组
- user_settings: 用户设置
- master_password: 主密码
```

## 四、Tauri 迁移方案

### 4.1 可直接使用 Tauri 官方插件的功能

| 功能模块      | Tauri 插件             | 版本 |
| ------------- | ---------------------- | ---- |
| SQLite 数据库 | `tauri-plugin-sql`     | v2   |
| 文件系统      | `tauri-plugin-fs`      | v2   |
| 对话框        | `tauri-plugin-dialog`  | v2   |
| 持久化存储    | `tauri-plugin-store`   | v2   |
| 自动更新      | `tauri-plugin-updater` | v2   |
| Shell 操作    | `tauri-plugin-shell`   | v2   |
| 日志          | `tauri-plugin-log`     | v2   |

### 4.2 需要手写 Rust 的功能

#### 1. 加密 ZIP 压缩/解压 ⭐⭐⭐⭐

**原因**: `archiver-zip-encrypted` 是 Node.js 特定库，Tauri 无官方插件  
**方案**:

- 使用 Rust `zip` crate + `aes` crate 手写
- 或使用前端纯 JS 库（如 `jszip` + `crypto-js`）在渲染进程处理

#### 2. 数据加密/解密 ⭐⭐

**原因**: 当前使用 Node.js `crypto` 模块  
**方案**:

- 使用 Rust `aes` + `cbc` crate
- 或使用 `tauri-plugin-stronghold`（更安全）

#### 3. 自动导出服务 ⭐⭐⭐

**原因**: 依赖 Electron 的定时器和文件系统  
**方案**:

- 使用 Rust `tokio` 定时器 + `tauri-plugin-fs`

### 4.3 架构对比

| 层级     | Electron                  | Tauri                       |
| -------- | ------------------------- | --------------------------- |
| 前端     | React + TypeScript        | React + TypeScript (无变化) |
| 通信层   | IPC (ipcMain/ipcRenderer) | Commands + Events           |
| 后端     | Node.js (TypeScript)      | Rust                        |
| 数据库   | better-sqlite3            | tauri-plugin-sql            |
| 打包体积 | ~150MB                    | ~10MB (预估)                |

## 五、迁移优先级建议

### 第一批：核心数据层（高优先级）

1. 数据库服务 (DatabaseService)
2. 密码管理 (PasswordService + IPC)
3. 分组管理 (GroupService + IPC)
4. 加密/解密功能

### 第二批：用户功能（中优先级）

5. 笔记管理 (NoteService + IPC)
6. 设置管理 (SettingsService + IPC)
7. 窗口管理

### 第三批：高级功能（低优先级）

8. 备份导入导出 (BackupService + IPC)
9. 安全管理 (SecurityService + IPC)
10. 自动导出服务
11. 数据完整性检查

## 六、风险评估

### 高风险项

1. **加密 ZIP 功能**: 需要确保与现有数据格式兼容
2. **数据库迁移**: SQLite 版本差异可能导致兼容性问题
3. **加密算法一致性**: 必须保证加密/解密结果与 Electron 版本一致

### 中风险项

1. **IPC 重构**: 大量接口需要逐一迁移和测试
2. **macOS 特定行为**: 窗口隐藏而非关闭的逻辑
3. **自动更新**: 需要配置签名和公证

### 低风险项

1. **前端代码**: React 组件基本无需修改
2. **UI 样式**: Ant Design 组件库完全兼容
3. **构建配置**: Tauri 提供完善的构建工具

## 七、性能优化预期

| 指标       | Electron | Tauri (预估) | 提升  |
| ---------- | -------- | ------------ | ----- |
| 安装包大小 | ~150MB   | ~10MB        | 93% ↓ |
| 内存占用   | ~200MB   | ~50MB        | 75% ↓ |
| 启动速度   | ~2s      | ~0.5s        | 75% ↑ |
| CPU 占用   | 中       | 低           | 30% ↓ |

## 八、下一步行动

1. **环境准备**: 安装 Rust 工具链和 Tauri CLI
2. **项目初始化**: 创建 Tauri 2.0 项目骨架
3. **数据库层迁移**: 优先完成 SQLite 和加密功能
4. **逐模块迁移**: 按优先级逐个迁移功能模块
5. **测试验证**: 每个模块迁移后立即测试

---

**文档版本**: v1.0  
**创建时间**: 2026-02-05  
**作者**: Antigravity AI
