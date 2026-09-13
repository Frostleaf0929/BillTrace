use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub data_dir: PathBuf,
}

/// 解析便携数据目录：优先 exe 同级 data/，不可写则回退到系统 AppData。
pub fn resolve_data_dir(app: &tauri::AppHandle) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if let Some(dir) = exe_dir {
        let candidate = dir.join("data");
        if std::fs::create_dir_all(&candidate).is_ok() {
            let probe = candidate.join(".write_test");
            if std::fs::write(&probe, b"ok").is_ok() {
                let _ = std::fs::remove_file(&probe);
                return candidate;
            }
        }
    }
    let fallback = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            parent_id INTEGER,
            name TEXT NOT NULL,
            sort INTEGER NOT NULL DEFAULT 0,
            UNIQUE(kind, parent_id, name)
        );

        CREATE TABLE IF NOT EXISTS rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword TEXT NOT NULL,
            kind TEXT NOT NULL,
            l1 TEXT NOT NULL,
            l2 TEXT,
            priority INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            source TEXT NOT NULL DEFAULT 'preset'
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tx_type TEXT NOT NULL,
            tx_time TEXT NOT NULL,
            l1 TEXT,
            l2 TEXT,
            account_out TEXT,
            account_in TEXT,
            currency TEXT,
            amount REAL NOT NULL,
            member TEXT,
            merchant TEXT,
            project_category TEXT,
            project TEXT,
            booker TEXT,
            remark TEXT,
            tx_no TEXT,
            source TEXT,
            source_file TEXT,
            batch_id TEXT,
            fingerprint TEXT NOT NULL UNIQUE
        );
        CREATE INDEX IF NOT EXISTS idx_tx_time ON transactions(tx_time);
        CREATE INDEX IF NOT EXISTS idx_tx_type ON transactions(tx_type);
        CREATE INDEX IF NOT EXISTS idx_tx_l1 ON transactions(l1);
        CREATE INDEX IF NOT EXISTS idx_tx_merchant ON transactions(merchant);

        CREATE TABLE IF NOT EXISTS pending_rows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            merchant TEXT NOT NULL,
            tx_json TEXT NOT NULL,
            batch_id TEXT,
            fingerprint TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS account_base (
            name TEXT PRIMARY KEY,
            balance REAL NOT NULL DEFAULT 0,
            updated_at TEXT
        );

        INSERT OR IGNORE INTO settings(key, value) VALUES
            ('newMerchantPrompt', 'true'),
            ('fallbackCategory', '统计未确认'),
            ('theme', 'system');
        "#,
    )?;
    // 旧库升级：交易表补充第三级分类列
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN l3 TEXT", []);
    Ok(())
}

/// 预置初始账户分类（首次启动时）
pub fn seed_default_data(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let account_tree: &[(&str, &[&str])] = &[
        ("现金账户", &["现金"]),
        ("储蓄账户", &["银行卡"]),
        ("虚拟账户", &["支付宝余额", "微信钱包"]),
        ("负债账户", &["应付款项"]),
        ("债权账户", &["应收款项"]),
        ("信用账户", &["信用卡"]),
        ("投资账户", &["基金账户", "股票账户"]),
    ];
    for (sort, (l1, children)) in account_tree.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES ('account', NULL, ?1, ?2)",
            rusqlite::params![l1, sort as i64],
        )?;
        let pid = conn.last_insert_rowid();
        for (csort, child) in children.iter().enumerate() {
            conn.execute(
                "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES ('account', ?1, ?2, ?3)",
                rusqlite::params![pid, child, csort as i64],
            )?;
        }
    }
    // 支出/收入的一级分类骨架（详细二级由用户通过预设导入填入）
    let expense: &[&str] = &[
        "兴趣爱好", "食品饮料", "购物支出", "行车交通", "生活", "医疗保健", "人情往来",
        "金融保险", "统计未确认",
    ];
    for (i, name) in expense.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES ('expense', NULL, ?1, ?2)",
            rusqlite::params![name, i as i64],
        )?;
    }
    let income: &[&str] = &["职业收入", "其他收入"];
    for (i, name) in income.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES ('income', NULL, ?1, ?2)",
            rusqlite::params![name, i as i64],
        )?;
    }
    Ok(())
}
