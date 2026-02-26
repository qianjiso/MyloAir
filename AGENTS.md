# AGENTS.md

## 项目信息

**项目名称**: MyloAir  
**项目描述**: 安全的跨平台密码管理应用  
**技术栈**: Tauri 2.0 + React 18 + TypeScript + SQLite (Rust)

---

## 🚨 当前迁移任务

本项目正在从 **Electron** 迁移到 **Tauri 2.0**。

### 迁移文档

| 文件                                 | 说明                 |
| ------------------------------------ | -------------------- |
| `docs/electron-to-tauri-analysis.md` | 项目分析（迁移背景） |
| `bak/`                               | 原 Electron 项目备份 |

## 项目结构

```
MyloAir/
├── src-tauri/          # Tauri Rust 后端
│   ├── src/
│   │   ├── commands/   # Tauri Commands
│   │   ├── services/   # 业务逻辑
│   │   └── models/     # 数据模型
│   └── Cargo.toml
├── src/               # 前端代码
│   └── renderer/      # React 应用
├── docs/              # 文档
└── bak/               # Electron 版本备份
```

## 构建命令

```bash
# 开发模式
npm run tauri:dev

# 生产构建
npm run tauri:build

# 仅检查 Rust 编译
cargo check --manifest-path src-tauri/Cargo.toml
```
