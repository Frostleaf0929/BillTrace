//! 统计聚合：概览摘要 + 图表数据（柱状：时间轴；饼图：分类占比）

use chrono::{Datelike, NaiveDate};

use crate::models::{ChartPoint, PiePoint, SummaryStats};

/// 转账/报销/代付等不计入收支统计，仅支出/收入参与
pub fn summary(conn: &rusqlite::Connection, today: &str, month_prefix: &str) -> rusqlite::Result<SummaryStats> {
    let year_prefix = &today[..4];
    let q = |cond: &str, params: &[&dyn rusqlite::ToSql]| -> rusqlite::Result<f64> {
        let sql = format!(
            "SELECT IFNULL(SUM(amount),0) FROM transactions WHERE tx_type = ?1 AND {cond}"
        );
        conn.query_row(&sql, params, |r| r.get(0))
    };
    let today_expense = q("substr(tx_time,1,10) = ?2", &[&"支出", &today])?;
    let today_income = q("substr(tx_time,1,10) = ?2", &[&"收入", &today])?;
    let month_expense = q("substr(tx_time,1,7) = ?2", &[&"支出", &month_prefix])?;
    let month_income = q("substr(tx_time,1,7) = ?2", &[&"收入", &month_prefix])?;
    let year_expense = q("substr(tx_time,1,4) = ?2", &[&"支出", &year_prefix])?;
    let year_income = q("substr(tx_time,1,4) = ?2", &[&"收入", &year_prefix])?;
    let month_transfer: f64 = conn.query_row(
        "SELECT IFNULL(SUM(amount),0) FROM transactions WHERE tx_type = '转账' AND substr(tx_time,1,7) = ?1",
        rusqlite::params![month_prefix],
        |r| r.get(0),
    )?;
    let tx_count: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))?;
    Ok(SummaryStats { today_expense, today_income, month_expense, month_income, year_expense, year_income, month_transfer, tx_count })
}

/// 柱状图数据：dimension = day|month|year
pub fn chart_series(
    conn: &rusqlite::Connection,
    dimension: &str,
    date_from: &str,
    date_to: &str,
) -> rusqlite::Result<Vec<ChartPoint>> {
    let fmt_len = match dimension {
        "day" => 10,
        "month" => 7,
        _ => 4,
    };
    let mut stmt = conn.prepare(
        "SELECT substr(tx_time,1,?1) AS bucket, tx_type, SUM(amount)
         FROM transactions
         WHERE tx_time >= ?2 AND tx_time <= ?3 AND (tx_type = '支出' OR tx_type = '收入')
         GROUP BY bucket, tx_type ORDER BY bucket",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![fmt_len, format!("{date_from} 00:00:00"), format!("{date_to} 23:59:59")],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
            ))
        },
    )?;
    let mut map: std::collections::BTreeMap<String, ChartPoint> = std::collections::BTreeMap::new();
    for row in rows {
        let (bucket, tx_type, amount) = row?;
        let point = map.entry(bucket.clone()).or_insert_with(|| ChartPoint {
            label: bucket,
            income: 0.0,
            expense: 0.0,
        });
        match tx_type.as_str() {
            "收入" => point.income += amount,
            "支出" => point.expense += amount,
            _ => {}
        }
    }
    // 补齐空档（日视图最多补 366 个点，避免无限循环）
    if let (Some(first), Some(last)) = (map.keys().next().cloned(), map.keys().next_back().cloned()) {
        let parse = |s: &str| -> Option<NaiveDate> {
            if s.len() == 10 {
                NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
            } else if s.len() == 7 {
                NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d").ok()
            } else {
                NaiveDate::from_ymd_opt(s.parse().ok()?, 1, 1)
            }
        };
        let step_months = dimension == "month";
        let step_years = dimension == "year";
        if let (Some(mut cur), Some(end)) = (parse(&first), parse(&last)) {
            let mut guard = 0;
            while cur <= end && guard < 800 {
                let label = if step_years {
                    format!("{}", cur.year())
                } else if step_months {
                    format!("{:04}-{:02}", cur.year(), cur.month())
                } else {
                    format!("{:04}-{:02}-{:02}", cur.year(), cur.month(), cur.day())
                };
                map.entry(label.clone()).or_insert(ChartPoint { label, income: 0.0, expense: 0.0 });
                cur = if step_years {
                    NaiveDate::from_ymd_opt(cur.year() + 1, 1, 1).unwrap_or(end)
                } else if step_months {
                    let (y, m) = if cur.month() == 12 { (cur.year() + 1, 1) } else { (cur.year(), cur.month() + 1) };
                    NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(end)
                } else {
                    cur.succ_opt().unwrap_or(end)
                };
                guard += 1;
            }
        }
    }
    Ok(map.into_values().collect())
}

/// 饼图/排行数据：按指定维度汇总支出或收入
/// group_by: l1 | l2 | merchant | account | project | member
pub fn pie_series(
    conn: &rusqlite::Connection,
    tx_type: &str,
    date_from: &str,
    date_to: &str,
    group_by: &str,
) -> rusqlite::Result<Vec<PiePoint>> {
    let expr = match group_by {
        "l2" => "IFNULL(NULLIF(l2,''),'未分类')",
        "merchant" => "IFNULL(NULLIF(merchant,''),'（无商家）')",
        "account" => "IFNULL(NULLIF(account_out,''),IFNULL(NULLIF(account_in,''),'（无账户）'))",
        "project" => "IFNULL(NULLIF(project_category,''),'（无项目）')",
        "member" => "IFNULL(NULLIF(member,''),'（无成员）')",
        _ => "IFNULL(NULLIF(l1,''),'未分类')",
    };
    let sql = format!(
        "SELECT {expr} AS cat, SUM(amount)
         FROM transactions
         WHERE tx_type = ?1 AND tx_time >= ?2 AND tx_time <= ?3
         GROUP BY cat ORDER BY SUM(amount) DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params![tx_type, format!("{date_from} 00:00:00"), format!("{date_to} 23:59:59")],
        |r| Ok(PiePoint { name: r.get(0)?, value: r.get(1)? }),
    )?;
    rows.collect()
}

/// 账户余额：基数（手动设定）+ 全部交易净额
/// 规则：支出/报销从 account_out 扣；收入进 account_in；转账/代付双边；
/// 余额变更/债权变更按带符号金额直接计入出现的账户。
pub fn account_balances(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<crate::models::AccountBalance>> {
    use std::collections::HashMap;
    let mut meta: HashMap<String, String> = HashMap::new(); // name -> 一级账户
    {
        // 根分组 id→name；叶子账户归入父级分组，没有子级的一级账户视为独立账户
        let mut root_names: HashMap<i64, String> = HashMap::new();
        {
            let mut stmt = conn.prepare("SELECT id, name FROM categories WHERE kind='account' AND parent_id IS NULL")?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
            for row in rows {
                let (id, name) = row?;
                root_names.insert(id, name);
            }
        }
        let mut stmt = conn.prepare(
            "SELECT name, parent_id,
                    (SELECT COUNT(*) FROM categories c2 WHERE c2.parent_id = c.id) AS kids
             FROM categories c WHERE kind = 'account'",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (name, parent, kids) = row?;
            match parent.and_then(|pid| root_names.get(&pid).cloned()) {
                Some(group) => {
                    meta.insert(name, group);
                }
                None if kids == 0 => {
                    meta.insert(name, "未分组".into());
                }
                _ => {}
            }
        }
    }
    let mut net: HashMap<String, f64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT tx_type, account_out, account_in, amount FROM transactions")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, f64>(3)?,
            ))
        })?;
        for row in rows {
            let (tx_type, aout, ain, amount) = row?;
            match tx_type.as_str() {
                "支出" | "报销" => {
                    if let Some(a) = aout.filter(|s| !s.is_empty()) {
                        *net.entry(a).or_insert(0.0) -= amount;
                    }
                }
                "收入" => {
                    if let Some(a) = ain.filter(|s| !s.is_empty()) {
                        *net.entry(a).or_insert(0.0) += amount;
                    }
                }
                "转账" | "代付" => {
                    if let Some(a) = aout.filter(|s| !s.is_empty()) {
                        *net.entry(a).or_insert(0.0) -= amount;
                    }
                    if let Some(a) = ain.filter(|s| !s.is_empty()) {
                        *net.entry(a).or_insert(0.0) += amount;
                    }
                }
                "余额变更" | "债权变更" => {
                    if let Some(a) = ain.filter(|s| !s.is_empty()).or_else(|| aout.filter(|s| !s.is_empty())) {
                        *net.entry(a).or_insert(0.0) += amount;
                    }
                }
                _ => {}
            }
        }
    }
    let mut base: HashMap<String, f64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT name, balance FROM account_base")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))?;
        for row in rows {
            let (name, b) = row?;
            base.insert(name, b);
        }
    }
    let mut names: Vec<&String> = meta.keys().chain(net.keys()).collect();
    names.sort();
    names.dedup();
    let mut out: Vec<crate::models::AccountBalance> = vec![];
    for name in names {
        let group = meta.get(name).cloned().unwrap_or_else(|| "未分组".into());
        let b = base.get(name).copied().unwrap_or(0.0);
        let n = net.get(name).copied().unwrap_or(0.0);
        out.push(crate::models::AccountBalance {
            name: name.clone(),
            group,
            base: b,
            net: n,
            balance: b + n,
        });
    }
    Ok(out)
}

/// 分类页：每个一级分类的支出汇总（当月）
pub fn category_month_sums(
    conn: &rusqlite::Connection,
    kind: &str,
    month_prefix: &str,
) -> rusqlite::Result<std::collections::HashMap<String, f64>> {
    let tx_type = if kind == "income" { "收入" } else { "支出" };
    let mut stmt = conn.prepare(
        "SELECT l1, SUM(amount) FROM transactions
         WHERE tx_type = ?1 AND substr(tx_time,1,7) = ?2 GROUP BY l1",
    )?;
    let rows = stmt.query_map(rusqlite::params![tx_type, month_prefix], |r| {
        Ok((r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "未分类".into()), r.get::<_, f64>(1)?))
    })?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}
