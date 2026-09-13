/// 用真实数据文件验证：md 预设解析 → 规则入库 → 微信流水导入 → 随手记年度账本导入
/// 数据目录通过环境变量 ZHANGJI_REAL_DATA_DIR 指定（不随仓库分发，未设置则自动跳过）
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

#[test]
fn real_data_full_flow() {
    // 该测试依赖本机真实账单数据（不随仓库分发）；未设置环境变量或目录不存在时优雅跳过
    let Some(data_dir) = data_dir() else {
        eprintln!("跳过：未设置 ZHANGJI_REAL_DATA_DIR 或目录不存在");
        return;
    };
    let mut conn = test_db();

    // 1. 解析分类预设 md
    let md_path = format!("{data_dir}\\分类记录_调整.md");
    let text = std::fs::read_to_string(&md_path).unwrap();
    let preview = presets::parse_md(&text);
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

    // 3. 随手记年度账本导入（2024 + 2025）
    for year in ["2024年.xlsx", "2025年.xlsx"] {
        let path = format!("{data_dir}\\{year}");
        let report = importer::parse_suishouji(&path, &conn, &format!("test-{year}")).unwrap();
        println!(
            "{year}: 解析 {} 条，新增 {}，重复 {}，跳过 {}；各 Sheet: {:?}",
            report.total_rows, report.inserted, report.duplicates, report.skipped,
            report.sheet_stats.iter().map(|s| format!("{}({}/{})", s.sheet, s.inserted, s.rows)).collect::<Vec<_>>()
        );
        assert!(report.inserted > 0, "{year} 没有导入任何数据");
    }
    let total_after_ss: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0)).unwrap();
    println!("随手记导入后总数: {total_after_ss}");

    // 4. 微信流水导入（关闭新商家弹窗 → 走兜底分类；再开弹窗验证待确认队列）
    let wechat_files = [
        "微信账单（原始数据）\\微信支付账单流水 - 202607.xlsx",
        "微信账单（原始数据）\\微信支付账单流水文件(20251001-20251231)_20260627152319.xlsx",
        "微信账单（原始数据）\\微信支付账单流水文件(20260101-20260531)_20260627152355.xlsx",
        "微信账单（原始数据）\\微信支付账单流水文件(20260601-20260627)_20260627133735.xlsx",
    ];
    for f in wechat_files {
        let path = format!("{data_dir}\\{f}");
        let format = importer::detect_format(&path).unwrap();
        assert_eq!(format, "wechat", "{f} 格式识别错误");
        let report = importer::parse_wechat(&path, &mut conn, "test-wx", false, "统计未确认").unwrap();
        println!(
            "{}: 解析 {} 条，新增 {}，重复 {}，跳过(中性等) {}",
            f, report.total_rows, report.inserted, report.duplicates, report.skipped
        );
        assert!(report.inserted > 0, "{f} 没有导入任何数据");
        // 重复导入应全部判重（跳过的中性行仍是 skipped，其余全为 duplicates）
        let report2 = importer::parse_wechat(&path, &mut conn, "test-wx-2", false, "统计未确认").unwrap();
        assert_eq!(report2.inserted, 0, "{f} 二次导入出现了重复数据");
        assert_eq!(report2.duplicates, report.total_rows, "{f} 二次导入判重计数不符");
    }

    // 5. 分类覆盖情况抽查
    let unclassified: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE tx_type='支出' AND (l1 IS NULL OR l1='')", [], |r| r.get(0))
        .unwrap();
    let total_expense: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE tx_type='支出'", [], |r| r.get(0))
        .unwrap();
    println!("支出总数 {total_expense}，其中无一级分类 {unclassified}");

    // 6. 待确认队列验证（开启弹窗再导一份微信账单，因已去重不会有待确认；
    //    改为直接用空规则库验证）
    let fresh = test_db();
    let path = format!("{data_dir}\\{}", wechat_files[3]);
    let report = importer::parse_wechat(&path, &mut { fresh }, "test-pending", true, "统计未确认").unwrap();
    println!("空规则库导入: 新增 {}, 待确认 {} (商家 {:?})", report.inserted, report.pending, &report.pending_merchants[..report.pending_merchants.len().min(5)]);
    assert!(report.pending > 0, "空规则库下应产生待确认商家");

    // 7. 指纹唯一性
    let dup_fp: i64 = conn
        .query_row("SELECT COUNT(*) FROM (SELECT fingerprint FROM transactions GROUP BY fingerprint HAVING COUNT(*) > 1)", [], |r| r.get(0))
        .unwrap();
    assert_eq!(dup_fp, 0, "存在重复指纹");
    println!("=== 全部真实数据验证通过 ===");
}
