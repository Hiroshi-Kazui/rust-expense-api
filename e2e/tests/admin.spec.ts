/**
 * E2E Tests — Section 3.3: Admin flow
 *
 * TC-E2E-ADMIN-01  Admin sees admin menu in sidebar
 * TC-E2E-ADMIN-02  Admin sees all expenses
 * TC-E2E-ADMIN-03  Individual approve → badge changes to "承認済"
 * TC-E2E-ADMIN-04  Individual reject → badge changes to "差し戻し"
 * TC-E2E-ADMIN-05  Bulk approve multiple → all badges change to "承認済"
 * TC-E2E-ADMIN-06  Admin creates user
 * TC-E2E-ADMIN-07  Admin creates category
 * TC-E2E-ADMIN-08  Non-admin cannot access /admin/users → redirect to /expenses
 */

import { test, expect } from '@playwright/test';
import {
  ADMIN_EMAIL,
  ADMIN_PASSWORD,
  USER_EMAIL,
  USER_PASSWORD,
  CATEGORY_NAME,
  loginAsAdmin,
  loginAsUser,
  clearAuth,
  getApiToken,
  createExpenseViaApi,
  deleteExpenseViaApi,
  findCategoryByName,
  deleteUserViaApi,
  createApiContext,
} from './helpers';

test.describe('TC-E2E-ADMIN: Admin flow', () => {
  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-01: Admin sees admin menu in sidebar
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-01: e2e_admin_sees_admin_menu', async ({ page }) => {
    await loginAsAdmin(page);
    await page.waitForURL(/\/expenses/, { timeout: 10_000 });

    // The sidebar should contain navigation links to all three admin sections.
    await expect(
      page.getByRole('link', { name: /全申請一覧/ })
    ).toBeVisible();
    await expect(
      page.getByRole('link', { name: /ユーザー/ })
    ).toBeVisible();
    await expect(
      page.getByRole('link', { name: /勘定項目/ })
    ).toBeVisible();
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-02: Admin sees all expenses
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-02: e2e_admin_sees_all_expenses', async ({ page }) => {
    const apiContext = await createApiContext();
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    const userToken = await getApiToken(apiContext, USER_EMAIL, USER_PASSWORD);
    const category = await findCategoryByName(apiContext, adminToken, CATEGORY_NAME);

    // Create one expense for admin and one for the regular user
    const adminExpense = await createExpenseViaApi(apiContext, adminToken, {
      categoryId: category.id,
      amount: 100,
      purpose: 'ADMIN申請 ADMIN-02',
      occurredAt: '2025-04-01',
    });
    const userExpense = await createExpenseViaApi(apiContext, userToken, {
      categoryId: category.id,
      amount: 200,
      purpose: 'USER申請 ADMIN-02',
      occurredAt: '2025-04-01',
    });

    try {
      await loginAsAdmin(page);
      await page.waitForURL(/\/expenses/, { timeout: 10_000 });

      // Navigate to the admin all-expenses page
      await page.getByRole('link', { name: /全申請一覧/ }).click();
      await page.waitForURL(/\/admin\/expenses/, { timeout: 8_000 });

      // Both expenses should be visible
      await expect(page.getByText('ADMIN申請 ADMIN-02')).toBeVisible();
      await expect(page.getByText('USER申請 ADMIN-02')).toBeVisible();
    } finally {
      await deleteExpenseViaApi(apiContext, adminToken, adminExpense.id);
      await deleteExpenseViaApi(apiContext, userToken, userExpense.id);
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-03: Individual approve → badge changes to "承認済"
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-03: e2e_admin_approve_single_expense', async ({ page }) => {
    const apiContext = await createApiContext();
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    const category = await findCategoryByName(apiContext, adminToken, CATEGORY_NAME);

    const expense = await createExpenseViaApi(apiContext, adminToken, {
      categoryId: category.id,
      amount: 400,
      purpose: '承認テスト ADMIN-03',
      occurredAt: '2025-04-02',
    });

    try {
      await loginAsAdmin(page);
      await page.getByRole('link', { name: /全申請一覧/ }).click();
      await page.waitForURL(/\/admin\/expenses/, { timeout: 8_000 });

      // Find the row for our expense and click its 承認 button
      const expenseRow = page.getByText('承認テスト ADMIN-03').locator('../..');
      await expenseRow.getByRole('button', { name: /^承認$/ }).click();

      // The badge in that row should change to 承認済
      await expect(
        expenseRow.locator('.badge-approved')
      ).toBeVisible({ timeout: 8_000 });
      await expect(expenseRow.getByText('承認済')).toBeVisible();
    } finally {
      // Attempt cleanup (already approved, may need admin token)
      try {
        await deleteExpenseViaApi(apiContext, adminToken, expense.id);
      } catch { /* ignore */ }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-04: Individual reject → badge changes to "差し戻し"
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-04: e2e_admin_reject_single_expense', async ({ page }) => {
    const apiContext = await createApiContext();
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    const category = await findCategoryByName(apiContext, adminToken, CATEGORY_NAME);

    const expense = await createExpenseViaApi(apiContext, adminToken, {
      categoryId: category.id,
      amount: 450,
      purpose: '差し戻しテスト ADMIN-04',
      occurredAt: '2025-04-03',
    });

    try {
      await loginAsAdmin(page);
      await page.getByRole('link', { name: /全申請一覧/ }).click();
      await page.waitForURL(/\/admin\/expenses/, { timeout: 8_000 });

      // Find the row and click its 差し戻し button
      const expenseRow = page.getByText('差し戻しテスト ADMIN-04').locator('../..');
      await expenseRow.getByRole('button', { name: /差し戻し/ }).click();

      // Badge should update to 差し戻し (rejected)
      await expect(
        expenseRow.locator('.badge-rejected')
      ).toBeVisible({ timeout: 8_000 });
      await expect(expenseRow.getByText('差し戻し')).toBeVisible();
    } finally {
      try {
        await deleteExpenseViaApi(apiContext, adminToken, expense.id);
      } catch { /* ignore */ }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-05: Bulk approve multiple → all badges change to "承認済"
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-05: e2e_admin_bulk_approve', async ({ page }) => {
    const apiContext = await createApiContext();
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    const category = await findCategoryByName(apiContext, adminToken, CATEGORY_NAME);

    // Create two pending expenses
    const expense1 = await createExpenseViaApi(apiContext, adminToken, {
      categoryId: category.id,
      amount: 111,
      purpose: '一括承認テスト1 ADMIN-05',
      occurredAt: '2025-04-04',
    });
    const expense2 = await createExpenseViaApi(apiContext, adminToken, {
      categoryId: category.id,
      amount: 222,
      purpose: '一括承認テスト2 ADMIN-05',
      occurredAt: '2025-04-04',
    });

    try {
      await loginAsAdmin(page);
      await page.getByRole('link', { name: /全申請一覧/ }).click();
      await page.waitForURL(/\/admin\/expenses/, { timeout: 8_000 });

      // Check the checkbox for each test expense row
      const row1 = page.getByText('一括承認テスト1 ADMIN-05').locator('../..');
      const row2 = page.getByText('一括承認テスト2 ADMIN-05').locator('../..');

      await row1.locator('input[type="checkbox"]').check();
      await row2.locator('input[type="checkbox"]').check();

      // Click the bulk approve button
      await page.getByRole('button', { name: /一括承認/ }).click();

      // Both rows should now show 承認済 badge
      await expect(row1.locator('.badge-approved')).toBeVisible({ timeout: 10_000 });
      await expect(row2.locator('.badge-approved')).toBeVisible({ timeout: 10_000 });
    } finally {
      try {
        await deleteExpenseViaApi(apiContext, adminToken, expense1.id);
        await deleteExpenseViaApi(apiContext, adminToken, expense2.id);
      } catch { /* ignore */ }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-06: Admin creates user
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-06: e2e_admin_create_user', async ({ page }) => {
    const newEmail = `e2e-user-${Date.now()}@example.com`;
    const newName = `E2E ユーザー ADMIN-06`;

    const apiContext = await createApiContext();
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    let createdUserId: string | undefined;

    try {
      await loginAsAdmin(page);
      await page.waitForURL(/\/expenses/, { timeout: 10_000 });

      // Navigate to /admin/users
      await page.getByRole('link', { name: /ユーザー/ }).click();
      await page.waitForURL(/\/admin\/users/, { timeout: 8_000 });

      // Fill in the user creation form
      // Form has: text (name), email, password, role(select)
      await page.locator('input[type="text"]').fill(newName);
      await page.locator('input[type="email"]').fill(newEmail);
      await page.locator('input[type="password"]').fill('TestPass123!');

      // Submit
      await page.getByRole('button', { name: /登録/ }).click();

      // The new user's name/email should appear in the user list
      await expect(page.getByText(newEmail)).toBeVisible({ timeout: 8_000 });

      // Capture the id from the DOM for cleanup if possible, otherwise use API
      const response = await apiContext.get('/admin/users', {
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      if (response.ok()) {
        const users = await response.json() as Array<{ id: string; email: string }>;
        const found = users.find((u) => u.email === newEmail);
        if (found) {
          createdUserId = found.id;
        }
      }
    } finally {
      if (createdUserId) {
        try {
          await deleteUserViaApi(apiContext, adminToken, createdUserId);
        } catch { /* ignore */ }
      }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-07: Admin creates category
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-07: e2e_admin_create_category', async ({ page }) => {
    const newCategoryName = `E2Eカテゴリ-${Date.now()}`;

    const apiContext = await createApiContext();
    const adminToken = await getApiToken(apiContext, ADMIN_EMAIL, ADMIN_PASSWORD);
    let createdCategoryId: string | undefined;

    try {
      await loginAsAdmin(page);
      await page.waitForURL(/\/expenses/, { timeout: 10_000 });

      // Navigate to /admin/categories
      await page.getByRole('link', { name: /勘定項目/ }).click();
      await page.waitForURL(/\/admin\/categories/, { timeout: 8_000 });

      // Fill the category name input (placeholder="勘定項目名")
      await page.getByPlaceholder('勘定項目名').fill(newCategoryName);

      // Submit
      await page.getByRole('button', { name: /追加/ }).click();

      // The new category should appear in the list
      await expect(page.getByText(newCategoryName)).toBeVisible({ timeout: 8_000 });

      // Capture id for cleanup
      const response = await apiContext.get('/categories', {
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      if (response.ok()) {
        const cats = await response.json() as Array<{ id: string; name: string }>;
        const found = cats.find((c) => c.name === newCategoryName);
        if (found) {
          createdCategoryId = found.id;
        }
      }
    } finally {
      if (createdCategoryId) {
        try {
          await apiContext.delete(`/categories/${createdCategoryId}`, {
            headers: { Authorization: `Bearer ${adminToken}` },
          });
        } catch { /* ignore */ }
      }
      await apiContext.dispose();
    }
  });

  // -------------------------------------------------------------------------
  // TC-E2E-ADMIN-08: Non-admin cannot access /admin/users → redirect to /expenses
  // -------------------------------------------------------------------------
  test('TC-E2E-ADMIN-08: e2e_non_admin_cannot_access_admin_pages', async ({ page }) => {
    // Log in as a regular user
    await loginAsUser(page);
    await page.waitForURL(/\/expenses/, { timeout: 10_000 });

    // Attempt direct navigation to an admin-only page
    await page.goto('/admin/users');

    // Should be redirected to /expenses
    await page.waitForURL(/\/expenses/, { timeout: 8_000 });
    expect(page.url()).toMatch(/\/expenses/);
    expect(page.url()).not.toContain('/admin');
  });
});
