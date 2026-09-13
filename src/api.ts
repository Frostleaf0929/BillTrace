import { invoke } from '@tauri-apps/api/core';
import type {
  Category, CategoryInput, Rule, RuleInput, Tx, TxFilter, TxPage,
  ImportReport, MerchantAssign, PendingRow, PresetPreview,
  SummaryStats, ChartPoint, PiePoint, AccountBalance,
} from './types';

export const api = {
  getDataDir: () => invoke<string>('get_data_dir'),
  getSetting: (key: string) => invoke<string>('get_setting', { key }),
  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),

  listCategories: (kind?: string) => invoke<Category[]>('list_categories', { kind }),
  saveCategory: (input: CategoryInput) => invoke<number>('save_category', { input }),
  deleteCategory: (id: number) => invoke<void>('delete_category', { id }),
  moveCategory: (id: number, parentId: number | null, sort: number) =>
    invoke<void>('move_category', { id, parentId, sort }),

  listRules: () => invoke<Rule[]>('list_rules'),
  saveRule: (input: RuleInput) => invoke<number>('save_rule', { input }),
  deleteRule: (id: number) => invoke<void>('delete_rule', { id }),

  presetParse: (path: string) => invoke<[string, PresetPreview]>('preset_parse', { path }),
  presetApply: (preview: PresetPreview) => invoke<[number, number]>('preset_apply', { preview }),

  importBill: (path: string) => invoke<ImportReport>('import_bill', { path }),
  listPending: () => invoke<PendingRow[]>('list_pending'),
  resolvePending: (assigns: MerchantAssign[], skip: string[]) =>
    invoke<[number, number]>('resolve_pending', { assigns, skip }),

  queryTransactions: (filter: TxFilter) => invoke<TxPage>('query_transactions', { filter }),
  saveTransaction: (tx: Tx) => invoke<number>('save_transaction', { tx }),
  deleteTransaction: (id: number) => invoke<void>('delete_transaction', { id }),
  deleteTransactions: (ids: number[]) => invoke<number>('delete_transactions', { ids }),
  setTransactionsCategory: (ids: number[], l1: string | null, l2: string | null, l3: string | null) =>
    invoke<number>('set_transactions_category', { ids, l1, l2, l3 }),
  clearData: (scope: 'tx' | 'all') => invoke<void>('clear_data', { scope }),
  importBackground: (path: string) => invoke<string>('import_background', { path }),
  getBackground: () => invoke<string | null>('get_background'),
  clearBackground: () => invoke<void>('clear_background'),

  statsSummary: () => invoke<SummaryStats>('stats_summary'),
  statsChart: (dimension: string, dateFrom: string, dateTo: string) =>
    invoke<ChartPoint[]>('stats_chart', { dimension, dateFrom, dateTo }),
  statsPie: (txType: string, dateFrom: string, dateTo: string, groupBy: string) =>
    invoke<PiePoint[]>('stats_pie', { txType, dateFrom, dateTo, groupBy }),
  categorySums: (kind: string, monthPrefix: string) =>
    invoke<Record<string, number>>('category_sums', { kind, monthPrefix }),
  accountBalances: () => invoke<AccountBalance[]>('account_balances'),
  setAccountBase: (name: string, balance: number) => invoke<void>('set_account_base', { name, balance }),

  exportData: (format: string, filter: TxFilter) =>
    invoke<number>('export_data', { format, filter }),
  exportCategories: (format?: string) => invoke<void>('export_categories', { format: format ?? null }),
  exportImage: (name: string, dataBase64: string) => invoke<string>('export_image', { name, dataBase64 }),
  backupNow: () => invoke<string>('backup_now'),
  revealInExplorer: (path: string) => invoke<void>('reveal_in_explorer', { path }),
};

export function fmtAmount(n: number): string {
  return n.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
