//! 数据库服务
//! 
//! 封装 SQLite 数据库操作

use std::path::Path;
use rusqlite::{Connection, Result};
use tauri::{AppHandle, Manager};

/// 数据库服务
pub struct DatabaseService {
    pub db_path: String,
}

impl DatabaseService {
    /// 创建新的数据库服务实例
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }

    // ... (existing methods)

    /// 获取所有密码（可选按分组筛选）
    pub fn get_passwords(&self, group_id: Option<i64>) -> Result<Vec<crate::models::password::Password>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        
        let mut stmt = if let Some(_) = group_id {
            conn.prepare("SELECT id, title, username, password, url, notes, group_id, created_at, updated_at, last_used_at, use_count, favorite, tags FROM passwords WHERE group_id = ? ORDER BY title")
                .map_err(|e| e.to_string())?
        } else {
            conn.prepare("SELECT id, title, username, password, url, notes, group_id, created_at, updated_at, last_used_at, use_count, favorite, tags FROM passwords ORDER BY title")
                .map_err(|e| e.to_string())?
        };

        let password_iter = if let Some(gid) = group_id {
            stmt.query_map([gid], Self::map_password_row).map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], Self::map_password_row).map_err(|e| e.to_string())?
        };

        let mut passwords = Vec::new();
        for password in password_iter {
            passwords.push(password.map_err(|e| e.to_string())?);
        }

        Ok(passwords)
    }

    /// 获取单个密码
    pub fn get_password(&self, id: i64) -> Result<Option<crate::models::password::Password>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        
        let mut stmt = conn.prepare("SELECT id, title, username, password, url, notes, group_id, created_at, updated_at, last_used_at, use_count, favorite, tags FROM passwords WHERE id = ?")
            .map_err(|e| e.to_string())?;
        
        // 使用 query_map 获取 iterator
        let mut password_iter = stmt.query_map([id], Self::map_password_row)
            .map_err(|e| e.to_string())?;

        if let Some(password) = password_iter.next() {
            Ok(Some(password.map_err(|e| e.to_string())?))
        } else {
            Ok(None)
        }
    }

    /// 添加密码
    pub fn add_password(&self, password: &crate::models::password::Password) -> Result<i64, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO passwords (title, username, password, url, notes, group_id, favorite, tags, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))",
            (
                &password.title,
                &password.username,
                &password.password,
                &password.url,
                &password.notes,
                password.group_id,
                password.favorite.map(|f| if f { 1 } else { 0 }),
                &password.tags,
            ),
        ).map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    /// 更新密码
    pub fn update_password(&self, password: &crate::models::password::Password) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        
        if let Some(id) = password.id {
             conn.execute(
                "UPDATE passwords SET title=?1, username=?2, password=?3, url=?4, notes=?5, group_id=?6, favorite=?7, tags=?8, updated_at=datetime('now') WHERE id=?9",
                (
                    &password.title,
                    &password.username,
                    &password.password,
                    &password.url,
                    &password.notes,
                    password.group_id,
                    password.favorite.map(|f| if f { 1 } else { 0 }),
                    &password.tags,
                    id
                ),
            ).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Password ID is missing".to_string())
        }
    }

    /// 删除密码
    pub fn delete_password(&self, id: i64) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM passwords WHERE id = ?", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 搜索密码（模糊匹配标题、用户名、网址、备注）
    pub fn search_passwords(&self, keyword: &str) -> Result<Vec<crate::models::password::Password>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let pattern = format!("%{}%", keyword);
        
        let mut stmt = conn.prepare(
            "SELECT id, title, username, password, url, notes, group_id, created_at, updated_at, last_used_at, use_count, favorite, tags FROM passwords 
            WHERE title LIKE ?1 OR username LIKE ?1 OR url LIKE ?1 OR notes LIKE ?1 
            ORDER BY title"
        ).map_err(|e| e.to_string())?;

        let password_iter = stmt.query_map([&pattern], Self::map_password_row)
            .map_err(|e| e.to_string())?;

        let mut passwords = Vec::new();
        for password in password_iter {
            passwords.push(password.map_err(|e| e.to_string())?);
        }

        Ok(passwords)
    }

    /// 检查是否已设置主密码
    pub fn has_master_password(&self) -> Result<bool, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM master_password WHERE id = 1",
            [],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        Ok(count > 0)
    }

    /// 获取主密码哈希
    pub fn get_master_password_hash(&self) -> Result<Option<String>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT password_hash FROM master_password WHERE id = 1")
            .map_err(|e| e.to_string())?;
        
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let hash: String = row.get(0).map_err(|e| e.to_string())?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    /// 设置主密码
    pub fn set_master_password(&self, hash: &str, hint: Option<&str>) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO master_password (id, password_hash, hint, require_password, created_at, updated_at) 
             VALUES (1, ?1, ?2, 1, datetime('now'), datetime('now'))
             ON CONFLICT(id) DO UPDATE SET 
             password_hash=excluded.password_hash, 
             hint=excluded.hint, 
             updated_at=datetime('now')",
            (hash, hint),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 获取所有分组
    pub fn get_groups(&self) -> Result<Vec<crate::models::group::Group>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at FROM groups ORDER BY sort_order, name")
            .map_err(|e| e.to_string())?;
        
        let iter = stmt.query_map([], Self::map_group_row).map_err(|e| e.to_string())?;
        
        let mut groups = Vec::new();
        for group in iter {
            groups.push(group.map_err(|e| e.to_string())?);
        }
        Ok(groups)
    }

    /// 获取单个分组
    pub fn get_group(&self, id: i64) -> Result<Option<crate::models::group::Group>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at FROM groups WHERE id = ?")
            .map_err(|e| e.to_string())?;
        
        let mut iter = stmt.query_map([id], Self::map_group_row).map_err(|e| e.to_string())?;
        
        if let Some(group) = iter.next() {
            Ok(Some(group.map_err(|e| e.to_string())?))
        } else {
            Ok(None)
        }
    }

    /// 添加分组
    pub fn add_group(&self, group: &crate::models::group::Group) -> Result<i64, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO groups (name, parent_id, icon, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            (
                &group.name,
                group.parent_id,
                &group.icon,
                &group.color,
                group.sort_order,
            ),
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    /// 更新分组
    pub fn update_group(&self, group: &crate::models::group::Group) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        if let Some(id) = group.id {
            conn.execute(
                "UPDATE groups SET name=?1, parent_id=?2, icon=?3, color=?4, sort_order=?5, updated_at=datetime('now') WHERE id=?6",
                (
                    &group.name,
                    group.parent_id,
                    &group.icon,
                    &group.color,
                    group.sort_order,
                    id
                ),
            ).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Group ID is missing".to_string())
        }
    }

    /// 删除分组
    pub fn delete_group(&self, id: i64) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM groups WHERE id = ?", [id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Note Groups ---

    pub fn get_note_groups(&self) -> Result<Vec<crate::models::note::SecureRecordGroup>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at FROM secure_record_groups ORDER BY sort_order, name")
            .map_err(|e| e.to_string())?;
        let iter = stmt.query_map([], Self::map_note_group_row).map_err(|e| e.to_string())?;
        let mut groups = Vec::new();
        for group in iter { groups.push(group.map_err(|e| e.to_string())?); }
        Ok(groups)
    }

    pub fn get_note_group(&self, id: i64) -> Result<Option<crate::models::note::SecureRecordGroup>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, name, parent_id, icon, color, sort_order, created_at, updated_at FROM secure_record_groups WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let mut iter = stmt.query_map([id], Self::map_note_group_row).map_err(|e| e.to_string())?;
        if let Some(group) = iter.next() { Ok(Some(group.map_err(|e| e.to_string())?)) } else { Ok(None) }
    }

    pub fn add_note_group(&self, group: &crate::models::note::SecureRecordGroup) -> Result<i64, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO secure_record_groups (name, parent_id, icon, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            (&group.name, group.parent_id, &group.icon, &group.color, group.sort_order),
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_note_group(&self, group: &crate::models::note::SecureRecordGroup) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        if let Some(id) = group.id {
            conn.execute(
                "UPDATE secure_record_groups SET name=?1, parent_id=?2, icon=?3, color=?4, sort_order=?5, updated_at=datetime('now') WHERE id=?6",
                (&group.name, group.parent_id, &group.icon, &group.color, group.sort_order, id),
            ).map_err(|e| e.to_string())?;
            Ok(())
        } else { Err("Group ID missing".to_string()) }
    }

    pub fn delete_note_group(&self, id: i64) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM secure_record_groups WHERE id = ?", [id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Notes ---

    pub fn get_notes(&self, group_id: Option<i64>) -> Result<Vec<crate::models::note::SecureRecord>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let sql = if group_id.is_some() {
            "SELECT id, title, content, group_id, pinned, archived, created_at, updated_at FROM secure_records WHERE group_id = ? ORDER BY title"
        } else {
            "SELECT id, title, content, group_id, pinned, archived, created_at, updated_at FROM secure_records ORDER BY title"
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        
        let iter = if let Some(gid) = group_id {
            stmt.query_map([gid], Self::map_note_row).map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], Self::map_note_row).map_err(|e| e.to_string())?
        };

        let mut notes = Vec::new();
        for note in iter { notes.push(note.map_err(|e| e.to_string())?); }
        Ok(notes)
    }

    pub fn get_note(&self, id: i64) -> Result<Option<crate::models::note::SecureRecord>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, title, content, group_id, pinned, archived, created_at, updated_at FROM secure_records WHERE id = ?").map_err(|e| e.to_string())?;
        let mut iter = stmt.query_map([id], Self::map_note_row).map_err(|e| e.to_string())?;
        if let Some(note) = iter.next() { Ok(Some(note.map_err(|e| e.to_string())?)) } else { Ok(None) }
    }

    pub fn add_note(&self, note: &crate::models::note::SecureRecord) -> Result<i64, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO secure_records (title, content, group_id, pinned, archived, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            (
                &note.title,
                &note.content,
                note.group_id,
                note.pinned.map(|p| if p { 1 } else { 0 }),
                note.archived.map(|a| if a { 1 } else { 0 }),
            ),
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_note(&self, note: &crate::models::note::SecureRecord) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        if let Some(id) = note.id {
            conn.execute(
                "UPDATE secure_records SET title=?1, content=?2, group_id=?3, pinned=?4, archived=?5, updated_at=datetime('now') WHERE id=?6",
                (
                    &note.title,
                    &note.content,
                    note.group_id,
                    note.pinned.map(|p| if p { 1 } else { 0 }),
                    note.archived.map(|a| if a { 1 } else { 0 }),
                    id
                ),
            ).map_err(|e| e.to_string())?;
            Ok(())
        } else { Err("Note ID missing".to_string()) }
    }

    pub fn delete_note(&self, id: i64) -> Result<(), String> {
         let conn = self.get_connection().map_err(|e| e.to_string())?;
         conn.execute("DELETE FROM secure_records WHERE id = ?", [id]).map_err(|e| e.to_string())?;
         Ok(())
    }

    pub fn search_notes(&self, keyword: &str) -> Result<Vec<crate::models::note::SecureRecord>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let pattern = format!("%{}%", keyword);
        let mut stmt = conn.prepare(
            "SELECT id, title, content, group_id, pinned, archived, created_at, updated_at FROM secure_records WHERE title LIKE ?1 OR content LIKE ?1 ORDER BY title"
        ).map_err(|e| e.to_string())?;
        let iter = stmt.query_map([&pattern], Self::map_note_row).map_err(|e| e.to_string())?;
        let mut notes = Vec::new();
        for note in iter { notes.push(note.map_err(|e| e.to_string())?); }
        Ok(notes)
    }

    // --- Settings ---

    pub fn get_user_settings(&self, category: Option<&str>) -> Result<Vec<crate::models::setting::UserSetting>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let sql = if category.is_some() {
            "SELECT id, key, value, type, category, description, created_at, updated_at FROM user_settings WHERE category = ?"
        } else {
             "SELECT id, key, value, type, category, description, created_at, updated_at FROM user_settings"
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let iter = if let Some(cat) = category {
            stmt.query_map([cat], Self::map_setting_row).map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], Self::map_setting_row).map_err(|e| e.to_string())?
        };
        let mut settings = Vec::new();
        for setting in iter { settings.push(setting.map_err(|e| e.to_string())?); }
        Ok(settings)
    }

    pub fn get_user_setting(&self, key: &str) -> Result<Option<crate::models::setting::UserSetting>, String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, key, value, type, category, description, created_at, updated_at FROM user_settings WHERE key = ?").map_err(|e| e.to_string())?;
        let mut iter = stmt.query_map([key], Self::map_setting_row).map_err(|e| e.to_string())?;
        if let Some(setting) = iter.next() { Ok(Some(setting.map_err(|e| e.to_string())?)) } else { Ok(None) }
    }

    pub fn set_user_setting(&self, setting: &crate::models::setting::UserSetting) -> Result<(), String> {
         let conn = self.get_connection().map_err(|e| e.to_string())?;
         conn.execute(
             "INSERT INTO user_settings (key, value, type, category, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
              ON CONFLICT(key) DO UPDATE SET value=excluded.value, type=excluded.type, category=excluded.category, description=excluded.description, updated_at=datetime('now')",
              (
                  &setting.key,
                  &setting.value,
                  &setting.r#type,
                  &setting.category,
                  &setting.description
              )
         ).map_err(|e| e.to_string())?;
         Ok(())
    }

    pub fn delete_user_setting(&self, key: &str) -> Result<(), String> {
        let conn = self.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM user_settings WHERE key = ?", [key]).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 映射数据库行到 Group 结构体
    fn map_group_row(row: &rusqlite::Row) -> Result<crate::models::group::Group, rusqlite::Error> {
        Ok(crate::models::group::Group {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            icon: row.get(3)?,
            color: row.get(4)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    /// 映射数据库行到 Password 结构体
    fn map_password_row(row: &rusqlite::Row) -> Result<crate::models::password::Password, rusqlite::Error> {
        Ok(crate::models::password::Password {
            id: row.get(0)?,
            title: row.get(1)?,
            username: row.get(2)?,
            password: row.get(3)?,
            url: row.get(4)?,
            notes: row.get(5)?,
            group_id: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_used_at: row.get(9)?,
            use_count: row.get(10)?,
            favorite: row.get::<_, Option<i32>>(11)?.map(|v| v != 0),
            tags: row.get(12)?,
        })
    }

    fn map_note_group_row(row: &rusqlite::Row) -> Result<crate::models::note::SecureRecordGroup, rusqlite::Error> {
        Ok(crate::models::note::SecureRecordGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            icon: row.get(3)?,
            color: row.get(4)?,
            sort_order: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    fn map_note_row(row: &rusqlite::Row) -> Result<crate::models::note::SecureRecord, rusqlite::Error> {
         Ok(crate::models::note::SecureRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            group_id: row.get(3)?,
            pinned: row.get::<_, Option<i32>>(4)?.map(|v| v != 0),
            archived: row.get::<_, Option<i32>>(5)?.map(|v| v != 0),
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    fn map_setting_row(row: &rusqlite::Row) -> Result<crate::models::setting::UserSetting, rusqlite::Error> {
        Ok(crate::models::setting::UserSetting {
            id: row.get(0)?,
            key: row.get(1)?,
            value: row.get(2)?,
            r#type: row.get(3)?,
            category: row.get(4)?,
            description: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    /// 检查数据库是否存在
    pub fn exists(&self) -> bool {
        Path::new(&self.db_path).exists()
    }

    /// 获取数据库路径
    pub fn get_path(&self) -> &str {
        &self.db_path
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    /// 初始化数据库（创建表和索引）
    pub fn initialize(&self) -> Result<(), String> {
        let conn = self.get_connection()
            .map_err(|e| format!("无法连接数据库: {}", e))?;

        // 可以在这里开启 WAL 模式提高性能
        conn.execute_batch("PRAGMA journal_mode = WAL;")
           .map_err(|e| format!("设置 WAL 模式失败: {}", e))?;
        
        // 执行建表 SQL
        conn.execute_batch(CREATE_TABLES_SQL)
            .map_err(|e| format!("创建表失败: {}", e))?;

        Ok(())
    }
}

/// 数据库表创建 SQL
pub const CREATE_TABLES_SQL: &str = r#"
-- 分组表
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_id INTEGER,
    icon TEXT,
    color TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES groups(id) ON DELETE SET NULL
);

-- 密码表
CREATE TABLE IF NOT EXISTS passwords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    username TEXT,
    password TEXT,
    url TEXT,
    notes TEXT,
    group_id INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    last_used_at TEXT,
    use_count INTEGER DEFAULT 0,
    favorite INTEGER DEFAULT 0,
    tags TEXT,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE SET NULL
);

-- 密码历史表
CREATE TABLE IF NOT EXISTS password_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    password_id INTEGER NOT NULL,
    old_password TEXT NOT NULL,
    changed_at TEXT DEFAULT CURRENT_TIMESTAMP,
    change_reason TEXT,
    FOREIGN KEY (password_id) REFERENCES passwords(id) ON DELETE CASCADE
);

-- 用户设置表
CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT UNIQUE NOT NULL,
    value TEXT,
    type TEXT DEFAULT 'string',
    category TEXT,
    description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 安全笔记分组表
CREATE TABLE IF NOT EXISTS secure_record_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_id INTEGER,
    icon TEXT,
    color TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES secure_record_groups(id) ON DELETE SET NULL
);

-- 安全笔记表
CREATE TABLE IF NOT EXISTS secure_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT,
    group_id INTEGER,
    pinned INTEGER DEFAULT 0,
    archived INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES secure_record_groups(id) ON DELETE SET NULL
);

-- 主密码表
CREATE TABLE IF NOT EXISTS master_password (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    password_hash TEXT,
    hint TEXT,
    require_password INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_passwords_group_id ON passwords(group_id);
CREATE INDEX IF NOT EXISTS idx_passwords_title ON passwords(title);
CREATE INDEX IF NOT EXISTS idx_password_history_password_id ON password_history(password_id);
CREATE INDEX IF NOT EXISTS idx_groups_parent_id ON groups(parent_id);
CREATE INDEX IF NOT EXISTS idx_user_settings_key ON user_settings(key);
CREATE INDEX IF NOT EXISTS idx_secure_records_group_id ON secure_records(group_id);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_initialize_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let db_service = DatabaseService::new(db_path_str);
        
        assert!(!db_service.exists());
        db_service.initialize().expect("Database initialization failed");
        assert!(db_service.exists());

        let conn = db_service.get_connection().expect("Failed to get connection");
        let tables = vec![
            "groups",
            "passwords",
            "password_history",
            "user_settings",
            "secure_record_groups",
            "secure_records",
            "master_password",
        ];

        for table in tables {
            let count: i32 = conn.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?",
                [table],
                |row| row.get(0),
            ).expect(&format!("Failed to query table {}", table));
            
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }

    #[test]
    fn test_password_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_password.db");
        let db_service = DatabaseService::new(db_path.to_str().unwrap());
        db_service.initialize().unwrap();

        // 1. Add
        let password = crate::models::password::Password {
            id: None,
            title: "Test Password".to_string(),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            url: None,
            notes: None,
            group_id: None,
            created_at: None,
            updated_at: None,
            last_used_at: None,
            use_count: None,
            favorite: Some(false),
            tags: None,
        };
        let id = db_service.add_password(&password).unwrap();
        assert!(id > 0);

        // 2. Get
        let fetched = db_service.get_password(id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Password");
        assert_eq!(fetched.username, Some("user".to_string()));

        // 3. Update
        let mut update_pw = fetched.clone();
        update_pw.title = "Updated Title".to_string();
        db_service.update_password(&update_pw).unwrap();
        
        let updated = db_service.get_password(id).unwrap().unwrap();
        assert_eq!(updated.title, "Updated Title");

        // 4. Search
        let results = db_service.search_passwords("Updated").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, Some(id));

        // 5. Delete
        db_service.delete_password(id).unwrap();
        let deleted = db_service.get_password(id).unwrap();
        assert!(deleted.is_none());
    }
    #[test]
    fn test_group_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_group.db");
        let db_service = DatabaseService::new(db_path.to_str().unwrap());
        db_service.initialize().unwrap();

        // 1. Add
        let group = crate::models::group::Group {
            id: None,
            name: "Test Group".to_string(),
            parent_id: None,
            icon: None,
            color: None,
            sort_order: Some(1),
            created_at: None,
            updated_at: None,
        };
        let id = db_service.add_group(&group).unwrap();
        assert!(id > 0);

        // 2. Get
        let fetched = db_service.get_group(id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test Group");
        assert_eq!(fetched.sort_order, Some(1));

        // 3. Update
        let mut update_group = fetched.clone();
        update_group.name = "Updated Group".to_string();
        db_service.update_group(&update_group).unwrap();
        
        let updated = db_service.get_group(id).unwrap().unwrap();
        assert_eq!(updated.name, "Updated Group");

        // 4. List
        let groups = db_service.get_groups().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id, Some(id));

        // 5. Delete
        db_service.delete_group(id).unwrap();
        let deleted = db_service.get_group(id).unwrap();
        assert!(deleted.is_none());
    }
}
