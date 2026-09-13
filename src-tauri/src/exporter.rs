//! 数据导出：xlsx（按交易类型分 Sheet，列与随手记年度账本对齐）与 csv。

use rusqlite::Connection;
use rust_xlsxwriter::{Format, Workbook};

use crate::models::TxFilter;

const COLUMNS: &[&str] = &[
    "交易类型", "日期", "一级分类", "二级分类", "转出账户", "转入账户", "账户币种",
    "金额", "成员", "商家", "项目分类", "项目", "记账人", "备注", "交易单号",
];

fn fetch_filtered(conn: &Connection, filter: &TxFilter) -> Result<Vec<Vec<String>>, String> {
    let (where_sql, params) = crate::commands::build_where(filter);
    let sql = format!(
        "SELECT tx_type, tx_time, l1, l2, account_out, account_in, currency, amount, member, merchant, project_category, project, booker, remark, tx_no FROM transactions {where_sql} ORDER BY tx_time"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |r| {
            Ok(vec![
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                r.get::<_, Option<String>>(4)?.unwrap_or_default(),
                r.get::<_, Option<String>>(5)?.unwrap_or_default(),
                r.get::<_, Option<String>>(6)?.unwrap_or_default(),
                r.get::<_, f64>(7)?.to_string(),
                r.get::<_, Option<String>>(8)?.unwrap_or_default(),
                r.get::<_, Option<String>>(9)?.unwrap_or_default(),
                r.get::<_, Option<String>>(10)?.unwrap_or_default(),
                r.get::<_, Option<String>>(11)?.unwrap_or_default(),
                r.get::<_, Option<String>>(12)?.unwrap_or_default(),
                r.get::<_, Option<String>>(13)?.unwrap_or_default(),
                r.get::<_, Option<String>>(14)?.unwrap_or_default(),
            ])
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn export_xlsx(conn: &Connection, filter: &TxFilter, path: &str) -> Result<usize, String> {
    let rows = fetch_filtered(conn, filter)?;
    let mut workbook = Workbook::new();
    let header_fmt = Format::new().set_bold();
    let sheet_types = ["支出", "收入", "转账", "报销", "代付", "余额变更", "债权变更"];
    for st in sheet_types {
        let sheet_rows: Vec<&Vec<String>> = rows.iter().filter(|r| r[0] == st).collect();
        let ws = workbook.add_worksheet().set_name(st).map_err(|e| e.to_string())?;
        for (ci, name) in COLUMNS.iter().enumerate() {
            ws.write_string_with_format(0, ci as u16, *name, &header_fmt).map_err(|e| e.to_string())?;
        }
        ws.set_column_width(1, 20).map_err(|e| e.to_string())?;
        ws.set_column_width(13, 30).map_err(|e| e.to_string())?;
        for (ri, row) in sheet_rows.iter().enumerate() {
            let r = (ri + 1) as u32;
            for (ci, cell) in row.iter().enumerate() {
                if ci == 7 {
                    let v: f64 = cell.parse().unwrap_or(0.0);
                    let _ = ws.write_number(r, ci as u16, v);
                } else if !cell.is_empty() {
                    let _ = ws.write_string(r, ci as u16, cell);
                }
            }
        }
        let last_row = sheet_rows.len() as u32;
        if last_row > 0 {
            let _ = ws.autofilter(0, 0, last_row, (COLUMNS.len() - 1) as u16);
        }
    }
    workbook.save(path).map_err(|e| format!("保存失败: {e}"))?;
    Ok(rows.len())
}

pub fn export_csv(conn: &Connection, filter: &TxFilter, path: &str) -> Result<usize, String> {
    let rows = fetch_filtered(conn, filter)?;
    let mut out = String::from("\u{FEFF}"); // BOM 保证 Excel 打开不乱码
    out.push_str(&COLUMNS.join(","));
    out.push('\n');
    for row in &rows {
        let cells: Vec<String> = row
            .iter()
            .map(|c| {
                if c.contains(',') || c.contains('"') || c.contains('\n') {
                    format!("\"{}\"", c.replace('"', "\"\""))
                } else {
                    c.clone()
                }
            })
            .collect();
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    std::fs::write(path, out).map_err(|e| format!("保存失败: {e}"))?;
    Ok(rows.len())
}

/// 导出分类体系为缩进层级 txt
pub fn export_categories_txt(conn: &Connection, path: &str) -> Result<(), String> {
    let fetch = |kind: &str| -> Result<Vec<(Option<String>, Option<String>)>, String> {
        let mut stmt = conn
            .prepare("SELECT l1, l2 FROM categories WHERE kind = ?1 AND parent_id IS NOT NULL ORDER BY sort, id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![kind], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    };
    let mut out = String::new();
    for (title, kind) in [("支出", "expense"), ("收入", "income"), ("账户", "account")] {
        let rows = fetch(kind)?;
        if rows.is_empty() {
            continue;
        }
        out.push_str(&format!("{title}\n"));
        let mut last_l1 = String::new();
        for (l1, l2) in rows {
            let l1 = l1.unwrap_or_default();
            if l1 != last_l1 {
                out.push_str(&format!("  {l1}\n"));
                last_l1 = l1.clone();
            }
            if let Some(l2) = l2 {
                out.push_str(&format!("    {l2}\n"));
            }
        }
        out.push('\n');
    }
    std::fs::write(path, out).map_err(|e| format!("保存失败: {e}"))?;
    Ok(())
}

/// 导出分类体系为 Markdown
pub fn export_categories_md(conn: &Connection, path: &str) -> Result<(), String> {    let fetch = |kind: &str| -> Result<Vec<(Option<String>, Option<String>)>, String> {
        let mut stmt = conn
            .prepare("SELECT l1, l2 FROM categories WHERE kind = ?1 AND parent_id IS NOT NULL ORDER BY sort, id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![kind], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    };
    let mut md = String::from("# 账单分类体系\n\n");
    for (title, kind) in [("支出分类", "expense"), ("收入分类", "income"), ("账户分类", "account")] {
        let rows = fetch(kind)?;
        if rows.is_empty() {
            continue;
        }
        md.push_str(&format!("## {title}\n\n| 一级 | 二级 |\n|------|------|\n"));
        for (l1, l2) in rows {
            md.push_str(&format!("| {} | {} |\n", l1.unwrap_or_default(), l2.unwrap_or_else(|| "（留空）".into())));
        }
        md.push('\n');
    }
    std::fs::write(path, md).map_err(|e| format!("保存失败: {e}"))?;
    Ok(())
}
