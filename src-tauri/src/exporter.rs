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

/// 取某 kind 的（一级, 二级）映射对。
/// categories 是 parent_id 层级树：parent_id 为空即一级，其直接子节点即二级；
/// 预设导入体系只认两级，更深层级（小级）不参与导出；一级无子项时二级为 None。
fn category_pairs(conn: &Connection, kind: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, parent_id, name FROM categories WHERE kind = ?1 ORDER BY sort, id")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(i64, Option<i64>, String)> = stmt
        .query_map(rusqlite::params![kind], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let known: std::collections::HashSet<i64> = rows.iter().map(|(id, _, _)| *id).collect();
    let mut pairs: Vec<(String, Option<String>)> = Vec::new();
    for (id, parent, name) in &rows {
        // 只从顶层节点带出子项；父节点缺失的孤儿行按顶层处理，避免静默丢数据
        let is_top = match parent {
            None => true,
            Some(p) => !known.contains(p),
        };
        if !is_top {
            continue;
        }
        let children: Vec<String> = rows
            .iter()
            .filter(|(_, p, _)| p == &Some(*id))
            .map(|(_, _, n)| n.clone())
            .collect();
        if children.is_empty() {
            pairs.push((name.clone(), None));
        } else {
            for child in children {
                pairs.push((name.clone(), Some(child)));
            }
        }
    }
    Ok(pairs)
}

pub(crate) fn categories_txt_string(conn: &Connection) -> Result<String, String> {
    let mut out = String::new();
    for (label, kind) in [("支出", "expense"), ("收入", "income"), ("账户", "account")] {
        for (l1, l2) in category_pairs(conn, kind)? {
            // 不用缩进格式：一级名含"支出/收入/账户"（如"购物支出"）会被导入器误判为分区标记；
            // 路径行里的 "/" 会截断层级，换成全角
            let l1 = l1.replace('/', "／");
            match l2.map(|s| s.replace('/', "／")) {
                Some(l2) => out.push_str(&format!("{label} > {l1} > {l2}\n")),
                None => out.push_str(&format!("{label} > {l1}\n")),
            }
        }
    }
    Ok(out)
}

pub(crate) fn categories_md_string(conn: &Connection) -> Result<String, String> {
    let mut md = String::from("# 账单分类体系\n\n");
    for (title, kind) in [("支出分类", "expense"), ("收入分类", "income"), ("账户分类", "account")] {
        let pairs = category_pairs(conn, kind)?;
        if pairs.is_empty() {
            continue;
        }
        md.push_str(&format!("## {title}\n\n| 一级 | 二级 |\n|------|------|\n"));
        for (l1, l2) in pairs {
            // 名称含 "|" 会截断表格列，换成全角
            let l1 = l1.replace('|', "／");
            let l2 = l2.map(|s| s.replace('|', "／")).unwrap_or_else(|| "（留空）".into());
            md.push_str(&format!("| {l1} | {l2} |\n"));
        }
        md.push('\n');
    }
    Ok(md)
}

/// 导出分类体系为 txt（"支出 > 一级 > 二级" 路径行，可被预设导入原样解析）
pub fn export_categories_txt(conn: &Connection, path: &str) -> Result<(), String> {
    let out = categories_txt_string(conn)?;
    std::fs::write(path, out).map_err(|e| format!("保存失败: {e}"))
}

/// 导出分类体系为 Markdown（与预设导入同款表格，可被预设导入原样解析）
pub fn export_categories_md(conn: &Connection, path: &str) -> Result<(), String> {
    let md = categories_md_string(conn)?;
    std::fs::write(path, md).map_err(|e| format!("保存失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets;
    use rusqlite::Connection;

    /// 空库 + 手工插入的样例：支出两棵一级（一棵带二级）、一个三级"外卖"、账户一棵
    fn sample_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::init_db(&conn).unwrap();
        let ins = |parent: Option<i64>, name: &str, sort: i64| -> i64 {
            conn.execute(
                "INSERT INTO categories(kind, parent_id, name, sort) VALUES ('expense', ?1, ?2, ?3)",
                rusqlite::params![parent, name, sort],
            )
            .unwrap();
            conn.last_insert_rowid()
        };
        let food = ins(None, "食品饮料", 1);
        let lunch = ins(Some(food), "早午晚餐", 1);
        ins(Some(food), "饮料费(便利店)", 2);
        ins(Some(lunch), "外卖", 1); // 三级（小级）：预设体系只认两级，不应出现在导出里
        ins(None, "购物支出", 2);
        ins(None, "统计未确认", 3);
        conn.execute(
            "INSERT INTO categories(kind, parent_id, name, sort) VALUES ('account', NULL, '现金账户', 1)",
            [],
        )
        .unwrap();
        let cash = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO categories(kind, parent_id, name, sort) VALUES ('account', ?1, '现金', 1)",
            rusqlite::params![cash],
        )
        .unwrap();
        conn
    }

    fn as_tuples(preview: &crate::models::PresetPreview) -> Vec<(String, String, Option<String>)> {
        preview
            .categories
            .iter()
            .map(|c| (c.kind.clone(), c.l1.clone(), c.l2.clone()))
            .collect()
    }

    #[test]
    fn pairs_cover_two_levels_and_skip_deeper() {
        let conn = sample_db();
        assert_eq!(
            category_pairs(&conn, "expense").unwrap(),
            vec![
                ("食品饮料".to_string(), Some("早午晚餐".to_string())),
                ("食品饮料".to_string(), Some("饮料费(便利店)".to_string())),
                ("购物支出".to_string(), None),
                ("统计未确认".to_string(), None),
            ]
        );
    }

    #[test]
    fn md_export_round_trips_through_preset_parser() {
        let conn = sample_db();
        let md = categories_md_string(&conn).unwrap();
        assert!(!md.contains("外卖"), "三级分类不应被导出：{md}");
        let preview = presets::parse_md(&md);
        assert!(preview.warnings.is_empty(), "解析不应有告警：{:?}", preview.warnings);
        assert_eq!(
            as_tuples(&preview),
            vec![
                ("expense".into(), "食品饮料".into(), Some("早午晚餐".into())),
                ("expense".into(), "食品饮料".into(), Some("饮料费(便利店)".into())),
                ("expense".into(), "购物支出".into(), None),
                ("expense".into(), "统计未确认".into(), None),
                ("account".into(), "现金账户".into(), Some("现金".into())),
            ]
        );
    }

    #[test]
    fn txt_export_round_trips_through_preset_parser() {
        let conn = sample_db();
        let txt = categories_txt_string(&conn).unwrap();
        assert!(!txt.contains("外卖"), "三级分类不应被导出：{txt}");
        let preview = presets::parse_txt_file(&txt);
        assert!(preview.warnings.is_empty(), "解析不应有告警：{:?}", preview.warnings);
        assert_eq!(
            as_tuples(&preview),
            vec![
                ("expense".into(), "食品饮料".into(), Some("早午晚餐".into())),
                ("expense".into(), "食品饮料".into(), Some("饮料费(便利店)".into())),
                ("expense".into(), "购物支出".into(), None),
                ("expense".into(), "统计未确认".into(), None),
                ("account".into(), "现金账户".into(), Some("现金".into())),
            ]
        );
    }
}
