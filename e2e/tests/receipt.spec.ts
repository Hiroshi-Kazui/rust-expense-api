/**
 * E2E Tests — Section 3.4: Receipt attachment flow
 *
 * TC-E2E-RECEIPT-01  Expense list shows DL link for receipt-attached expense
 * TC-E2E-RECEIPT-02  Edit page shows receipt section with ダウンロード/添付削除
 * TC-E2E-RECEIPT-03  Clicking 添付削除 removes the receipt section from edit page
 */

import { test, expect } from '@playwright/test';
import {
  USER_EMAIL,
  USER_PASSWORD,
  CATEGORY_NAME,
  loginAsUser,
  getApiToken,
  createExpenseWithReceiptViaApi,
  deleteExpenseViaApi,
  findCategoryByName,
  createApiContext,
  BACKEND_URL,
} from './helpers';

test.describe('TC-E2E-RECEIPT: Receipt attachment flow', () => {
  // -------------------------------------------------------------------------
  // TC-E2E-RECEIPT-01: List page shows DL link
  // -------------------------------------------------------------------------
  test('TC-E2E-RECEIPT-01: e2e_receipt_links_visible_in_list', async ({ page }) => {
    const apiContext = await createApiContext();
    const token = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, token, CATEGORY_NAME);
    const runId = Date.now();

    const expense = await createExpenseWithReceiptViaApi(apiContext, token, {
      categoryId: category.id,
      amount: 500,
      purpose: `レシートテスト1-${runId}`,
      occurredAt: '2025-06-01',
    });

    try {
      await loginAsUser(page);
      await page.waitForURL(/\/expenses/, { timeout: 8_000 });

      const row = page.getByText(`レシートテスト1-${runId}`).locator('..');

      // DLリンクが存在して download 属性を持つ
      const dlLink = row.getByRole('link', { name: /DL/ });
      await expect(dlLink).toBeVisible({ timeout: 8_000 });
      await expect(dlLink).toHaveAttribute('download');
    } finally {
      try { await deleteExpenseViaApi(apiContext, token, expense.id); } catch { /* ignore */ }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-RECEIPT-02: Edit page shows receipt section
  // -------------------------------------------------------------------------
  test('TC-E2E-RECEIPT-02: e2e_receipt_section_visible_in_edit', async ({ page }) => {
    const apiContext = await createApiContext();
    const token = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, token, CATEGORY_NAME);
    const runId = Date.now();

    const expense = await createExpenseWithReceiptViaApi(apiContext, token, {
      categoryId: category.id,
      amount: 600,
      purpose: `レシートテスト2-${runId}`,
      occurredAt: '2025-06-02',
    });

    try {
      await loginAsUser(page);
      await page.goto(`/expenses/${expense.id}/edit`);
      await page.waitForURL(new RegExp(`/expenses/${expense.id}/edit`), { timeout: 8_000 });

      // 添付ファイルセクションが表示される
      await expect(page.getByText('添付ファイル')).toBeVisible({ timeout: 8_000 });

      // ダウンロード・添付削除ボタンが存在する
      const dlLink = page.getByRole('link', { name: /ダウンロード/ });
      await expect(dlLink).toBeVisible();
      await expect(page.getByRole('button', { name: /添付削除/ })).toBeVisible();

      // ダウンロードリンクが /uploads/ を指している
      const dlHref = await dlLink.getAttribute('href');
      expect(dlHref).toMatch(/\/uploads\//);
    } finally {
      try { await deleteExpenseViaApi(apiContext, token, expense.id); } catch { /* ignore */ }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-RECEIPT-03: 添付削除ボタンでレシートが削除される
  // -------------------------------------------------------------------------
  test('TC-E2E-RECEIPT-03: e2e_receipt_delete_removes_section', async ({ page }) => {
    const apiContext = await createApiContext();
    const token = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, token, CATEGORY_NAME);
    const runId = Date.now();

    const expense = await createExpenseWithReceiptViaApi(apiContext, token, {
      categoryId: category.id,
      amount: 700,
      purpose: `レシートテスト3-${runId}`,
      occurredAt: '2025-06-03',
    });

    try {
      await loginAsUser(page);
      await page.goto(`/expenses/${expense.id}/edit`);
      await page.waitForURL(new RegExp(`/expenses/${expense.id}/edit`), { timeout: 8_000 });

      // 添付削除ボタンをクリック
      await expect(page.getByRole('button', { name: /添付削除/ })).toBeVisible({ timeout: 8_000 });
      await page.getByRole('button', { name: /添付削除/ }).click();

      // 添付ファイルセクションが消える
      await expect(page.getByText('添付ファイル')).not.toBeVisible({ timeout: 8_000 });
      await expect(page.getByRole('button', { name: /添付削除/ })).not.toBeVisible();

      // API でも receipt_file が null になっていることを確認
      const resp = await apiContext.get(`${BACKEND_URL}/expenses/${expense.id}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      const body = await resp.json();
      expect(body.receipt_file).toBeNull();
    } finally {
      try { await deleteExpenseViaApi(apiContext, token, expense.id); } catch { /* ignore */ }
      await apiContext.dispose();
    }
  });
});
