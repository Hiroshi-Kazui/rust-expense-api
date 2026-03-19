/**
 * E2E Tests — Section 3.1: Authentication Flow
 *
 * TC-E2E-AUTH-01  Login success → redirect to /expenses
 * TC-E2E-AUTH-02  Wrong password → error message shown, stay on /login
 * TC-E2E-AUTH-03  Unauthenticated access to /expenses → redirect to /login
 * TC-E2E-AUTH-04  Logout → redirect to /login, localStorage token removed
 */

import { test, expect } from '@playwright/test';
import {
  ADMIN_EMAIL,
  ADMIN_PASSWORD,
  clearAuth,
  loginAsAdmin,
} from './helpers';

test.describe('TC-E2E-AUTH: Authentication flow', () => {
  // Ensure every test starts with a clean localStorage so tests are isolated.
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await clearAuth(page);
  });

  // -------------------------------------------------------------------------
  // TC-E2E-AUTH-01: Login success → redirect to /expenses
  // -------------------------------------------------------------------------
  test('TC-E2E-AUTH-01: e2e_login_success_redirects_to_expenses', async ({ page }) => {
    await page.goto('/login');

    await page.locator('input[type="email"]').fill(ADMIN_EMAIL);
    await page.locator('input[type="password"]').fill(ADMIN_PASSWORD);
    await page.getByRole('button', { name: /ログイン/ }).click();

    // Should redirect to /expenses
    await page.waitForURL(/\/expenses/, { timeout: 10_000 });
    expect(page.url()).toContain('/expenses');

    // The page heading should confirm we are on the expense list
    await expect(page.getByText('経費申請一覧')).toBeVisible();
  });

  // -------------------------------------------------------------------------
  // TC-E2E-AUTH-02: Wrong password → error message shown, stay on /login
  // -------------------------------------------------------------------------
  test('TC-E2E-AUTH-02: e2e_login_wrong_password_shows_error', async ({ page }) => {
    await page.goto('/login');

    await page.locator('input[type="email"]').fill(ADMIN_EMAIL);
    await page.locator('input[type="password"]').fill('wrong-password-xyz');
    await page.getByRole('button', { name: /ログイン/ }).click();

    // Should stay on /login
    await page.waitForLoadState('networkidle');
    expect(page.url()).toContain('/login');

    // The app renders error as <p class="text-xs text-red-500">
    const errorLocator = page.locator('p.text-red-500, [class*="red"]');
    await expect(errorLocator.first()).toBeVisible({ timeout: 5_000 });
  });

  // -------------------------------------------------------------------------
  // TC-E2E-AUTH-03: Unauthenticated access to /expenses → redirect to /login
  // -------------------------------------------------------------------------
  test('TC-E2E-AUTH-03: e2e_unauthenticated_redirects_to_login', async ({ page }) => {
    // Start on a blank page so we can manipulate localStorage before navigation.
    await page.goto('/login');
    await page.evaluate(() => localStorage.clear());

    // Navigate directly to the protected route
    await page.goto('/expenses');

    // Should be redirected to /login
    await page.waitForURL(/\/login/, { timeout: 10_000 });
    expect(page.url()).toContain('/login');
  });

  // -------------------------------------------------------------------------
  // TC-E2E-AUTH-04: Logout → redirect to /login, token removed from localStorage
  // -------------------------------------------------------------------------
  test('TC-E2E-AUTH-04: e2e_logout_redirects_to_login', async ({ page }) => {
    // First log in so we have a valid session
    await loginAsAdmin(page);

    // Confirm we are authenticated
    await page.waitForURL(/\/expenses/, { timeout: 10_000 });

    // Click the logout button/link in the sidebar
    await page.getByRole('button', { name: /ログアウト/ }).click();

    // Should redirect to /login
    await page.waitForURL(/\/login/, { timeout: 10_000 });
    expect(page.url()).toContain('/login');

    // localStorage token must be absent (the app uses key 'expense_token')
    const token = await page.evaluate(() => localStorage.getItem('expense_token'));
    expect(token).toBeNull();
  });
});
