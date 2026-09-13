use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub kind: String,          // expense | income | account
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInput {
    pub id: Option<i64>,
    pub kind: String,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: i64,
    pub keyword: String,
    pub kind: String, // expense | income
    pub l1: String,
    pub l2: Option<String>,
    pub priority: i64,
    pub enabled: bool,
    pub source: String, // preset | learned
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInput {
    pub id: Option<i64>,
    pub keyword: String,
    pub kind: String,
    pub l1: String,
    pub l2: Option<String>,
    pub priority: Option<i64>,
    pub enabled: Option<bool>,
}

/// 一条交易记录，字段与随手记年度 Excel 对齐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tx {
    #[serde(default)]
    pub id: i64,
    pub tx_type: String, // 支出/收入/转账/报销/代付/余额变更/债权变更
    pub tx_time: String, // YYYY-MM-DD HH:MM:SS
    #[serde(default)]
    pub l1: Option<String>,
    #[serde(default)]
    pub l2: Option<String>,
    #[serde(default)]
    pub l3: Option<String>,
    #[serde(default)]
    pub account_out: Option<String>,
    #[serde(default)]
    pub account_in: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    pub amount: f64,
    #[serde(default)]
    pub member: Option<String>,
    #[serde(default)]
    pub merchant: Option<String>,
    #[serde(default)]
    pub project_category: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub booker: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub tx_no: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TxFilter {
    #[serde(default)]
    pub tx_type: Option<String>,
    #[serde(default)]
    pub l1: Option<String>,
    #[serde(default)]
    pub l2: Option<String>,
    #[serde(default)]
    pub merchant: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub date_from: Option<String>,
    #[serde(default)]
    pub date_to: Option<String>,
    #[serde(default)]
    pub min_amount: Option<f64>,
    #[serde(default)]
    pub max_amount: Option<f64>,
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxPage {
    pub total: i64,
    pub rows: Vec<Tx>,
}

/// 导入报告：每个文件解析后返回给前端预览
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportReport {
    pub file: String,
    pub detected_format: String, // suishouji | wechat
    pub total_rows: i64,
    pub inserted: i64,
    pub duplicates: i64,
    pub skipped: i64,           // 无法解析的行（如微信"中性"交易）
    pub pending: i64,           // 进入待确认队列的行数（新商家）
    pub pending_merchants: Vec<String>,
    pub sheet_stats: Vec<SheetStat>,
    pub batch_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetStat {
    pub sheet: String,
    pub rows: i64,
    pub inserted: i64,
    pub duplicates: i64,
}

/// 待确认行（存储在 pending_rows 表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRow {
    pub id: i64,
    pub tx: Tx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantAssign {
    pub merchant: String,
    pub l1: String,
    pub l2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetPreview {
    pub categories: Vec<PresetCategory>,
    pub rules: Vec<RuleInput>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetCategory {
    pub kind: String, // expense | income | account
    pub l1: String,
    pub l2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub today_expense: f64,
    pub today_income: f64,
    pub month_expense: f64,
    pub month_income: f64,
    pub year_expense: f64,
    pub year_income: f64,
    pub month_transfer: f64,
    pub tx_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub label: String,
    pub income: f64,
    pub expense: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiePoint {
    pub name: String,
    pub value: f64,
}

/// 账户余额 = 手动设定的基数 + 该账户全部交易净额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub name: String,
    pub group: String, // 所属一级账户（现金账户/虚拟账户…）
    pub base: f64,
    pub net: f64,
    pub balance: f64,
}
