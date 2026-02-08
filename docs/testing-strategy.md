# MyloAir 自动化测试策略

## 问题背景

`npm run dev` 只能测试前端渲染进程，无法真实反映 `cargo tauri dev` 的实际运行情况，因为：

1. **后端逻辑缺失**: Rust 后端的 Tauri Commands 不会被调用
2. **Mock API**: 前端使用的是 `electronAPI-mock.ts`，而非真实的 Tauri invoke
3. **加密服务**: 真实的加密/解密逻辑在 Rust 端，Mock 无法完全模拟
4. **数据库操作**: SQLite 数据库操作只在 Rust 端执行

---

## 推荐的测试方案

### 方案 1: Rust 单元测试 + 集成测试（推荐）

#### 1.1 Rust 单元测试

为每个 service 和 command 编写单元测试。

**优点**:
- 快速执行
- 不依赖 UI
- 可以精确测试每个功能点
- CI/CD 友好

**示例**: 已有的数据库测试

```rust
// src-tauri/src/services/database.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_password_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_password.db");
        let db_service = DatabaseService::new(db_path.to_str().unwrap());
        db_service.initialize().unwrap();

        // 测试添加、获取、更新、删除密码
        // ...
    }
}
```

**需要添加的测试**:

```rust
// src-tauri/src/commands/passwords.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_password() {
        let options = PasswordGeneratorOptions {
            length: Some(16),
            include_uppercase: Some(true),
            include_lowercase: Some(true),
            include_numbers: Some(true),
            include_symbols: Some(true),
        };
        
        let result = generate_password(options).await;
        assert!(result.is_ok());
        
        let password = result.unwrap();
        assert_eq!(password.len(), 16);
    }

    #[tokio::test]
    async fn test_generate_password_no_charset() {
        let options = PasswordGeneratorOptions {
            length: Some(16),
            include_uppercase: Some(false),
            include_lowercase: Some(false),
            include_numbers: Some(false),
            include_symbols: Some(false),
        };
        
        let result = generate_password(options).await;
        assert!(result.is_err());
    }
}
```

**运行方式**:
```bash
cd src-tauri
cargo test
```

---

#### 1.2 Rust 集成测试

测试完整的 command 调用流程（包括状态管理）。

**创建文件**: `src-tauri/tests/integration_test.rs`

```rust
use myloair_lib::{AppState, commands};
use myloair_lib::services::database::DatabaseService;
use std::sync::Mutex;
use tempfile::tempdir;

#[tokio::test]
async fn test_password_workflow() {
    // 设置测试环境
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db_service = DatabaseService::new(db_path.to_str().unwrap());
    db_service.initialize().unwrap();
    
    let state = AppState {
        db: db_service,
        session: Mutex::new(None),
    };
    
    // 测试添加分组
    let group = myloair_lib::models::Group {
        id: None,
        name: "测试分组".to_string(),
        parent_id: None,
        icon: None,
        color: Some("blue".to_string()),
        sort_order: Some(0),
        created_at: None,
        updated_at: None,
    };
    
    let result = commands::groups::add_group(
        tauri::State::from(&state),
        group
    ).await;
    
    assert!(result.is_ok());
    // ... 更多测试
}
```

**运行方式**:
```bash
cd src-tauri
cargo test --test integration_test
```

---

### 方案 2: WebDriver 端到端测试

使用 WebDriver 自动化测试完整的 Tauri 应用。

#### 2.1 使用 WebDriver

**安装依赖**:
```bash
npm install --save-dev webdriverio @wdio/cli @wdio/local-runner @wdio/mocha-framework
```

**配置文件**: `wdio.conf.js`

```javascript
exports.config = {
    runner: 'local',
    specs: [
        './tests/e2e/**/*.spec.js'
    ],
    capabilities: [{
        maxInstances: 1,
        'tauri:options': {
            application: './src-tauri/target/debug/myloair'
        }
    }],
    logLevel: 'info',
    framework: 'mocha',
    reporters: ['spec'],
    mochaOpts: {
        ui: 'bdd',
        timeout: 60000
    }
}
```

**测试示例**: `tests/e2e/password.spec.js`

```javascript
describe('Password Management', () => {
    it('should create a new password entry', async () => {
        // 等待应用加载
        await browser.pause(2000);
        
        // 点击新建密码按钮
        const addBtn = await $('button*=新建密码');
        await addBtn.click();
        
        // 填写表单
        const titleInput = await $('input[placeholder*="标题"]');
        await titleInput.setValue('测试密码');
        
        const usernameInput = await $('input[placeholder*="用户名"]');
        await usernameInput.setValue('testuser');
        
        // 点击保存
        const saveBtn = await $('button*=保存');
        await saveBtn.click();
        
        // 验证密码已创建
        await browser.pause(1000);
        const passwordItem = await $('*=测试密码');
        expect(await passwordItem.isDisplayed()).toBe(true);
    });
});
```

**运行方式**:
```bash
# 先构建应用
cargo tauri build --debug

# 运行测试
npx wdio run wdio.conf.js
```

---

### 方案 3: Tauri 官方测试工具（最简单）

Tauri 2.0 提供了内置的测试支持。

#### 3.1 使用 `tauri-driver`

**安装**:
```bash
cargo install tauri-driver
```

**创建测试**: `tests/webdriver.rs`

```rust
use tauri_driver::WebDriver;

#[test]
fn test_password_creation() {
    let driver = WebDriver::new("myloair").unwrap();
    
    // 等待应用启动
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // 使用 WebDriver API 进行测试
    let add_button = driver.find_element_by_text("新建密码").unwrap();
    add_button.click().unwrap();
    
    // ... 更多测试步骤
}
```

---

### 方案 4: 混合测试策略（推荐用于生产）

结合多种测试方法，形成完整的测试金字塔：

```
        /\
       /  \      E2E Tests (WebDriver)
      /    \     - 关键用户流程
     /------\    
    /        \   Integration Tests (Rust)
   /          \  - Command 调用流程
  /------------\ 
 /              \ Unit Tests (Rust + Jest)
/________________\ - 单个函数/模块
```

**测试脚本**: `package.json`

```json
{
  "scripts": {
    "test:unit": "cd src-tauri && cargo test",
    "test:integration": "cd src-tauri && cargo test --test integration_test",
    "test:e2e": "npm run build && npx wdio run wdio.conf.js",
    "test:all": "npm run test:unit && npm run test:integration && npm run test:e2e"
  }
}
```

---

## 具体实施建议

### 第一阶段：基础测试（1-2天）

1. **为新功能添加单元测试**
   - `test_generate_password` - 密码生成功能
   - `test_group_tree_building` - 分组树构建
   - `test_password_serialization` - 序列化测试

2. **运行现有测试**
   ```bash
   cd src-tauri
   cargo test
   ```

### 第二阶段：集成测试（2-3天）

1. **创建集成测试文件**
   - `tests/password_workflow.rs`
   - `tests/group_hierarchy.rs`

2. **测试完整流程**
   - 创建分组 → 创建密码 → 验证保存
   - 父子分组 → 验证层级关系

### 第三阶段：E2E 测试（3-5天）

1. **设置 WebDriver 环境**
2. **编写关键路径测试**
   - 用户登录流程
   - 密码 CRUD 操作
   - 密码生成器

---

## CI/CD 集成

**GitHub Actions 示例**: `.github/workflows/test.yml`

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Rust tests
        run: |
          cd src-tauri
          cargo test
      
      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install dependencies
        run: npm install
      
      - name: Build Tauri app
        run: npm run tauri build -- --debug
      
      - name: Run E2E tests
        run: npm run test:e2e
```

---

## 快速开始

**立即可以做的**（5分钟）:

```bash
# 1. 运行现有的 Rust 测试
cd src-tauri
cargo test

# 2. 添加一个简单的测试
# 编辑 src-tauri/src/commands/passwords.rs，添加上面的测试代码

# 3. 再次运行测试
cargo test
```

---

## 总结

| 测试类型 | 执行速度 | 覆盖范围 | 维护成本 | 推荐度 |
|---------|---------|---------|---------|--------|
| Rust 单元测试 | ⚡️⚡️⚡️ | 单个函数 | 低 | ⭐️⭐️⭐️⭐️⭐️ |
| Rust 集成测试 | ⚡️⚡️ | Command 层 | 中 | ⭐️⭐️⭐️⭐️ |
| WebDriver E2E | ⚡️ | 完整应用 | 高 | ⭐️⭐️⭐️ |

**建议优先级**:
1. ✅ **立即实施**: Rust 单元测试（已有基础，补充新功能）
2. 📅 **本周内**: Rust 集成测试（测试 command 调用）
3. 📅 **下周**: WebDriver E2E 测试（关键流程）
