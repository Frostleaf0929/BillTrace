//! 账单导入：随手记年度 Excel（多 Sheet）与微信官方流水 xlsx。
//! 流程：解析 → 规则引擎分类 → 指纹查重 → 入库；无规则的新商家进入待确认队列。

use calamine::{Data, Reader, Xlsx};
use chrono::{DateTime, NaiveDate, NaiveDateTime};

use crate::models::{ImportReport, MerchantAssign, PendingRow, SheetStat, Tx};
use crate::rules;

/// Excel 日期序列号 → NaiveDateTime
fn serial_to_datetime(serial: f64) -> Option<NaiveDateTime> {
    let base = NaiveDate::from_ymd_opt(1899, 12, 30)?.and_hms_opt(0, 0, 0)?;
    Some(base + chrono::Duration::seconds((serial * 86400.0).round() as i64))
}

fn data_to_string(d: &Data) -> String {
    match d {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::DateTime(dt) => serial_to_datetime(dt.as_f64())
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
        Data::Error(_) => String::new(),
        _ => String::new(),
    }
}

/// 把单元格尽力解析为 "YYYY-MM-DD HH:MM:SS"
fn parse_time(d: &Data) -> Option<String> {
    match d {
        Data::DateTime(dt) => {
            serial_to_datetime(dt.as_f64()).map(|ndt| ndt.format("%Y-%m-%d %H:%M:%S").to_string())
        }
        Data::String(s) => {
            let s = s.trim();
            for fmt in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y/%m/%d %H:%M:%S", "%Y/%m/%d %H:%M", "%Y-%m-%d", "%Y/%m/%d"] {
                if let Ok(nd) = NaiveDateTime::parse_from_str(s, fmt) {
                    return Some(nd.format("%Y-%m-%d %H:%M:%S").to_string());
                }
                if let Ok(nd) = NaiveDate::parse_from_str(s, fmt) {
                    return Some(nd.format("%Y-%m-%d 00:00:00").to_string());
                }
            }
            None
        }
        Data::Float(f) => {
            // Excel 日期序列号（无 Data::DateTime 类型时）
            let serial = *f;
            if (40000.0..60000.0).contains(&serial) {
                let nd = NaiveDate::from_ymd_opt(1899, 12, 30)?
                    .and_hms_opt(0, 0, 0)?
                    + chrono::Duration::seconds((serial * 86400.0) as i64);
                return Some(nd.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            None
        }
        _ => None,
    }
}

fn parse_amount(s: &str) -> Option<f64> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let cleaned = cleaned.trim_start_matches(['¥', '￥', '$', '€']);
    if cleaned.is_empty() || cleaned == "/" || cleaned.starts_with('#') {
        return None;
    }
    cleaned.replace(',', "").parse::<f64>().ok()
}

fn fingerprint(tx: &Tx) -> String {
    if let Some(no) = &tx.tx_no {
        if !no.is_empty() {
            return format!("wx|{}", no);
        }
    }
    format!(
        "ss|{}|{}|{:.2}|{}|{}|{}",
        tx.tx_type,
        tx.tx_time,
        tx.amount,
        tx.merchant.clone().unwrap_or_default(),
        tx.account_out.clone().unwrap_or_default(),
        tx.account_in.clone().unwrap_or_default()
    )
}

pub fn detect_format(path: &str) -> Result<String, String> {
    let mut wb: Xlsx<_> = calamine::open_workbook(path).map_err(|e| format!("无法打开文件: {e}"))?;
    let names = wb.sheet_names().to_vec();
    let joined = names.join("|");
    if ["支出", "收入", "转账"].iter().all(|s| joined.contains(s)) {
        return Ok("suishouji".into());
    }
    // 微信流水：通常单 Sheet，前几行含 "微信支付账单明细"
    for name in names.iter().take(3) {
        if let Ok(range) = wb.worksheet_range(name) {
            for row in range.rows().take(20) {
                let row_text = row.iter().map(data_to_string).collect::<String>();
                if row_text.contains("微信支付账单") || row_text.contains("交易单号") && row_text.contains("交易时间") {
                    return Ok("wechat".into());
                }
            }
        }
    }
    if joined.contains("交易") || joined.contains("微信") {
        return Ok("wechat".into());
    }
    Err("无法识别文件格式（支持年度账本多 Sheet Excel 与微信支付流水 xlsx）".into())
}

// ---------------- 随手记年度 Excel ----------------

const TX_TYPES: &[&str] = &["支出", "收入", "转账", "报销", "代付", "余额变更", "债权变更"];

fn header_index(headers: &[Data], candidates: &[&str]) -> Option<usize> {
    for cand in candidates {
        for (i, h) in headers.iter().enumerate() {
            let s = data_to_string(h);
            if s == *cand {
                return Some(i);
            }
        }
    }
    // 模糊匹配
    for cand in candidates {
        for (i, h) in headers.iter().enumerate() {
            let s = data_to_string(h);
            if !s.is_empty() && (s.contains(cand) || cand.contains(s.as_str())) && s.chars().count() >= 2 {
                return Some(i);
            }
        }
    }
    None
}

pub fn parse_suishouji(path: &str, conn: &rusqlite::Connection, batch_id: &str) -> Result<ImportReport, String> {
    let mut wb: Xlsx<_> = calamine::open_workbook(path).map_err(|e| format!("无法打开文件: {e}"))?;
    let mut report = ImportReport {
        file: path.to_string(),
        detected_format: "suishouji".into(),
        batch_id: batch_id.to_string(),
        ..Default::default()
    };

    for sheet_name in wb.sheet_names().to_vec() {
        let tx_type = match TX_TYPES.iter().find(|t| sheet_name.contains(**t)) {
            Some(t) => t.to_string(),
            None => continue,
        };
        let range = wb
            .worksheet_range(&sheet_name)
            .map_err(|e| format!("读取 Sheet [{sheet_name}] 失败: {e}"))?;
        let mut rows_iter = range.rows();
        let mut header_idx: Vec<Option<usize>> = vec![None; 14];
        // 字段槽位: 0日期 1一级 2二级 3转出 4转入 5币种 6金额 7成员 8商家 9项目分类 10项目 11记账人 12备注
        let mut sheet_rows = 0i64;
        let mut inserted = 0i64;
        let mut duplicates = 0i64;

        for row in rows_iter.by_ref() {
            let non_empty: Vec<usize> = (0..row.len()).filter(|i| !data_to_string(&row[*i]).is_empty()).collect();
            if non_empty.len() < 3 {
                continue;
            }
            // 找表头行（含"日期"与"金额"）
            let texts: Vec<String> = row.iter().map(data_to_string).collect();
            if texts.iter().any(|t| t == "日期") && texts.iter().any(|t| t.contains("金额")) {
                header_idx[0] = header_index(&row, &["日期", "交易时间"]);
                header_idx[1] = header_index(&row, &["一级分类", "分类"]);
                header_idx[2] = header_index(&row, &["二级分类"]);
                header_idx[3] = header_index(&row, &["转出账户", "支出账户", "账户"]);
                header_idx[4] = header_index(&row, &["转入账户", "收入账户"]);
                header_idx[5] = header_index(&row, &["账户币种", "币种"]);
                header_idx[6] = header_index(&row, &["金额"]);
                header_idx[7] = header_index(&row, &["成员"]);
                header_idx[8] = header_index(&row, &["商家"]);
                header_idx[9] = header_index(&row, &["项目分类"]);
                header_idx[10] = header_index(&row, &["项目"]);
                header_idx[11] = header_index(&row, &["记账人"]);
                header_idx[12] = header_index(&row, &["备注"]);
                break;
            }
        }
        if header_idx[0].is_none() || header_idx[6].is_none() {
            report.skipped += 1;
            report.sheet_stats.push(SheetStat { sheet: sheet_name, rows: 0, inserted: 0, duplicates: 0 });
            continue;
        }
        let g = |row: &[Data], slot: usize| -> String {
            header_idx[slot].and_then(|i| row.get(i)).map(data_to_string).unwrap_or_default()
        };
        for row in rows_iter {
            let time = header_idx[0].and_then(|i| row.get(i)).and_then(parse_time);
            let amount = header_idx[6].and_then(|i| row.get(i)).map(data_to_string);
            let Some(time) = time else {
                if row.iter().all(|c| data_to_string(c).is_empty()) {
                    continue;
                }
                report.skipped += 1;
                continue;
            };
            let Some(amount) = amount.and_then(|s| parse_amount(&s)) else {
                report.skipped += 1;
                continue;
            };
            sheet_rows += 1;
            let tx = Tx {
                id: 0,
                tx_type: tx_type.clone(),
                tx_time: time,
                l1: opt_nonempty(g(&row, 1)),
                l2: opt_nonempty(g(&row, 2)),
                l3: None,
                account_out: opt_nonempty(g(&row, 3)),
                account_in: opt_nonempty(g(&row, 4)),
                currency: some_or_default(g(&row, 5), "CNY"),
                amount,
                member: opt_nonempty(g(&row, 7)),
                merchant: opt_nonempty(g(&row, 8)),
                project_category: opt_nonempty(g(&row, 9)),
                project: opt_nonempty(g(&row, 10)),
                booker: opt_nonempty(g(&row, 11)),
                remark: opt_nonempty(g(&row, 12)),
                tx_no: None,
                source: Some("import:suishouji".into()),
                source_file: Some(path.to_string()),
            };
            match insert_tx(conn, &tx, batch_id) {
                Ok(true) => inserted += 1,
                Ok(false) => duplicates += 1,
                Err(_) => report.skipped += 1,
            }
        }
        report.total_rows += sheet_rows;
        report.inserted += inserted;
        report.duplicates += duplicates;
        report.sheet_stats.push(SheetStat { sheet: sheet_name, rows: sheet_rows, inserted, duplicates });
    }
    Ok(report)
}

fn opt_nonempty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}
fn some_or_default(s: String, default: &str) -> Option<String> {
    Some(if s.is_empty() { default.to_string() } else { s })
}

// ---------------- 微信流水 ----------------

pub fn parse_wechat(path: &str, conn: &mut rusqlite::Connection, batch_id: &str, new_merchant_prompt: bool, fallback: &str) -> Result<ImportReport, String> {
    let mut wb: Xlsx<_> = calamine::open_workbook(path).map_err(|e| format!("无法打开文件: {e}"))?;
    let mut report = ImportReport {
        file: path.to_string(),
        detected_format: "wechat".into(),
        batch_id: batch_id.to_string(),
        ..Default::default()
    };
    let sheet_name = wb.sheet_names().first().cloned().ok_or("文件为空")?;
    let range = wb.worksheet_range(&sheet_name).map_err(|e| format!("读取失败: {e}"))?;

    // 找表头行：包含 交易时间 与 交易对方（或 收/支）
    let mut header_row_idx = None;
    for (ri, row) in range.rows().enumerate().take(30) {
        let texts: Vec<String> = row.iter().map(data_to_string).collect();
        if texts.iter().any(|t| t == "交易时间") && (texts.iter().any(|t| t.contains("交易对方")) || texts.iter().any(|t| t.contains("收/支"))) {
            header_row_idx = Some(ri);
            break;
        }
    }
    let Some(hri) = header_row_idx else {
        return Err("未找到微信账单表头（需包含「交易时间」「交易对方」）".into());
    };
    let headers: Vec<String> = range.rows().nth(hri).unwrap().iter().map(data_to_string).collect();
    let col = |names: &[&str]| -> Option<usize> {
        for n in names {
            if let Some(i) = headers.iter().position(|h| h == n) {
                return Some(i);
            }
        }
        for n in names {
            if let Some(i) = headers.iter().position(|h| h.contains(n)) {
                return Some(i);
            }
        }
        None
    };
    let c_time = col(&["交易时间"]).ok_or("缺少「交易时间」列")?;
    let c_counterparty = col(&["交易对方"]).ok_or("缺少「交易对方」列")?;
    let c_goods = col(&["商品"]);
    let c_dir = col(&["收/支"]).ok_or("缺少「收/支」列")?;
    let c_amount = col(&["金额(元)", "金额"]).ok_or("缺少「金额」列")?;
    let c_pay = col(&["支付方式"]);
    let c_status = col(&["当前状态"]);
    let c_txno = col(&["交易单号"]);
    let c_remark = col(&["备注"]);

    let rows: Vec<Vec<Data>> = range.rows().skip(hri + 1).map(|r| r.to_vec()).collect();
    for row in &rows {
        let get = |i: Option<usize>| -> String { i.and_then(|i| row.get(i)).map(data_to_string).unwrap_or_default() };
        let Some(time) = row.get(c_time).and_then(parse_time) else {
            if row.iter().all(|c| data_to_string(c).is_empty()) {
                continue;
            }
            report.skipped += 1;
            continue;
        };
        let dir = get(Some(c_dir));
        let tx_type = match dir.as_str() {
            "支出" => "支出",
            "收入" => "收入",
            _ => {
                // 中性交易（充值/理财/零钱提现等）
                report.skipped += 1;
                continue;
            }
        };
        let Some(amount) = parse_amount(&get(Some(c_amount))) else {
            report.skipped += 1;
            continue;
        };
        report.total_rows += 1;
        let merchant = get(Some(c_counterparty));
        let goods = get(c_goods);
        let kind = if tx_type == "收入" { "income" } else { "expense" };
        let account = opt_nonempty(get(c_pay));
        let status = get(c_status);
        let mut remark = get(c_remark);
        if remark.is_empty() && !goods.is_empty() {
            remark = goods.clone();
        }
        if !status.is_empty() && status != "支付成功" && status != "已存入零钱" && status != "已收款" {
            remark = if remark.is_empty() { format!("({status})") } else { format!("{remark} ({status})") };
        }

        let mut tx = Tx {
            id: 0,
            tx_type: tx_type.to_string(),
            tx_time: time,
            l1: None,
            l2: None,
            l3: None,
            account_out: if tx_type == "支出" { account.clone() } else { None },
            account_in: if tx_type == "收入" { account.clone() } else { None },
            currency: Some("CNY".into()),
            amount,
            member: None,
            merchant: opt_nonempty(merchant.clone()),
            project_category: None,
            project: None,
            booker: None,
            remark: opt_nonempty(remark),
            tx_no: opt_nonempty(get(c_txno)),
            source: Some("import:wechat".into()),
            source_file: Some(path.to_string()),
        };

        if let Some((l1, l2)) = rules::classify(conn, kind, &merchant, &goods) {
            tx.l1 = Some(l1);
            tx.l2 = l2;
            match insert_tx(conn, &tx, batch_id) {
                Ok(true) => report.inserted += 1,
                Ok(false) => report.duplicates += 1,
                Err(_) => report.skipped += 1,
            }
        } else if new_merchant_prompt && !merchant.is_empty() && !rules::merchant_known(conn, &merchant) {
            // 新商家 → 待确认队列
            let fp = fingerprint(&tx);
            let n = conn.execute(
                "INSERT OR IGNORE INTO pending_rows(merchant, tx_json, batch_id, fingerprint) VALUES (?1,?2,?3,?4)",
                rusqlite::params![merchant, serde_json::to_string(&tx).unwrap_or_default(), batch_id, fp],
            );
            match n {
                Ok(count) if count > 0 => {
                    report.pending += 1;
                    if !report.pending_merchants.contains(&merchant) {
                        report.pending_merchants.push(merchant);
                    }
                }
                _ => report.duplicates += 1,
            }
        } else {
            // 兜底分类
            tx.l1 = Some(fallback.to_string());
            match insert_tx(conn, &tx, batch_id) {
                Ok(true) => report.inserted += 1,
                Ok(false) => report.duplicates += 1,
                Err(_) => report.skipped += 1,
            }
        }
    }
    Ok(report)
}

// ---------------- 入库与待确认 ----------------

/// 返回 true=新增，false=重复
pub fn insert_tx(conn: &rusqlite::Connection, tx: &Tx, batch_id: &str) -> Result<bool, rusqlite::Error> {
    let fp = fingerprint(tx);
    let n = conn.execute(
        "INSERT OR IGNORE INTO transactions(tx_type,tx_time,l1,l2,l3,account_out,account_in,currency,amount,member,merchant,project_category,project,booker,remark,tx_no,source,source_file,batch_id,fingerprint)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
        rusqlite::params![
            tx.tx_type, tx.tx_time, tx.l1, tx.l2, tx.l3, tx.account_out, tx.account_in, tx.currency,
            tx.amount, tx.member, tx.merchant, tx.project_category, tx.project, tx.booker,
            tx.remark, tx.tx_no, tx.source, tx.source_file, batch_id, fp
        ],
    )?;
    Ok(n > 0)
}

pub fn list_pending(conn: &rusqlite::Connection) -> Result<Vec<PendingRow>, String> {
    let mut stmt = conn
        .prepare("SELECT id, merchant, tx_json FROM pending_rows ORDER BY merchant, id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })
        .map_err(|e| e.to_string())?;
    let mut out = vec![];
    for row in rows {
        let (id, merchant, json) = row.map_err(|e| e.to_string())?;
        let tx: Tx = serde_json::from_str(&json).map_err(|e| format!("待确认数据损坏: {e}"))?;
        let _ = merchant;
        out.push(PendingRow { id, tx });
    }
    Ok(out)
}

/// 按商家处理待确认：确认分类（并学习规则）或跳过
pub fn resolve_pending(conn: &mut rusqlite::Connection, assigns: &[MerchantAssign], skip: &[String], batch_id: &str) -> Result<(i64, i64), String> {
    let mut confirmed = 0i64;
    let skipped = 0i64;
    for m in skip {
        conn.execute("DELETE FROM pending_rows WHERE merchant = ?1", rusqlite::params![m])
            .map_err(|e| e.to_string())?;
    }
    for a in assigns {
        let kind = if a.l1.contains("收入") { "income" } else { "expense" };
        let rows: Vec<(i64, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, tx_json FROM pending_rows WHERE merchant = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(rusqlite::params![a.merchant], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for (id, json) in rows {
            let mut tx: Tx = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            tx.l1 = Some(a.l1.clone());
            tx.l2 = a.l2.clone();
            let fp = fingerprint(&tx);
            let n = conn
                .execute(
                    "INSERT OR IGNORE INTO transactions(tx_type,tx_time,l1,l2,l3,account_out,account_in,currency,amount,member,merchant,project_category,project,booker,remark,tx_no,source,source_file,batch_id,fingerprint)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
                    rusqlite::params![
                        tx.tx_type, tx.tx_time, tx.l1, tx.l2, tx.l3, tx.account_out, tx.account_in, tx.currency,
                        tx.amount, tx.member, tx.merchant, tx.project_category, tx.project, tx.booker,
                        tx.remark, tx.tx_no, tx.source, tx.source_file, batch_id, fp
                    ],
                )
                .map_err(|e| e.to_string())?;
            confirmed += n as i64;
            let _ = conn.execute("DELETE FROM pending_rows WHERE id = ?1", rusqlite::params![id]);
        }
        let _ = rules::learn_rule(conn, kind, &a.merchant, &a.l1, a.l2.as_deref());
    }
    Ok((confirmed, skipped))
}

/// 给定商家样本时间（用于导入测试）
pub fn now_batch_id() -> String {
    let now: DateTime<chrono::Local> = chrono::Local::now();
    now.format("%Y%m%d%H%M%S%3f").to_string()
}
