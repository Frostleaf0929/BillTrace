//! 数据库诊断：打印副本库中的行数与样本（只读）
//! 用法：cargo run --example dbdump -- <db-path>
use rusqlite::Connection;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    println!("打开: {path}");
    let conn = match Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("打开失败: {e}");
            return;
        }
    };
    for table in ["transactions", "categories", "rules", "pending_rows", "account_base", "settings"] {
        let n: Result<i64, _> = conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0));
        match n {
            Ok(n) => println!("{table}: {n} 行"),
            Err(e) => println!("{table}: 查询失败 {e}"),
        }
    }
    let dump_sample = |conn: &Connection| {
        if let Ok(mut stmt) = conn.prepare("SELECT tx_type, tx_time, merchant, amount FROM transactions ORDER BY tx_time DESC LIMIT 5") {
            println!("--- 最新 5 笔 ---");
            if let Ok(rows) = stmt.query_map([], |r| {
                Ok(format!("{} | {} | {} | {}", r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default(), r.get::<_, f64>(3)?))
            }) {
                for row in rows {
                    println!("{}", row.unwrap_or_default());
                }
            }
        }
    };
    dump_sample(&conn);
}
