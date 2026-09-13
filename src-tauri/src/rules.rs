use rusqlite::Connection;

use crate::models::{Rule, RuleInput};

pub fn list_rules(conn: &Connection) -> rusqlite::Result<Vec<Rule>> {
    let mut stmt = conn.prepare(
        "SELECT id, keyword, kind, l1, l2, priority, enabled, source FROM rules ORDER BY priority DESC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Rule {
            id: r.get(0)?,
            keyword: r.get(1)?,
            kind: r.get(2)?,
            l1: r.get(3)?,
            l2: r.get(4)?,
            priority: r.get(5)?,
            enabled: r.get::<_, i64>(6)? != 0,
            source: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// 规则引擎：在 merchant 与商品文本中查找关键词（不区分大小写），优先级高者优先。
/// 返回 (l1, l2)
pub fn classify(conn: &Connection, kind: &str, merchant: &str, goods: &str) -> Option<(String, Option<String>)> {
    let rules = list_rules(conn).ok()?;
    let merchant_lc = merchant.to_lowercase();
    let goods_lc = goods.to_lowercase();
    for rule in rules.iter().filter(|r| r.enabled && r.kind == kind) {
        let kw = rule.keyword.to_lowercase();
        if (kw.is_empty() || merchant_lc.contains(&kw) || goods_lc.contains(&kw)) && !kw.is_empty() {
            return Some((rule.l1.clone(), rule.l2.clone()));
        }
    }
    None
}

/// 商家是否已有任何启用规则覆盖（用于"新商家"判断）
pub fn merchant_known(conn: &Connection, merchant: &str) -> bool {
    let lc = merchant.to_lowercase();
    let mut stmt = match conn.prepare("SELECT keyword FROM rules WHERE enabled = 1") {
        Ok(s) => s,
        Err(_) => return false,
    };
    let keywords: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    keywords.iter().any(|kw| !kw.is_empty() && lc.contains(&kw.to_lowercase()))
}

pub fn upsert_rule(conn: &Connection, input: &RuleInput) -> rusqlite::Result<i64> {
    let prio = input.priority.unwrap_or(0);
    let enabled = input.enabled.unwrap_or(true);
    if let Some(id) = input.id {
        conn.execute(
            "UPDATE rules SET keyword=?1, kind=?2, l1=?3, l2=?4, priority=?5, enabled=?6 WHERE id=?7",
            rusqlite::params![input.keyword, input.kind, input.l1, input.l2, prio, enabled as i64, id],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO rules(keyword, kind, l1, l2, priority, enabled, source) VALUES (?1,?2,?3,?4,?5,?6,'preset')",
            rusqlite::params![input.keyword, input.kind, input.l1, input.l2, prio, enabled as i64],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// 学习一条新规则（新商家归类后调用）
pub fn learn_rule(conn: &Connection, kind: &str, merchant: &str, l1: &str, l2: Option<&str>) -> rusqlite::Result<()> {
    // 完全相同的关键词+分类则跳过
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM rules WHERE keyword=?1 AND kind=?2 AND l1=?3 AND IFNULL(l2,'')=IFNULL(?4,''))",
            rusqlite::params![merchant, kind, l1, l2],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if exists {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO rules(keyword, kind, l1, l2, priority, enabled, source) VALUES (?1,?2,?3,?4,0,1,'learned')",
        rusqlite::params![merchant, kind, l1, l2],
    )?;
    Ok(())
}
