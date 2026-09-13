export interface Category {
  id: number;
  kind: 'expense' | 'income' | 'account';
  parent_id: number | null;
  name: string;
  sort: number;
}

export interface CategoryInput {
  id?: number;
  kind: string;
  parent_id: number | null;
  name: string;
  sort?: number;
}

export interface Rule {
  id: number;
  keyword: string;
  kind: 'expense' | 'income';
  l1: string;
  l2: string | null;
  priority: number;
  enabled: boolean;
  source: 'preset' | 'learned';
}

export interface RuleInput {
  id?: number;
  keyword: string;
  kind: string;
  l1: string;
  l2: string | null;
  priority?: number;
  enabled?: boolean;
}

export interface Tx {
  id: number;
  tx_type: string;
  tx_time: string;
  l1: string | null;
  l2: string | null;
  l3?: string | null;
  account_out: string | null;
  account_in: string | null;
  currency: string | null;
  amount: number;
  member: string | null;
  merchant: string | null;
  project_category: string | null;
  project: string | null;
  booker: string | null;
  remark: string | null;
  tx_no?: string | null;
  source?: string | null;
  source_file?: string | null;
}

export interface TxFilter {
  tx_type?: string;
  l1?: string;
  l2?: string;
  merchant?: string;
  keyword?: string;
  date_from?: string;
  date_to?: string;
  min_amount?: number;
  max_amount?: number;
  page?: number;
  page_size?: number;
}

export interface TxPage {
  total: number;
  rows: Tx[];
}

export interface SheetStat {
  sheet: string;
  rows: number;
  inserted: number;
  duplicates: number;
}

export interface ImportReport {
  file: string;
  detected_format: string;
  total_rows: number;
  inserted: number;
  duplicates: number;
  skipped: number;
  pending: number;
  pending_merchants: string[];
  sheet_stats: SheetStat[];
  batch_id: string;
}

export interface MerchantAssign {
  merchant: string;
  l1: string;
  l2: string | null;
}

export interface PendingRow {
  id: number;
  tx: Tx;
}

export interface PresetCategory {
  kind: string;
  l1: string;
  l2: string | null;
}

export interface PresetPreview {
  categories: PresetCategory[];
  rules: RuleInput[];
  warnings: string[];
}

export interface SummaryStats {
  today_expense: number;
  today_income: number;
  month_expense: number;
  month_income: number;
  year_expense: number;
  year_income: number;
  month_transfer: number;
  tx_count: number;
}

export interface ChartPoint {
  label: string;
  income: number;
  expense: number;
}

export interface PiePoint {
  name: string;
  value: number;
}

export interface AccountBalance {
  name: string;
  group: string;
  base: number;
  net: number;
  balance: number;
}

export const TX_TYPES = ['支出', '收入', '转账', '报销', '代付', '余额变更', '债权变更'] as const;
