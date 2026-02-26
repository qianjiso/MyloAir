# Tauri 开发/生产环境自动切换方案

## 问题

每次执行 `cargo tauri dev` 或 `cargo tauri build` 时需要手动修改 `tauri.conf.json` 中的 `identifier` 字段：

- 开发环境：`com.yourcompany.myloair.dev`
- 生产环境：`com.yourcompany.myloair`

## 解决方案

已创建自动化脚本 `scripts/set-tauri-env.js`，在运行 Tauri 命令前自动切换配置。

## 使用方法

### 开发环境

```bash
npm run tauri:dev
```

自动将 identifier 设置为 `com.yourcompany.myloair.dev`，然后启动开发服务器。

### 生产环境

```bash
npm run tauri:build
```

自动将 identifier 设置为 `com.yourcompany.myloair`，然后执行生产构建。

### 手动切换（可选）

如果只想切换配置而不运行 Tauri：

```bash
# 切换到开发环境
npm run tauri:set-dev

# 切换到生产环境
npm run tauri:set-prod
```

## 工作原理

1. `scripts/set-tauri-env.js` - Node.js 脚本，读取并修改 `tauri.conf.json`
2. `package.json` 中的 npm scripts 在运行 Tauri 命令前先调用此脚本
3. 脚本会检查当前配置，只在需要时才修改文件

## 优势

✅ **无需手动修改配置文件** - 完全自动化  
✅ **避免提交错误配置** - 每次构建前自动设置正确的 identifier  
✅ **简单易用** - 只需记住 `npm run tauri:dev` 和 `npm run tauri:build`  
✅ **幂等性** - 重复运行不会产生副作用

## 注意事项

- 确保 `scripts/set-tauri-env.js` 有执行权限
- 如果需要添加更多环境配置差异，可以扩展脚本
- 建议将 `src-tauri/tauri.conf.json` 的默认 identifier 设置为生产环境值
