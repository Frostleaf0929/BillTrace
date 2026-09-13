//! 分类预设解析：支持 Markdown（标题分节 + 表格，如用户的《分类记录_调整.md》）
//! 与 txt（缩进或斜杠层级）。解析结果先返回预览，用户确认后才入库。

use crate::models::{PresetCategory, PresetPreview, RuleInput};

/// 去掉名称里的序号标记与空白："(1) 兴趣爱好" → "兴趣爱好"，"杂费（5）" → "杂费"
fn clean_name(s: &str) -> String {
    let mut out = s.trim().to_string();
    // 去前缀序号：(1) （1） 1. 1、
    loop {
        let trimmed = out.trim_start();
        let bytes = trimmed.as_bytes();
        if bytes.is_empty() {
            break;
        }
        let starts_digit = bytes[0].is_ascii_digit();
        let starts_paren = trimmed.starts_with('(') || trimmed.starts_with('（');
        if starts_digit || starts_paren {
            let end = trimmed
                .char_indices()
                .find(|(_, c)| *c == ')' || *c == '）')
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or_else(|| {
                    if starts_digit {
                        trimmed.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '、')).unwrap_or(trimmed.len())
                    } else {
                        0
                    }
                });
            if end > 0 {
                let candidate = trimmed[end..].trim().to_string();
                if !candidate.is_empty() {
                    out = candidate;
                    continue;
                }
            }
        }
        break;
    }
    // 去后缀序号：兴趣爱好（1）/ 杂费(5)
    loop {
        let chars: Vec<char> = out.chars().collect();
        if chars.len() > 2 && (chars[chars.len() - 1] == ')' || chars[chars.len() - 1] == '）') {
            // 找对应的左括号
            let open_char = if chars[chars.len() - 1] == ')' { '(' } else { '（' };
            if let Some(open_idx) = out.rfind(open_char) {
                let open_char_i = out[..open_idx].chars().count();
                let inner: String = chars[open_char_i + 1..chars.len() - 1].iter().collect();
                let inner_trim = inner.trim();
                let is_num = !inner_trim.is_empty() && inner_trim.chars().all(|c| c.is_ascii_digit() || c == '.');
                let is_note = inner_trim.contains("借") || inner_trim.contains("收入") || inner_trim.contains("支出");
                if is_num {
                    out = out[..open_idx].trim().to_string();
                    continue;
                } else if is_note {
                    // 去括号注释："转账（借?）" → "转账"
                    out = out[..open_idx].trim().to_string();
                    continue;
                }
            }
        }
        break;
    }
    out.trim().trim_end_matches('等').to_string()
}

/// "谷费，ゲーム ， live&演出" → ["谷费", "ゲーム", "live&演出"]
fn split_children(s: &str) -> Vec<String> {
    s.split(|c: char| c == '，' || c == ',' || c == '、' || c == '；' || c == ';')
        .map(clean_name)
        .filter(|s| !s.is_empty() && s != "-" && s != "—")
        .collect()
}

fn placeholder_is_empty(s: &str) -> bool {
    let t = s.trim();
    t.is_empty() || t == "-" || t == "—" || t == "（留空）" || t == "(留空)" || t == "留空" || t == "无"
}

fn heading_kind(heading: &str) -> Option<&'static str> {
    if heading.contains("账户") {
        Some("account")
    } else if heading.contains("收入") {
        Some("income")
    } else if heading.contains("支出") {
        Some("expense")
    } else {
        None
    }
}

/// 解析 Markdown
pub fn parse_md(text: &str) -> PresetPreview {
    let mut preview = PresetPreview { categories: vec![], rules: vec![], warnings: vec![] };
    let mut current_kind: Option<String> = None;

    // 按 "##" 或 "# " 分节
    let mut sections: Vec<(String, Vec<&str>)> = vec![];
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            sections.push((t.trim_start_matches('#').trim().to_string(), vec![]));
        } else if let Some(last) = sections.last_mut() {
            last.1.push(line);
        }
    }
    if sections.is_empty() {
        sections.push(("全文".into(), text.lines().collect()));
    }

    for (title, lines) in &sections {
        // 规则表分节（标题含"规则"或"自动分类"）
        if title.contains("规则") || title.contains("自动分类") {
            parse_rules_table(&lines, &mut preview);
            continue;
        }
        if let Some(kind) = heading_kind(title) {
            current_kind = Some(kind.to_string());
        }
        let Some(kind) = current_kind.clone() else {
            continue;
        };

        // 一级目录行："**一级目录**：A（1），B（2），..."
        for line in lines {
            let t = line.trim_start_matches(['-', '*', ' ']).trim();
            let is_l1_line = (t.contains("一级目录") || t.contains("一级分类"))
                && (t.contains('：') || t.contains(':'));
            if !is_l1_line {
                continue;
            }
            let names = t
                .split_once('：')
                .or_else(|| t.split_once(':'))
                .map(|(_, rest)| split_children(rest))
                .unwrap_or_default();
            // 一级目录行本身可能混入说明文字，限制每项长度
            for name in names {
                if name.chars().count() <= 20 {
                    preview.categories.push(PresetCategory { kind: kind.clone(), l1: name, l2: None });
                }
            }
        }

        // 表格：表头「| 一级 | 二级子项/二级 |」，其后的行都是映射数据
        let mut in_cat_mapping = false;
        for line in lines {
            let t = line.trim();
            if !(t.starts_with('|') && t.ends_with('|')) {
                if t.is_empty() {
                    in_cat_mapping = false;
                }
                continue;
            }
            let cells: Vec<String> = t[1..t.len() - 1].split('|').map(|c| c.trim().to_string()).collect();
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ' ' || ch == ':')) {
                continue; // 分隔行
            }
            if cells.is_empty() {
                continue;
            }
            // 表头行：| 一级 | 二级子项 |（规则表由 parse_rules_table 单独处理）
            if cells[0].contains("一级") {
                in_cat_mapping = !cells.iter().skip(1).any(|c| c.contains("归属"));
                continue;
            }
            if !in_cat_mapping || cells.len() < 2 {
                continue;
            }
            let l1 = clean_name(&cells[0]);
            if l1.is_empty() {
                continue;
            }
            // 纯括号说明（如"（保留空白，用于……）"）视为无子项
            let l2_raw = cells[1].trim();
            let is_note_only = (l2_raw.starts_with('（') && l2_raw.ends_with('）'))
                || (l2_raw.starts_with('(') && l2_raw.ends_with(')'))
                || placeholder_is_empty(l2_raw);
            let children = if is_note_only { vec![] } else { split_children(&cells[1]) };
            if children.is_empty() {
                preview.categories.push(PresetCategory { kind: kind.clone(), l1: l1.clone(), l2: None });
            } else {
                for child in children {
                    preview.categories.push(PresetCategory { kind: kind.clone(), l1: l1.clone(), l2: Some(child) });
                }
            }
        }

        // 账户分类行格式："现金账户：现金" / "现金账户: 现金"
        if kind == "account" {
            for line in lines {
                let t = line.trim().trim_start_matches(['-', '*', ' ']).trim();
                // 跳过表头、表格行、以及「（格式：…）」这类说明行
                if t.starts_with('|') || t.contains('|') || t.starts_with('#')
                    || t.starts_with('（') || t.starts_with('(') {
                    continue;
                }
                if t.contains("格式") || t.contains("一级分类") || t.contains("二级分类") {
                    continue;
                }
                if let Some((l1_raw, rest_raw)) = t.split_once('：').or_else(|| t.split_once(':')) {
                    let l1 = clean_name(l1_raw);
                    if !l1.is_empty() && l1.chars().count() <= 15 {
                        let children = split_children(rest_raw);
                        if children.is_empty() {
                            preview.categories.push(PresetCategory { kind: kind.clone(), l1: l1.clone(), l2: None });
                        } else {
                            for child in children {
                                preview.categories.push(PresetCategory { kind: kind.clone(), l1: l1.clone(), l2: Some(child) });
                            }
                        }
                    }
                }
            }
        }
    }

    // 去重（同 kind+l1+l2）
    preview.categories.dedup_by(|a, b| a.kind == b.kind && a.l1 == b.l1 && a.l2 == b.l2);
    if preview.categories.is_empty() && preview.rules.is_empty() {
        preview.warnings.push("未识别出任何分类或规则，请检查文档格式（需包含「一级目录」「二级目录」或对应表格）".into());
    }
    preview
}

/// 解析规则表：| 规则/关键词 | 归属一级 | 归属二级 |
fn parse_rules_table(lines: &[&str], preview: &mut PresetPreview) {
    let mut table_rows: Vec<Vec<String>> = vec![];
    for line in lines {
        let t = line.trim();
        if t.starts_with('|') && t.ends_with('|') {
            let cells: Vec<String> = t[1..t.len() - 1].split('|').map(|c| c.trim().to_string()).collect();
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ' ' || ch == ':')) {
                continue;
            }
            table_rows.push(cells);
        }
    }
    for cells in &table_rows {
        if cells.len() < 2 {
            continue;
        }
        let first = &cells[0];
        if first.contains("规则") || first.contains("关键词") {
            continue; // 表头
        }
        // 判断收支方向：归属一级在收入体系则 income，否则 expense
        let l1_raw = cells.get(1).cloned().unwrap_or_default();
        let l1 = clean_name(&l1_raw);
        let kind = if l1.contains("收入") || l1 == "职业收入" || l1 == "其他收入" {
            "income"
        } else {
            "expense"
        };
        let l2_raw = cells.get(2).cloned().unwrap_or_default();
        let l2 = if placeholder_is_empty(&l2_raw) { None } else { Some(clean_name(&l2_raw)) };
        // 关键词列按 / 、 切分；去掉装饰性括号与"等"
        let kws: Vec<String> = first
            .split(|c: char| c == '/' || c == '、' || c == '，' || c == ',')
            .map(|k| {
                let k = clean_name(k);
                let k = k.trim_end_matches('等').trim().to_string();
                // 去括号注释："红包（收入）" → "红包"
                match k.find(|c: char| c == '(' || c == '（') {
                    Some(i) if i > 0 => k[..i].trim().to_string(),
                    _ => k,
                }
            })
            .filter(|k| !k.is_empty() && k.chars().count() <= 30)
            .collect();
        if kws.is_empty() {
            continue;
        }
        for kw in kws {
            preview.rules.push(RuleInput {
                id: None,
                keyword: kw,
                kind: kind.to_string(),
                l1: l1.clone(),
                l2: l2.clone(),
                priority: Some(0),
                enabled: Some(true),
            });
        }
    }
}

/// 解析 txt：支持
///   支出 > 食品饮料 > 早午晚餐   （斜杠/箭头单行）
///   缩进层级（顶层为 支出/收入/账户，依次缩进为 一级/二级）
pub fn parse_txt_file(text: &str) -> PresetPreview {
    let mut preview = PresetPreview { categories: vec![], rules: vec![], warnings: vec![] };
    let mut current_kind: Option<String> = None;

    for raw in text.lines() {
        let t = raw.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        // 单行路径：A / B / C 或 A > B > C 或 A－B
        if t.contains('/') || t.contains('>') || t.contains('＞') {
            let parts: Vec<&str> = t
                .split(|c: char| c == '/' || c == '>' || c == '＞')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .collect();
            if parts.len() >= 2 {
                let kind = kind_from_word(parts[0]).unwrap_or_else(|| current_kind.clone().unwrap_or_else(|| "expense".into()));
                if let Some(k) = kind_from_word(parts[0]) {
                    current_kind = Some(k.clone());
                }
                if parts.len() >= 2 {
                    preview.categories.push(PresetCategory {
                        kind: kind,
                        l1: clean_name(parts[1]),
                        l2: parts.get(2).map(|p| clean_name(p)),
                    });
                }
                continue;
            }
        }
        // 纯单词行：可能是 收支/账户 域标识
        if let Some(kind) = kind_from_word(t) {
            current_kind = Some(kind.to_string());
            continue;
        }
        // 缩进层级
        let indent = raw.len() - raw.trim_start().len();
        let name = clean_name(t);
        if name.is_empty() {
            continue;
        }
        let kind = current_kind.clone().unwrap_or_else(|| "expense".into());
        if indent == 0 {
            preview.categories.push(PresetCategory { kind, l1: name, l2: None });
        } else {
            if let Some(last) = preview.categories.last().cloned() {
                preview.categories.push(PresetCategory { kind: last.kind, l1: last.l1, l2: Some(name) });
            } else {
                preview.warnings.push(format!("行「{name}」缺少所属一级分类，已跳过"));
            }
        }
    }
    preview.categories.dedup_by(|a, b| a.kind == b.kind && a.l1 == b.l1 && a.l2 == b.l2);
    preview
}

fn kind_from_word(word: &str) -> Option<String> {
    let w = word.trim();
    if w.contains("账户") {
        Some("account".into())
    } else if w.contains("收入") {
        Some("income".into())
    } else if w.contains("支出") {
        Some("expense".into())
    } else {
        None
    }
}

/// 自动判断格式并解析
pub fn parse_auto(text: &str) -> (String, PresetPreview) {
    let looks_md = text.lines().any(|l| l.trim_start().starts_with('#'))
        || (text.matches('|').count() >= 4);
    if looks_md {
        ("md".into(), parse_md(text))
    } else {
        ("txt".into(), parse_txt_file(text))
    }
}
