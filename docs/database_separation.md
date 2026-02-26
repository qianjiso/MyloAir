# 数据库分离方案

## 为什么开发和生产使用不同的数据库？

为了避免开发环境的测试数据污染生产环境，我们使用不同的应用标识符（identifier）来分离数据库路径。

## 数据库路径配置

**开发环境**：

- Identifier: `com.yourcompany.myloair.dev`
- 数据库路径（macOS）: `~/Library/Application Support/com.yourcompany.myloair.dev/myloair.db`
- 配置文件：`src-tauri/tauri.conf.json`

**生产环境**：

- Identifier: `com.yourcompany.myloair`
- 数据库路径（macOS）: `~/Library/Application Support/com.yourcompany.myloair/myloair.db`
- 配置文件：打包前需修改 `tauri.conf.json` 的 identifier

## 如何切换环境？

> 💡 **推荐方案**：使用自动化脚本，无需手动修改配置文件。详见 [Tauri 开发/生产环境自动切换方案](./tauri_dev_build_solution.md)

**自动化方式（推荐）**：

```bash
# 开发环境（自动设置为 .dev）
npm run tauri:dev

# 生产打包（自动设置为正式版）
npm run tauri:build
```

**手动方式**：

1. **开发环境** - 默认配置（已设置为 `.dev`）

   ```bash
   cargo tauri dev
   ```

2. **生产打包** - 需要修改 identifier

   ```bash
   # 1. 编辑 src-tauri/tauri.conf.json
   # 将 identifier 从 "com.yourcompany.myloair.dev" 改为 "com.yourcompany.myloair"

   # 2. 打包
   cargo tauri build

   # 3. 打包完成后，记得改回 .dev 以便继续开发
   ```

## 数据迁移

如果需要将旧开发数据迁移到新路径：

**macOS**:

```bash
cp ~/Library/Application\ Support/com.yourcompany.myloair/myloair.db \
   ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db
```

**Windows**:

```cmd
copy "%APPDATA%\com.yourcompany.myloair\myloair.db" ^
     "%APPDATA%\com.yourcompany.myloair.dev\myloair.db"
```

## 技术实现

- **文件**：`src-tauri/tauri.conf.json` (Line 5)
- **原理**：Tauri 使用 identifier 生成 `app_data_dir` 路径
- **修改记录**：2026-02-10 - 实现开发/生产数据库隔离
