// 开发工具：把真实账本数据库导出为 JSON（供 tools/mock 测试台注入使用）
// 用法：cargo run --release --bin dump_json -- <db路径> <输出json路径>
// 只读打开，不写库；含 WAL 时 SQLite 会自动读取最新已提交状态
use rusqlite::Connection;
use serde_json::{json, Value};
use std::env;

fn rows_to_json(conn: &Connection, sql: &str) -> Vec<Value> {
    let mut stmt = conn.prepare(sql).expect("prepare failed");
    let cols: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let mut rows = stmt.query([]).expect("query failed");
    let mut out = Vec::new();
    while let Some(row) = rows.next().expect("row failed") {
        let mut obj = serde_json::Map::new();
        for (i, c) in cols.iter().enumerate() {
            let v: Value = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                Ok(rusqlite::types::ValueRef::Integer(n)) => json!(n),
                Ok(rusqlite::types::ValueRef::Real(f)) => json!(f),
                Ok(rusqlite::types::ValueRef::Text(t)) => {
                    json!(String::from_utf8_lossy(t).to_string())
                }
                Ok(rusqlite::types::ValueRef::Blob(b)) => json!(format!("<blob {}b>", b.len())),
                Err(_) => Value::Null,
            };
            obj.insert(c.clone(), v);
        }
        out.push(Value::Object(obj));
    }
    out
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: dump_json <db路径> <输出json路径>");
        std::process::exit(1);
    }
    let conn = Connection::open_with_flags(
        &args[1],
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("打不开数据库");
    let cats = rows_to_json(&conn, "SELECT id, kind, parent_id, name, sort FROM categories ORDER BY id");
    let txs = rows_to_json(
        &conn,
        "SELECT id, tx_type, tx_time, l1, l2, account_out, account_in, currency, amount, member, merchant, project_category, project, booker, remark FROM transactions ORDER BY tx_time DESC, id DESC",
    );
    let out = json!({ "cats": cats, "txs": txs });
    std::fs::write(&args[2], serde_json::to_string(&out).unwrap()).expect("写文件失败");
    println!("cats={} txs={} -> {}", cats.len(), txs.len(), args[2]);
}
