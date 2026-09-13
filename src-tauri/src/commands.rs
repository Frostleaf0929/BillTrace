//! Tauri 命令层：前端 invoke 的所有入口

use std::fs;

use rusqlite::Connection;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::db::AppState;
use crate::importer::{self, now_batch_id};
use crate::models::*;
use crate::presets;
use crate::rules;
use crate::stats;
use crate::exporter;

type C<'a> = State<'a, AppState>;

fn with_conn<T>(state: &C, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, String> {
    let conn = state.conn.lock().map_err(|_| "数据库被占用")?;
    f(&conn).map_err(|e| e.to_string())
}

/// 在 rusqlite::Result 语境里构造业务错误
fn biz_err(msg: &str) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(msg.to_string().into())
}

// ---------------- 数据目录 / 设置 ----------------

#[tauri::command]
pub fn get_data_dir(state: C) -> String {
    state.data_dir.to_string_lossy().to_string()
}

#[tauri::command]
pub fn get_setting(state: C, key: String) -> Result<String, String> {
    with_conn(&state, |conn| {
        conn.query_row("SELECT value FROM settings WHERE key = ?1", rusqlite::params![key], |r| r.get(0))
    })
}

#[tauri::command]
pub fn set_setting(state: C, key: String, value: String) -> Result<(), String> {
    with_conn(&state, |conn| {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            rusqlite::params![key, value],
        )
        .map(|_| ())
    })
}

// ---------------- 分类 ----------------

#[tauri::command]
pub fn list_categories(state: C, kind: Option<String>) -> Result<Vec<Category>, String> {
    with_conn(&state, |conn| {
        let sql = match kind {
            Some(_) => "SELECT id, kind, parent_id, name, sort FROM categories WHERE kind = ?1 ORDER BY sort, id",
            None => "SELECT id, kind, parent_id, name, sort FROM categories ORDER BY kind, sort, id",
        };
        let mut stmt = conn.prepare(sql)?;
        let map = |r: &rusqlite::Row| Ok(Category { id: r.get(0)?, kind: r.get(1)?, parent_id: r.get(2)?, name: r.get(3)?, sort: r.get(4)? });
        match kind {
            Some(k) => stmt.query_map(rusqlite::params![k], map)?.collect(),
            None => stmt.query_map([], map)?.collect(),
        }
    })
}

/// 节点深度：根 = 0，最多允许到 2（第三级）
fn node_depth(conn: &Connection, mut id: i64) -> rusqlite::Result<i32> {
    let mut depth = 0;
    while let Some(pid) = conn
        .query_row("SELECT parent_id FROM categories WHERE id = ?1", rusqlite::params![id], |r| r.get::<_, Option<i64>>(0))?
    {
        depth += 1;
        id = pid;
        if depth > 10 {
            break; // 防御性：数据异常时终止
        }
    }
    Ok(depth)
}

#[tauri::command]
pub fn save_category(state: C, input: CategoryInput) -> Result<i64, String> {
    with_conn(&state, |conn| {
        if let Some(id) = input.id {
            conn.execute(
                "UPDATE categories SET name = ?1, sort = IFNULL(?2, sort) WHERE id = ?3",
                rusqlite::params![input.name, input.sort, id],
            )?;
            Ok(id)
        } else {
            // 新增子分类时校验层级与体系
            if let Some(pid) = input.parent_id {
                let parent_kind: String = conn
                    .query_row("SELECT kind FROM categories WHERE id = ?1", rusqlite::params![pid], |r| r.get(0))
                    .map_err(|_| biz_err("父分类不存在"))?;
                if parent_kind != input.kind {
                    return Err(biz_err("子分类必须与父分类属于同一体系（支出/收入/账户）"));
                }
                if node_depth(conn, pid)? >= 2 {
                    return Err(biz_err("最多支持三级分类（一级 → 二级 → 小级）"));
                }
            }
            conn.execute(
                "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES (?1, ?2, ?3, IFNULL(?4, 999))",
                rusqlite::params![input.kind, input.parent_id, input.name, input.sort],
            )?;
            let id = conn.last_insert_rowid();
            let changed: i64 = conn.query_row("SELECT changes()", [], |r| r.get(0))?;
            if changed == 0 {
                // 已存在同名，取现有 id
                conn.query_row(
                    "SELECT id FROM categories WHERE kind=?1 AND parent_id IS ?2 AND name=?3",
                    rusqlite::params![input.kind, input.parent_id, input.name],
                    |r| r.get(0),
                )
            } else {
                Ok(id)
            }
        }
    })
}

#[tauri::command]
pub fn delete_category(state: C, id: i64) -> Result<(), String> {
    with_conn(&state, |conn| {
        let has_children: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE parent_id = ?1)",
            rusqlite::params![id],
            |r| r.get(0),
        )?;
        if has_children {
            return Err(biz_err("该分类下存在子分类，请先删除或移动子分类"));
        }
        conn.execute("DELETE FROM categories WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    })
}

#[tauri::command]
pub fn move_category(state: C, id: i64, parent_id: Option<i64>, sort: i64) -> Result<(), String> {
    with_conn(&state, |conn| {
        if parent_id == Some(id) {
            return Err(biz_err("不能把自己设为父级"));
        }
        if let Some(pid) = parent_id {
            let pid_kind: Option<String> = conn
                .query_row("SELECT kind FROM categories WHERE id = ?1", rusqlite::params![pid], |r| r.get(0))
                .ok();
            let my_kind: Option<String> = conn
                .query_row("SELECT kind FROM categories WHERE id = ?1", rusqlite::params![id], |r| r.get(0))
                .ok();
            if pid_kind != my_kind {
                return Err(biz_err("只能移动到同一体系（支出/收入/账户）内"));
            }
            // 环检测：新父级的祖先链上不能出现自己
            let mut cur = Some(pid);
            let mut guard = 0;
            while let Some(cid) = cur {
                if cid == id {
                    return Err(biz_err("不能移动到自己的子分类下"));
                }
                cur = conn
                    .query_row("SELECT parent_id FROM categories WHERE id = ?1", rusqlite::params![cid], |r| r.get::<_, Option<i64>>(0))
                    .unwrap_or(None);
                guard += 1;
                if guard > 10 {
                    break;
                }
            }
            // 层级限制：最多三级 → 目标父级深度必须 ≤ 1
            let pdepth = node_depth(conn, pid)?;
            if pdepth >= 2 {
                return Err(biz_err("最多支持三级分类（一级 → 二级 → 小级）"));
            }
        }
        conn.execute(
            "UPDATE categories SET parent_id = ?1, sort = ?2 WHERE id = ?3",
            rusqlite::params![parent_id, sort, id],
        )?;
        Ok(())
    })
}

// ---------------- 规则 ----------------

#[tauri::command]
pub fn list_rules(state: C) -> Result<Vec<Rule>, String> {
    with_conn(&state, |conn| rules::list_rules(conn)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_rule(state: C, input: RuleInput) -> Result<i64, String> {
    with_conn(&state, |conn| rules::upsert_rule(conn, &input)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_rule(state: C, id: i64) -> Result<(), String> {
    with_conn(&state, |conn| conn.execute("DELETE FROM rules WHERE id = ?1", rusqlite::params![id])).map(|_| ())
}

// ---------------- 预设导入 ----------------

#[tauri::command]
pub fn preset_parse(_state: C, path: String) -> Result<(String, PresetPreview), String> {
    let text = fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
    let (format, preview) = presets::parse_auto(&text);
    Ok((format, preview))
}

#[tauri::command]
pub fn preset_apply(state: C, preview: PresetPreview) -> Result<(i64, i64), String> {
    let (cat_n, rule_n) = with_conn(&state, |conn| {
        let mut cat_n = 0i64;
        let mut rule_n = 0i64;
        for c in &preview.categories {
            // 确保一级存在
            let pid: i64 = match conn.query_row(
                "SELECT id FROM categories WHERE kind=?1 AND parent_id IS NULL AND name=?2",
                rusqlite::params![c.kind, c.l1],
                |r| r.get(0),
            ) {
                Ok(id) => id,
                Err(_) => {
                    let max_sort: i64 = conn
                        .query_row(
                            "SELECT IFNULL(MAX(sort),-1) FROM categories WHERE kind=?1 AND parent_id IS NULL",
                            rusqlite::params![c.kind],
                            |r| r.get(0),
                        )
                        .unwrap_or(-1);
                    conn.execute(
                        "INSERT INTO categories(kind, parent_id, name, sort) VALUES (?1, NULL, ?2, ?3)",
                        rusqlite::params![c.kind, c.l1, max_sort + 1],
                    )?;
                    conn.last_insert_rowid()
                }
            };
            if let Some(l2) = &c.l2 {
                let max_sort: i64 = conn
                    .query_row(
                        "SELECT IFNULL(MAX(sort),-1) FROM categories WHERE parent_id=?1",
                        rusqlite::params![pid],
                        |r| r.get(0),
                    )
                    .unwrap_or(-1);
                let n = conn.execute(
                    "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![c.kind, pid, l2, max_sort + 1],
                )?;
                cat_n += n as i64;
            } else {
                cat_n += 1; // 一级分类计入
            }
        }
        for r in &preview.rules {
            if rules::upsert_rule(conn, r).is_ok() {
                rule_n += 1;
            }
        }
        Ok((cat_n, rule_n))
    })?;
    Ok((cat_n, rule_n))
}

// ---------------- 导入账单 ----------------

#[tauri::command]
pub fn import_bill(state: C, path: String) -> Result<ImportReport, String> {
    let format = importer::detect_format(&path)?;
    let batch = now_batch_id();
    let prompt: bool = with_conn(&state, |conn| {
        Ok(conn
            .query_row("SELECT value FROM settings WHERE key='newMerchantPrompt'", [], |r| r.get::<_, String>(0))?
            == "true")
    })?;
    let fallback: String = with_conn(&state, |conn| {
        conn.query_row("SELECT value FROM settings WHERE key='fallbackCategory'", [], |r| r.get(0))
    })
    .unwrap_or_else(|_| "统计未确认".into());

    let mut conn = state.conn.lock().map_err(|_| "数据库被占用")?;
    let report = match format.as_str() {
        "suishouji" => importer::parse_suishouji(&path, &conn, &batch)?,
        "wechat" => importer::parse_wechat(&path, &mut conn, &batch, prompt, &fallback)?,
        _ => return Err("未知格式".into()),
    };
    Ok(report)
}

#[tauri::command]
pub fn list_pending(state: C) -> Result<Vec<PendingRow>, String> {
    let conn = state.conn.lock().map_err(|_| "数据库被占用")?;
    importer::list_pending(&conn)
}

#[tauri::command]
pub fn resolve_pending(state: C, assigns: Vec<MerchantAssign>, skip: Vec<String>) -> Result<(i64, i64), String> {
    let batch = now_batch_id();
    let mut conn = state.conn.lock().map_err(|_| "数据库被占用")?;
    importer::resolve_pending(&mut conn, &assigns, &skip, &batch)
}

// ---------------- 交易 CRUD ----------------

pub(crate) fn build_where(filter: &TxFilter) -> (String, Vec<String>) {
    let mut conds: Vec<String> = vec![];
    let mut params: Vec<String> = vec![];
    if let Some(v) = &filter.tx_type {
        if v != "全部" {
            conds.push(format!("tx_type = ?{}", params.len() + 1));
            params.push(v.clone());
        }
    }
    if let Some(v) = &filter.l1 {
        conds.push(format!("l1 = ?{}", params.len() + 1));
        params.push(v.clone());
    }
    if let Some(v) = &filter.l2 {
        conds.push(format!("l2 = ?{}", params.len() + 1));
        params.push(v.clone());
    }
    if let Some(v) = &filter.merchant {
        conds.push(format!("merchant LIKE ?{}", params.len() + 1));
        params.push(format!("%{v}%"));
    }
    if let Some(v) = &filter.keyword {
        conds.push(format!(
            "(merchant LIKE ?{p} OR remark LIKE ?{p} OR l1 LIKE ?{p} OR l2 LIKE ?{p} OR project LIKE ?{p} OR member LIKE ?{p} OR account_out LIKE ?{p} OR account_in LIKE ?{p})",
            p = params.len() + 1
        ));
        params.push(format!("%{v}%"));
    }
    if let Some(v) = &filter.date_from {
        conds.push(format!("tx_time >= ?{}", params.len() + 1));
        params.push(format!("{v} 00:00:00"));
    }
    if let Some(v) = &filter.date_to {
        conds.push(format!("tx_time <= ?{}", params.len() + 1));
        params.push(format!("{v} 23:59:59"));
    }
    if let Some(v) = filter.min_amount {
        conds.push(format!("amount >= CAST(?{} AS REAL)", params.len() + 1));
        params.push(format!("{v}"));
    }
    if let Some(v) = filter.max_amount {
        conds.push(format!("amount <= CAST(?{} AS REAL)", params.len() + 1));
        params.push(format!("{v}"));
    }
    let where_sql = if conds.is_empty() { String::new() } else { format!("WHERE {}", conds.join(" AND ")) };
    (where_sql, params)
}

#[tauri::command]
pub fn query_transactions(state: C, filter: TxFilter) -> Result<TxPage, String> {
    with_conn(&state, |conn| {
        let (where_sql, params) = build_where(&filter);
        let page_size = if filter.page_size <= 0 { 50 } else { filter.page_size };
        let page = if filter.page <= 0 { 1 } else { filter.page };
        let total: i64 = {
            let sql = format!("SELECT COUNT(*) FROM transactions {where_sql}");
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_row(rusqlite::params_from_iter(params.iter()), |r| r.get(0))?
        };
        let sql = format!(
            "SELECT id, tx_type, tx_time, l1, l2, l3, account_out, account_in, currency, amount, member, merchant, project_category, project, booker, remark, tx_no, source, source_file FROM transactions {where_sql} ORDER BY tx_time DESC, id DESC LIMIT {page_size} OFFSET {}",
            (page - 1) * page_size
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), row_to_tx)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TxPage { total, rows })
    })
}

fn row_to_tx(r: &rusqlite::Row) -> rusqlite::Result<Tx> {
    Ok(Tx {
        id: r.get(0)?,
        tx_type: r.get(1)?,
        tx_time: r.get(2)?,
        l1: r.get(3)?,
        l2: r.get(4)?,
        l3: r.get(5)?,
        account_out: r.get(6)?,
        account_in: r.get(7)?,
        currency: r.get(8)?,
        amount: r.get(9)?,
        member: r.get(10)?,
        merchant: r.get(11)?,
        project_category: r.get(12)?,
        project: r.get(13)?,
        booker: r.get(14)?,
        remark: r.get(15)?,
        tx_no: r.get(16)?,
        source: r.get(17)?,
        source_file: r.get(18)?,
    })
}

#[tauri::command]
pub fn save_transaction(state: C, tx: Tx) -> Result<i64, String> {
    let is_new = tx.id == 0;
    let batch = now_batch_id();
    if is_new {
        let tx2 = tx.clone();
        with_conn(&state, move |conn| {
            let mut tx = tx2;
            if tx.source.is_none() {
                tx.source = Some("manual".into());
            }
            let fp = {
                let no = tx.tx_no.clone().unwrap_or_default();
                if !no.is_empty() {
                    format!("manual|{no}|{batch}")
                } else {
                    format!(
                        "manual|{}|{}|{:.2}|{}|{}",
                        tx.tx_type, tx.tx_time, tx.amount,
                        tx.merchant.clone().unwrap_or_default(),
                        chrono::Local::now().timestamp_nanos_opt().unwrap_or_default()
                    )
                }
            };
            conn.execute(
                "INSERT INTO transactions(tx_type,tx_time,l1,l2,l3,account_out,account_in,currency,amount,member,merchant,project_category,project,booker,remark,tx_no,source,source_file,batch_id,fingerprint)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
                rusqlite::params![
                    tx.tx_type, tx.tx_time, tx.l1, tx.l2, tx.l3, tx.account_out, tx.account_in, tx.currency,
                    tx.amount, tx.member, tx.merchant, tx.project_category, tx.project, tx.booker,
                    tx.remark, tx.tx_no, tx.source, tx.source_file, batch, fp
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })
    } else {
        let tx2 = tx.clone();
        with_conn(&state, move |conn| {
            let tx = tx2;
            conn.execute(
                "UPDATE transactions SET tx_type=?1, tx_time=?2, l1=?3, l2=?4, l3=?5, account_out=?6, account_in=?7, currency=?8, amount=?9, member=?10, merchant=?11, project_category=?12, project=?13, booker=?14, remark=?15 WHERE id=?16",
                rusqlite::params![
                    tx.tx_type, tx.tx_time, tx.l1, tx.l2, tx.l3, tx.account_out, tx.account_in, tx.currency,
                    tx.amount, tx.member, tx.merchant, tx.project_category, tx.project, tx.booker,
                    tx.remark, tx.id
                ],
            )?;
            Ok(tx.id)
        })
    }
}

#[tauri::command]
pub fn delete_transaction(state: C, id: i64) -> Result<(), String> {
    with_conn(&state, |conn| conn.execute("DELETE FROM transactions WHERE id = ?1", rusqlite::params![id])).map(|_| ())
}

/// 批量删除
#[tauri::command]
pub fn delete_transactions(state: C, ids: Vec<i64>) -> Result<u64, String> {
    with_conn(&state, |conn| {
        let mut n = 0u64;
        for id in &ids {
            n += conn.execute("DELETE FROM transactions WHERE id = ?1", rusqlite::params![id])? as u64;
        }
        Ok(n)
    })
}

/// 批量改分类（只覆盖提供的层级，传 null 表示清除该层级）
#[tauri::command]
pub fn set_transactions_category(state: C, ids: Vec<i64>, l1: Option<String>, l2: Option<String>, l3: Option<String>) -> Result<u64, String> {
    with_conn(&state, |conn| {
        let mut n = 0u64;
        for id in &ids {
            n += conn.execute(
                "UPDATE transactions SET l1 = ?1, l2 = ?2, l3 = ?3 WHERE id = ?4",
                rusqlite::params![l1, l2, l3, id],
            )? as u64;
        }
        Ok(n)
    })
}

/// 清除软件记录的数据：tx = 仅账目；all = 账目+分类+规则（账户基数一并清除，恢复初始骨架）
/// 清除前强制 checkpoint 并自动备份到 data/backups/before-clear-*，防误操作
#[tauri::command]
pub fn clear_data(state: C, scope: String) -> Result<(), String> {
    if !matches!(scope.as_str(), "tx" | "all") {
        return Err("未知的数据清除范围".into());
    }
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    with_conn(&state, |conn| conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []).map(|_| ())).map_err(|e| e.to_string())?;
    let backup_dir = state.data_dir.join("backups").join(format!("before-clear-{scope}-{ts}"));
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let src = state.data_dir.join("zhangji.db");
    if src.exists() {
        fs::copy(&src, backup_dir.join("zhangji.db")).map_err(|e| format!("清除前备份失败: {e}"))?;
    }
    with_conn(&state, |conn| {
        if scope == "tx" {
            conn.execute("DELETE FROM transactions", [])?;
            conn.execute("DELETE FROM pending_rows", [])?;
        } else {
            conn.execute("DELETE FROM transactions", [])?;
            conn.execute("DELETE FROM pending_rows", [])?;
            conn.execute("DELETE FROM categories", [])?;
            conn.execute("DELETE FROM rules", [])?;
            conn.execute("DELETE FROM account_base", [])?;
            crate::db::seed_default_data(conn)?;
        }
        Ok(())
    })
}

// ---------------- 背景图 ----------------

const BG_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif"];

#[tauri::command]
pub fn import_background(state: C, path: String) -> Result<String, String> {
    let src = std::path::Path::new(&path);
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("无法识别文件类型")?;
    if !BG_EXTS.contains(&ext.as_str()) {
        return Err("仅支持 png / jpg / webp / bmp / gif 图片".into());
    }
    for e in BG_EXTS {
        let _ = fs::remove_file(state.data_dir.join(format!("background.{e}")));
    }
    let dest = state.data_dir.join(format!("background.{ext}"));
    fs::copy(src, &dest).map_err(|e| format!("复制背景图失败: {e}"))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_background(state: C) -> Result<Option<String>, String> {
    for e in BG_EXTS {
        let p = state.data_dir.join(format!("background.{e}"));
        if p.exists() {
            return Ok(Some(p.to_string_lossy().to_string()));
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn clear_background(state: C) -> Result<(), String> {
    for e in BG_EXTS {
        let _ = fs::remove_file(state.data_dir.join(format!("background.{e}")));
    }
    Ok(())
}

// ---------------- 统计 ----------------

#[tauri::command]
pub fn stats_summary(state: C) -> Result<SummaryStats, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let month = chrono::Local::now().format("%Y-%m").to_string();
    with_conn(&state, |conn| stats::summary(conn, &today, &month)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stats_chart(state: C, dimension: String, date_from: String, date_to: String) -> Result<Vec<ChartPoint>, String> {
    with_conn(&state, |conn| stats::chart_series(conn, &dimension, &date_from, &date_to)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stats_pie(state: C, tx_type: String, date_from: String, date_to: String, group_by: Option<String>) -> Result<Vec<PiePoint>, String> {
    let gb = group_by.as_deref().unwrap_or("l1");
    with_conn(&state, |conn| stats::pie_series(conn, &tx_type, &date_from, &date_to, gb)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn account_balances(state: C) -> Result<Vec<AccountBalance>, String> {
    with_conn(&state, |conn| stats::account_balances(conn)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_account_base(state: C, name: String, balance: f64) -> Result<(), String> {
    with_conn(&state, |conn| {
        conn.execute(
            "INSERT INTO account_base(name, balance, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET balance = ?2, updated_at = ?3",
            rusqlite::params![name, balance, chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()],
        )
        .map(|_| ())
    })
}

#[tauri::command]
pub fn category_sums(state: C, kind: String, month_prefix: String) -> Result<std::collections::HashMap<String, f64>, String> {
    with_conn(&state, |conn| stats::category_month_sums(conn, &kind, &month_prefix)).map_err(|e| e.to_string())
}

// ---------------- 导出 / 备份 ----------------

#[tauri::command]
pub fn export_data(state: C, app: AppHandle, format: String, filter: TxFilter) -> Result<usize, String> {
    let ext = if format == "csv" { "csv" } else { "xlsx" };
    let default_name = format!("账迹导出_{}.{}", chrono::Local::now().format("%Y%m%d_%H%M%S"), ext);
    let path = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("表格文件", &[ext])
        .blocking_save_file()
        .ok_or("已取消")?
        .into_path()
        .map_err(|e| e.to_string())?;
    let path = path.to_string_lossy().to_string();
    let conn = state.conn.lock().map_err(|_| "数据库被占用")?;
    if format == "csv" {
        exporter::export_csv(&conn, &filter, &path)
    } else {
        exporter::export_xlsx(&conn, &filter, &path)
    }
}

#[tauri::command]
pub fn export_categories(state: C, app: AppHandle, format: Option<String>) -> Result<(), String> {
    let fmt = format.unwrap_or_else(|| "md".into());
    let ext = if fmt == "txt" { "txt" } else { "md" };
    let default_name = format!("分类体系_{}.{}", chrono::Local::now().format("%Y%m%d"), ext);
    let path = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("分类预设", &[ext])
        .blocking_save_file()
        .ok_or("已取消")?
        .into_path()
        .map_err(|e| e.to_string())?;
    let conn = state.conn.lock().map_err(|_| "数据库被占用")?;
    if fmt == "txt" {
        exporter::export_categories_txt(&conn, &path.to_string_lossy())
    } else {
        exporter::export_categories_md(&conn, &path.to_string_lossy())
    }
}

/// 图表导出：前端传 base64 PNG，弹出保存对话框
#[tauri::command]
pub fn export_image(app: AppHandle, name: String, data_base64: String) -> Result<String, String> {
    use base64::Engine as _;
    let default_name = format!("{}_{}.png", name, chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let path = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("PNG 图片", &["png"])
        .blocking_save_file()
        .ok_or("已取消")?
        .into_path()
        .map_err(|e| e.to_string())?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64)
        .map_err(|e| format!("图片数据无效: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("保存失败: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn backup_now(state: C) -> Result<String, String> {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_dir = state.data_dir.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let target = backup_dir.join(format!("backup_{ts}"));
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    // WAL 模式下先 checkpoint 保证数据落盘
    with_conn(&state, |conn| conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])).map(|_| ())?;
    for f in ["zhangji.db", "zhangji.db-wal", "zhangji.db-shm"] {
        let src = state.data_dir.join(f);
        if src.exists() {
            fs::copy(&src, target.join(f)).map_err(|e| e.to_string())?;
        }
    }
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
