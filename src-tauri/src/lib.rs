pub mod commands;
pub mod db;
pub mod exporter;
pub mod importer;
pub mod models;
pub mod presets;
pub mod rules;
pub mod stats;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

use db::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = db::resolve_data_dir(app.handle());
            let db_path = data_dir.join("zhangji.db");
            let conn = Connection::open(&db_path)
                .map_err(|e| format!("打开数据库失败({}): {e}", db_path.display()))?;
            db::init_db(&conn).map_err(|e| format!("初始化数据库失败: {e}"))?;
            db::seed_default_data(&conn).map_err(|e| format!("写入初始数据失败: {e}"))?;
            app.manage(AppState { conn: Mutex::new(conn), data_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_data_dir,
            commands::get_setting,
            commands::set_setting,
            commands::list_categories,
            commands::save_category,
            commands::delete_category,
            commands::move_category,
            commands::list_rules,
            commands::save_rule,
            commands::delete_rule,
            commands::preset_parse,
            commands::preset_apply,
            commands::import_bill,
            commands::list_pending,
            commands::resolve_pending,
            commands::query_transactions,
            commands::save_transaction,
            commands::delete_transaction,
            commands::delete_transactions,
            commands::set_transactions_category,
            commands::clear_data,
            commands::import_background,
            commands::get_background,
            commands::clear_background,
            commands::stats_summary,
            commands::stats_chart,
            commands::stats_pie,
            commands::category_sums,
            commands::account_balances,
            commands::set_account_base,
            commands::export_data,
            commands::export_categories,
            commands::export_image,
            commands::backup_now,
            commands::reveal_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
