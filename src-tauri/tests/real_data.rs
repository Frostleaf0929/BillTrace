/// 用真实数据文件验证：md 预设解析 → 规则入库 → 随手记年度账本导入 → 微信流水导入
/// 数据目录通过环境变量 ZHANGJI_REAL_DATA_DIR 指定（不随仓库分发，未设置则自动跳过）。
/// 不写死任何真实文件名：按扩展名/命名模式在目录内发现文件，避免隐私信息进入仓库。
use zhangji_lib::{db, importer, presets, rules};

fn test_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    db::init_db(&conn).unwrap();
    db::seed_default_data(&conn).unwrap();
    conn
}

fn data_dir() -> Option<String> {
    std::env::var("ZHANGJI_REAL_DATA_DIR").ok().filter(|p| std::path::Path::new(p).exists())
}

/// 列出目录下指定扩展名（不分大小写）的文件，按文件名排序
fn list_files(dir: &str, ext: &str) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().map(|x| x.to_string_lossy().eq_ignore_ascii_case(ext)).unwrap_or(false)
        })
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    out.sort();
    out
}

#[test]
fn real_data_full_flow() {
    // 该测试依赖本机真实账单数据（不随仓库分发）；未设置环境变量或目录不存在时优雅跳过
    let Some(data_dir) = data_dir() else {
        eprintln!("跳过：未设置 ZHANGJI_REAL_DATA_DIR 或目录不存在");
        return;
    };
    let mut conn = test_db();

    // 1. 解析分类预设 md：依次尝试目录下各 .md，取第一个能解析出内容的（跳过 AGENTS.md 等说明文件）
    let mds = list_files(&data_dir, "md");
    let mut preview = None;
    for path in &mds {
        let Ok(text) = std::fs::read_to_string(path) else { continue };
        let p = presets::parse_md(&text);
        if !p.categories.is_empty() || !p.rules.is_empty() {
            println!(
                "使用预设文件: {}",
                std::path::Path::new(path).file_name().unwrap().to_string_lossy()
            );
            preview = Some(p);
            break;
        }
    }
    let Some(preview) = preview else {
        panic!("数据目录下的 .md 文件均未解析出分类预设");
    };
    println!("分类条目: {}", preview.categories.len());
    println!("规则条目: {}", preview.rules.len());
    for w in &preview.warnings {
        println!("警告: {w}");
    }
    assert!(preview.categories.len() > 20, "分类解析数量异常");
    assert!(preview.rules.len() > 10, "规则解析数量异常");

    // 2. 写入分类与规则
    for c in &preview.categories {
        let pid: i64 = match conn
            .query_row(
                "SELECT id FROM categories WHERE kind=?1 AND parent_id IS NULL AND name=?2",
                rusqlite::params![c.kind, c.l1],
                |r| r.get(0),
            ) {
            Ok(id) => id,
            Err(_) => {
                conn.execute(
                    "INSERT INTO categories(kind, parent_id, name, sort) VALUES (?1, NULL, ?2, 999)",
                    rusqlite::params![c.kind, c.l1],
                )
                .unwrap();
                conn.last_insert_rowid()
            }
        };
        if let Some(l2) = &c.l2 {
            conn.execute(
                "INSERT OR IGNORE INTO categories(kind, parent_id, name, sort) VALUES (?1, ?2, ?3, 999)",
                rusqlite::params![c.kind, pid, l2],
            )
            .unwrap();
        }
    }
    for r in &preview.rules {
        rules::upsert_rule(&conn, r).unwrap();
    }
    let rule_count: i64 = conn.query_row("SELECT COUNT(*) FROM rules", [], |r| r.get(0)).unwrap();
    println!("入库规则数: {rule_count}");
    assert!(rule_count > 10);

    // 3. 随手记年度账本导入：文件名以「年」结尾的 xlsx（如 2024年.xlsx）
    let ss_files: Vec<String> = list_files(&data_dir, "xlsx")
        .into_iter()
        .filter(|p| {
            std::path::Path::new(p)
                .file_stem()
                .map(|s| s.to_string_lossy().ends_with("年"))
                .unwrap_or(false)
        })
        .collect();
    assert!(!ss_files.is_empty(), "数据目录下未找到年度账本（文件名以「年」结尾的 xlsx）");
    for path in &ss_files {
        let name = std::path::Path::new(path).file_stem().unwrap().to_string_lossy().to_string();
        let report = importer::parse_suishouji(path, &conn, &format!("test-{name}")).unwrap();
        println!(
            "{name}: 解析 {} 条，新增 {}，重复 {}，跳过 {}；各 Sheet: {:?}",
            report.total_rows, report.inserted, report.duplicates, report.skipped,
            report.sheet_stats.iter().map(|s| format!("{}({}/{})", s.sheet, s.inserted, s.rows)).collect::<Vec<_>>()
        );
        assert!(report.inserted > 0, "{name} 没有导入任何数据");
    }
    let total_after_ss: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0)).unwrap();
    println!("随手记导入后总数: {total_after_ss}");

    // 4. 微信流水导入：「微信账单（原始数据）」子目录下的 xlsx；目录不存在则跳过该部分
    let wx_dir = format!("{data_dir}\\微信账单（原始数据）");
    let wx_files = list_files(&wx_dir, "xlsx");
    if wx_files.is_empty() {
        eprintln!("跳过微信流水部分：未找到目录 {wx_dir}");
    }
    for f in &wx_files {
        let name = std::path::Path::new(f).file_stem().unwrap().to_string_lossy().to_string();
        let format = importer::detect_format(f).unwrap();
        assert_eq!(format, "wechat", "{name} 格式识别错误");
        let report = importer::parse_wechat(f, &mut conn, "test-wx", false, "统计未确认").unwrap();
        println!(
            "{name}: 解析 {} 条，新增 {}，重复 {}，跳过(中性等) {}",
            report.total_rows, report.inserted, report.duplicates, report.skipped
        );
        assert!(report.inserted > 0, "{name} 没有导入任何数据");
        // 重复导入应全部判重（跳过的中性行仍是 skipped，其余全为 duplicates）
        let report2 = importer::parse_wechat(f, &mut conn, "test-wx-2", false, "统计未确认").unwrap();
        assert_eq!(report2.inserted, 0, "{name} 二次导入出现了重复数据");
        assert_eq!(report2.duplicates, report.total_rows, "{name} 二次导入判重计数不符");
    }

    // 5. 分类覆盖情况抽查
    let unclassified: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE tx_type='支出' AND (l1 IS NULL OR l1='')", [], |r| r.get(0))
        .unwrap();
    let total_expense: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE tx_type='支出'", [], |r| r.get(0))
        .unwrap();
    println!("支出总数 {total_expense}，其中无一级分类 {unclassified}");

    // 6. 待确认队列验证（空规则库导入一份微信账单，应产生待确认商家）
    if let Some(path) = wx_files.last() {
        let fresh = test_db();
        let report = importer::parse_wechat(path, &mut { fresh }, "test-pending", true, "统计未确认").unwrap();
        println!("空规则库导入: 新增 {}, 待确认 {} (商家 {:?})", report.inserted, report.pending, &report.pending_merchants[..report.pending_merchants.len().min(5)]);
        assert!(report.pending > 0, "空规则库下应产生待确认商家");
    }

    // 7. 指纹唯一性
    let dup_fp: i64 = conn
        .query_row("SELECT COUNT(*) FROM (SELECT fingerprint FROM transactions GROUP BY fingerprint HAVING COUNT(*) > 1)", [], |r| r.get(0))
        .unwrap();
    assert_eq!(dup_fp, 0, "存在重复指纹");
    println!("=== 全部真实数据验证通过 ===");
}
