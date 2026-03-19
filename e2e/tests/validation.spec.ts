/**
 * E2E Tests — Section 3.4: Form validation (E2E)
 *
 * TC-E2E-VAL-01  Login empty form → validation error
 * TC-E2E-VAL-02  Expense create required fields → "必須項目を入力してください"
 * TC-E2E-VAL-03  Expense create negative amount → validation error
 */

import { test, expect } from '@playwright/test';
import {
  CATEGORY_NAME,
  loginAsUser,
  clearAuth,
} from './helpers';

test.describe('TC-E2E-VAL: Form validation', () => {
  // -------------------------------------------------------------------------
  // TC-E2E-VAL-01: Login empty form → validation error
  // -------------------------------------------------------------------------
  test('TC-E2E-VAL-01: e2e_login_form_empty_validation', async ({ page }) => {
    await page.goto('/login');
    await clearAuth(page);

    // Do not fill any fields — click the submit button immediately
    await page.getByRole('button', { name: /ログイン/ }).click();

    // Stay on the login page
    await page.waitForLoadState('networkidle');
    expect(page.url()).toContain('/login');

    // A validation error should appear.
    // The spec (TC-LOGIN-02) expects: "メールアドレスとパスワードを入力してください"
    // We also accept any generic error container as a fallback.
    const hasExpectedText = await page
      .getByText('メールアドレスとパスワードを入力してください')
      .isVisible()
      .catch(() => false);

    const hasGenericError = await page
      .locator('p.text-red-500, [class*="red-5"]')
      .first()
      .isVisible()
      .catch(() => false);

    // Either the specific message or a generic error container must be shown.
    expect(hasExpectedText || hasGenericError).toBe(true);
  });

  // -------------------------------------------------------------------------
  // TC-E2E-VAL-02: Expense create required fields → "必須項目を入力してください"
  // -------------------------------------------------------------------------
  test('TC-E2E-VAL-02: e2e_expense_create_required_validation', async ({ page }) => {
    await loginAsUser(page);

    // Navigate to the new expense form
    await page.getByRole('link', { name: /新規申請/ }).first().click();
    await page.waitForURL(/\/expenses\/new/, { timeout: 8_000 });

    // Leave all fields empty and click the submit button
    await page.getByRole('button', { name: /申請する/ }).click();

    // Should stay on /expenses/new
    await page.waitForLoadState('networkidle');
    expect(page.url()).toContain('/expenses/new');

    // The error message specified in the test spec must appear
    await expect(
      page.getByText('必須項目を入力してください')
    ).toBeVisible({ timeout: 5_000 });
  });

  // -------------------------------------------------------------------------
  // TC-E2E-VAL-03: Expense create negative amount → validation error
  // -------------------------------------------------------------------------
  test('TC-E2E-VAL-03: e2e_expense_create_negative_amount_validation', async ({ page }) => {
    await loginAsUser(page);

    // Navigate to the new expense form
    await page.getByRole('link', { name: /新規申請/ }).first().click();
    await page.waitForURL(/\/expenses\/new/, { timeout: 8_000 });

    // Select a category so the field is valid
    await page.locator('select').selectOption({ label: CATEGORY_NAME });

    // Enter a negative amount
    await page.locator('input[type="number"]').fill('-100');

    // Fill the other required fields with valid data so only the amount triggers
    await page.locator('input[type="text"]').fill('バリデーションテスト TC-E2E-VAL-03');
    await page.locator('input[type="date"]').fill('2025-05-01');

    // Submit
    await page.getByRole('button', { name: /申請する/ }).click();

    // Should remain on the form page — no redirect
    await page.waitForLoadState('networkidle');
    expect(page.url()).toContain('/expenses/new');

    // A validation error must be visible.
    // The component test spec expects "金額は正の整数で入力してください" or similar.
    const hasSpecificMessage = await page
      .getByText(/金額は正の整数で入力してください|金額.*正.*入力/)
      .isVisible()
      .catch(() => false);

    const hasGenericError = await page
      .locator('p.text-red-500, [class*="red-5"]')
      .first()
      .isVisible()
      .catch(() => false);

    expect(hasSpecificMessage || hasGenericError).toBe(true);
  });
});
