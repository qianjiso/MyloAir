# 数据库调试与验证

## 数据库位置

**开发环境**:

```bash
~/Library/Application Support/com.yourcompany.myloair.dev/myloair.db
```

**生产环境**:

```bash
~/Library/Application Support/com.yourcompany.myloair/myloair.db
```

## 常用验证命令

### 查看主密码状态

```bash
sqlite3 ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
  "SELECT id, password_hash IS NOT NULL as has_hash, hint, require_password FROM master_password WHERE id = 1"
```

**输出说明**:

- `has_hash = 1`: 已设置主密码
- `has_hash = 0`: 未设置主密码或已清除

### 查看所有密码条目

```bash
sqlite3 ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
  "SELECT id, title, username, group_id FROM passwords"
```

### 查看所有分组

```bash
sqlite3 ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
  "SELECT id, name, parent_id, sort_order FROM groups"
```

### 查看用户设置

```bash
sqlite3 ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
  "SELECT key, value, type, category FROM user_settings"
```

### 查看笔记

```bash
sqlite3 ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
  "SELECT id, title, group_id FROM secure_records"
```

## 数据库表结构

### 主要表

- `master_password` - 主密码配置
- `passwords` - 密码条目
- `groups` - 密码分组
- `secure_records` - 安全笔记
- `secure_record_groups` - 笔记分组
- `user_settings` - 用户设置
- `password_history` - 密码历史记录

### 查看表结构

```bash
sqlite3 ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
  ".schema master_password"
```

## 备份与恢复

### 备份数据库

```bash
cp ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db \
   ~/Desktop/myloair_backup_$(date +%Y%m%d_%H%M%S).db
```

### 恢复数据库

```bash
cp ~/Desktop/myloair_backup_20260211_150000.db \
   ~/Library/Application\ Support/com.yourcompany.myloair.dev/myloair.db
```

## 注意事项

⚠️ **警告**: 直接修改数据库可能导致数据损坏或应用异常,仅用于调试和验证。

✅ **建议**: 在执行任何数据库操作前,先备份数据库文件。
