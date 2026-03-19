/**
 * E2E Tests — Section 3.2: Expense flow (regular user)
 *
 * TC-E2E-EXP-01  Create expense → redirect to list, "申請中" badge
 * TC-E2E-EXP-02  Create expense with file attachment
 * TC-E2E-EXP-03  Edit pending expense → update amount → redirect to list
 * TC-E2E-EXP-04  Delete pending expense (2-click confirm) → disappears from list
 * TC-E2E-EXP-05  User sees only own expenses
 */

import { test, expect, request as playwrightRequest } from '@playwright/test';
import path from 'path';
import {
  ADMIN_EMAIL,
  ADMIN_PASSWORD,
  USER_EMAIL,
  USER_PASSWORD,
  CATEGORY_NAME,
  loginAsUser,
  loginAsAdmin,
  loginViaUI,
  clearAuth,
  getApiToken,
  createExpenseViaApi,
  deleteExpenseViaApi,
  findCategoryByName,
  createUserViaApi,
  deleteUserViaApi,
  createApiContext,
} from './helpers';

// ---------------------------------------------------------------------------
// Shared test fixture: a small PNG encoded inline so no external file needed.
// Playwright can set file inputs from a Buffer.
// ---------------------------------------------------------------------------
const DUMMY_PNG_BASE64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwADhQGAWjR9awAAAABJRU5ErkJggg==';

test.describe('TC-E2E-EXP: Expense flow (regular user)', () => {
  // -------------------------------------------------------------------------
  // TC-E2E-EXP-01: Create expense → redirect to list, appears with "申請中" badge
  // -------------------------------------------------------------------------
  test('TC-E2E-EXP-01: e2e_create_expense_success', async ({ page }) => {
    await loginAsUser(page);

    // Navigate to the create form via the sidebar link
    await page.getByRole('link', { name: /新規申請/ }).first().click();
    await page.waitForURL(/\/expenses\/new/, { timeout: 8_000 });

    // Select category (交通費)
    await page.locator('select').selectOption({ label: CATEGORY_NAME });

    // Fill amount
    await page.locator('input[type="number"]').fill('1500');

    // Fill purpose (the only text input in the form)
    await page.locator('input[type="text"]').fill('E2E テスト申請 TC-E2E-EXP-01');

    // Fill occurred_at date
    await page.locator('input[type="date"]').fill('2025-01-15');

    // Submit
    await page.getByRole('button', { name: /申請する/ }).click();

    // Should redirect to /expenses
    await page.waitForURL(/\/expenses$/, { timeout: 10_000 });
    expect(page.url()).toMatch(/\/expenses$/);

    // The new expense should appear with a 申請中 badge
    await expect(
      page.locator('.badge-pending').first()
    ).toBeVisible({ timeout: 8_000 });

    // The purpose text should be visible in the list
    await expect(
      page.getByText('E2E テスト申請 TC-E2E-EXP-01').first()
    ).toBeVisible();
  });

  // -------------------------------------------------------------------------
  // TC-E2E-EXP-02: Create expense with file attachment
  // -------------------------------------------------------------------------
  test('TC-E2E-EXP-02: e2e_create_expense_with_receipt', async ({ page }) => {
    await loginAsUser(page);

    await page.getByRole('link', { name: /新規申請/ }).first().click();
    await page.waitForURL(/\/expenses\/new/, { timeout: 8_000 });

    // Fill required fields
    await page.locator('select').selectOption({ label: CATEGORY_NAME });
    await page.locator('input[type="number"]').fill('2000');
    await page.locator('input[type="text"]').fill('E2E ファイル添付テスト TC-E2E-EXP-02');
    await page.locator('input[type="date"]').fill('2025-01-16');

    // Attach a dummy PNG to the file input
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'receipt.png',
      mimeType: 'image/png',
      buffer: Buffer.from(DUMMY_PNG_BASE64, 'base64'),
    });

    await page.getByRole('button', { name: /申請する/ }).click();

    // Should redirect to /expenses successfully
    await page.waitForURL(/\/expenses$/, { timeout: 10_000 });
    expect(page.url()).toMatch(/\/expenses$/);

    // Confirm the submitted expense shows up in the list
    await expect(
      page.getByText('E2E ファイル添付テスト TC-E2E-EXP-02').first()
    ).toBeVisible();
  });

  // -------------------------------------------------------------------------
  // TC-E2E-EXP-03: Edit pending expense → update amount → redirect to list
  // -------------------------------------------------------------------------
  test('TC-E2E-EXP-03: e2e_edit_pending_expense', async ({ page }) => {
    // Pre-create a pending expense via the API so we have a reliable target.
    const apiContext = await createApiContext();
    const token = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, token, CATEGORY_NAME);

    const expense = await createExpenseViaApi(apiContext, token, {
      categoryId: category.id,
      amount: 500,
      purpose: 'E2E 編集テスト TC-E2E-EXP-03',
      occurredAt: '2025-02-01',
    });

    try {
      await loginAsUser(page);

      // Navigate to the expense list
      await page.waitForURL(/\/expenses/, { timeout: 8_000 });

      // Click the 編集 link for our test expense
      const row = page.getByText('E2E 編集テスト TC-E2E-EXP-03').locator('..');
      await row.getByRole('link', { name: /編集/ }).click();

      await page.waitForURL(new RegExp(`/expenses/${expense.id}/edit`), {
        timeout: 8_000,
      });

      // Clear the amount field and enter a new value
      const amountInput = page.locator('input[type="number"]');
      await amountInput.clear();
      await amountInput.fill('9999');

      // Submit the edit
      await page.getByRole('button', { name: /更新する/ }).click();

      // Should redirect to /expenses
      await page.waitForURL(/\/expenses$/, { timeout: 10_000 });
      expect(page.url()).toMatch(/\/expenses$/);

      // The updated amount should be visible in the list
      await expect(page.getByText('¥9999')).toBeVisible();
    } finally {
      // Best-effort cleanup; the test may have deleted it already
      try {
        await deleteExpenseViaApi(apiContext, token, expense.id);
      } catch {
        // ignore
      }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-EXP-04: Delete pending expense (2-click confirm) → disappears from list
  // -------------------------------------------------------------------------
  test('TC-E2E-EXP-04: e2e_delete_pending_expense', async ({ page }) => {
    const apiContext = await createApiContext();
    const token = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, token, CATEGORY_NAME);

    const expense = await createExpenseViaApi(apiContext, token, {
      categoryId: category.id,
      amount: 300,
      purpose: 'E2E 削除テスト TC-E2E-EXP-04',
      occurredAt: '2025-02-02',
    });

    try {
      await loginAsUser(page);
      await page.waitForURL(/\/expenses/, { timeout: 8_000 });

      // Navigate directly to the edit page
      await page.goto(`/expenses/${expense.id}/edit`);
      await page.waitForURL(new RegExp(`/expenses/${expense.id}/edit`), {
        timeout: 8_000,
      });

      // First click on delete button — should change text to 本当に削除
      const deleteBtn = page.getByRole('button', { name: /削除/ });
      await deleteBtn.click();

      // Button text should now show the confirmation label
      await expect(
        page.getByRole('button', { name: /本当に削除/ })
      ).toBeVisible({ timeout: 5_000 });

      // Second click — confirms deletion
      await page.getByRole('button', { name: /本当に削除/ }).click();

      // Should redirect to /expenses
      await page.waitForURL(/\/expenses$/, { timeout: 10_000 });
      expect(page.url()).toMatch(/\/expenses$/);

      // The deleted expense should no longer appear in the list
      await expect(
        page.getByText('E2E 削除テスト TC-E2E-EXP-04')
      ).not.toBeVisible();
    } finally {
      // The expense was deleted through the UI; ignore API cleanup errors.
      try {
        await deleteExpenseViaApi(apiContext, token, expense.id);
      } catch {
        // ignore
      }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-EXP-05: User sees only own expenses
  // -------------------------------------------------------------------------
  test('TC-E2E-EXP-05: e2e_user_sees_own_expenses_only', async ({ page }) => {
    const apiContext = await createApiContext();

    // Obtain tokens for both users
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    const userToken = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, adminToken, CATEGORY_NAME);

    // Create one expense as admin (should NOT appear in user list)
    const adminExpense = await createExpenseViaApi(apiContext, adminToken, {
      categoryId: category.id,
      amount: 1111,
      purpose: 'ADMIN専用申請 TC-E2E-EXP-05',
      occurredAt: '2025-03-01',
    });

    // Create one expense as the regular user (MUST appear in their list)
    const userExpense = await createExpenseViaApi(apiContext, userToken, {
      categoryId: category.id,
      amount: 2222,
      purpose: 'USER専用申請 TC-E2E-EXP-05',
      occurredAt: '2025-03-01',
    });

    try {
      // Log in as the regular user and check the expense list
      await loginAsUser(page);
      await page.waitForURL(/\/expenses/, { timeout: 8_000 });

      // Own expense must be visible
      await expect(page.getByText('USER専用申請 TC-E2E-EXP-05')).toBeVisible();

      // Admin's expense must NOT be visible
      await expect(
        page.getByText('ADMIN専用申請 TC-E2E-EXP-05')
      ).not.toBeVisible();
    } finally {
      await deleteExpenseViaApi(apiContext, adminToken, adminExpense.id);
      await deleteExpenseViaApi(apiContext, userToken, userExpense.id);
      await apiContext.dispose();
    }
  });
});
