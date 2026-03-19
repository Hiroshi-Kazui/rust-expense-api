import { Page, APIRequestContext, request } from '@playwright/test';

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const FRONTEND_URL = 'http://localhost:3000';
export const BACKEND_URL = 'http://localhost:8080';

export const ADMIN_EMAIL = 'admin@example.com';
export const ADMIN_PASSWORD = 'changeme';

export const USER_EMAIL = 'test-user@example.com';
export const USER_PASSWORD = 'password123';

export const CATEGORY_NAME = '交通費';

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/**
 * Log in via the UI login form.
 * After this call the browser is on /expenses (or wherever the app redirects).
 */
export async function loginViaUI(
  page: Page,
  email: string,
  password: string
): Promise<void> {
  await page.goto('/login');
  await page.locator('input[type="email"]').fill(email);
  await page.locator('input[type="password"]').fill(password);
  await page.getByRole('button', { name: /ログイン/ }).click();
  // Wait for navigation away from /login
  await page.waitForURL(/\/expenses/, { timeout: 10_000 });
}

/**
 * Log in as admin via the UI.
 */
export async function loginAsAdmin(page: Page): Promise<void> {
  await loginViaUI(page, ADMIN_EMAIL, ADMIN_PASSWORD);
}

/**
 * Log in as regular test user via the UI.
 */
export async function loginAsUser(page: Page): Promise<void> {
  await loginViaUI(page, USER_EMAIL, USER_PASSWORD);
}

/**
 * Clear localStorage to simulate an unauthenticated browser.
 * Call this before navigating to a protected page in unauthenticated tests.
 */
export async function clearAuth(page: Page): Promise<void> {
  await page.evaluate(() => localStorage.clear());
}

// ---------------------------------------------------------------------------
// API helpers — obtain a Bearer token without touching the browser
// ---------------------------------------------------------------------------

/**
 * Obtain a JWT bearer token from the backend API directly.
 * Returns the raw token string.
 */
export async function getApiToken(
  apiContext: APIRequestContext,
  email: string,
  password: string
): Promise<string> {
  const response = await apiContext.post(`${BACKEND_URL}/auth/login`, {
    data: { email, password },
  });
  if (!response.ok()) {
    throw new Error(
      `Login API call failed: ${response.status()} ${response.statusText()}`
    );
  }
  const body = await response.json();
  return body.token as string;
}

// ---------------------------------------------------------------------------
// Expense API helpers
// ---------------------------------------------------------------------------

export interface CreatedExpense {
  id: string;
  purpose: string;
}

/**
 * Create an expense via the API and return its id.
 * Requires a valid bearer token obtained from getApiToken().
 */
export async function createExpenseViaApi(
  apiContext: APIRequestContext,
  token: string,
  params: {
    categoryId: string;
    amount: number;
    purpose: string;
    occurredAt: string;
    note?: string;
  }
): Promise<CreatedExpense> {
  const formData: Record<string, string> = {
    category_id: params.categoryId,
    amount: params.amount.toString(),
    purpose: params.purpose,
    occurred_at: params.occurredAt,
  };
  if (params.note) {
    formData.note = params.note;
  }
  const response = await apiContext.post(`${BACKEND_URL}/expenses`, {
    headers: { Authorization: `Bearer ${token}` },
    multipart: formData,
  });
  if (!response.ok()) {
    throw new Error(
      `Create expense API call failed: ${response.status()} ${response.statusText()}`
    );
  }
  const body = await response.json();
  return { id: body.id as string, purpose: body.purpose as string };
}

// 1×1 transparent PNG for use as a dummy receipt file
const DUMMY_PNG_BASE64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwADhQGAWjR9awAAAABJRU5ErkJggg==';

/**
 * Create an expense with a dummy PNG receipt via the API.
 * Returns the expense id, purpose, and receipt_file filename.
 */
export async function createExpenseWithReceiptViaApi(
  apiContext: APIRequestContext,
  token: string,
  params: {
    categoryId: string;
    amount: number;
    purpose: string;
    occurredAt: string;
  }
): Promise<{ id: string; purpose: string; receiptFile: string }> {
  const response = await apiContext.post(`${BACKEND_URL}/expenses`, {
    headers: { Authorization: `Bearer ${token}` },
    multipart: {
      category_id: params.categoryId,
      amount: params.amount.toString(),
      purpose: params.purpose,
      occurred_at: params.occurredAt,
      receipt: {
        name: 'receipt.png',
        mimeType: 'image/png',
        buffer: Buffer.from(DUMMY_PNG_BASE64, 'base64'),
      },
    },
  });
  if (!response.ok()) {
    throw new Error(
      `Create expense with receipt failed: ${response.status()} ${response.statusText()}`
    );
  }
  const body = await response.json();
  return {
    id: body.id as string,
    purpose: body.purpose as string,
    receiptFile: body.receipt_file as string,
  };
}

/**
 * Delete an expense via the API.
 */
export async function deleteExpenseViaApi(
  apiContext: APIRequestContext,
  token: string,
  expenseId: string
): Promise<void> {
  await apiContext.delete(`${BACKEND_URL}/expenses/${expenseId}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
}

// ---------------------------------------------------------------------------
// Category API helpers
// ---------------------------------------------------------------------------

export interface Category {
  id: string;
  name: string;
}

/**
 * Fetch all categories and return them.
 */
export async function getCategoriesViaApi(
  apiContext: APIRequestContext,
  token: string
): Promise<Category[]> {
  const response = await apiContext.get(`${BACKEND_URL}/categories`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok()) {
    throw new Error(
      `Get categories API call failed: ${response.status()} ${response.statusText()}`
    );
  }
  return (await response.json()) as Category[];
}

/**
 * Find a category by name. Throws if not found.
 */
export async function findCategoryByName(
  apiContext: APIRequestContext,
  token: string,
  name: string
): Promise<Category> {
  const categories = await getCategoriesViaApi(apiContext, token);
  const cat = categories.find((c) => c.name === name);
  if (!cat) {
    throw new Error(`Category "${name}" not found`);
  }
  return cat;
}

// ---------------------------------------------------------------------------
// User API helpers
// ---------------------------------------------------------------------------

export interface CreatedUser {
  id: string;
  email: string;
  name: string;
}

/**
 * Create a user via the admin API.
 */
export async function createUserViaApi(
  apiContext: APIRequestContext,
  token: string,
  params: { email: string; name: string; password: string; role?: string }
): Promise<CreatedUser> {
  const response = await apiContext.post(`${BACKEND_URL}/admin/users`, {
    headers: { Authorization: `Bearer ${token}` },
    data: {
      email: params.email,
      name: params.name,
      password: params.password,
      role: params.role ?? 'User',
    },
  });
  if (!response.ok()) {
    throw new Error(
      `Create user API call failed: ${response.status()} ${response.statusText()}`
    );
  }
  const body = await response.json();
  return { id: body.id as string, email: body.email as string, name: body.name as string };
}

/**
 * Delete a user via the admin API.
 */
export async function deleteUserViaApi(
  apiContext: APIRequestContext,
  token: string,
  userId: string
): Promise<void> {
  await apiContext.delete(`${BACKEND_URL}/admin/users/${userId}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
}

// ---------------------------------------------------------------------------
// Shared fixture: API request context factory
// ---------------------------------------------------------------------------

/**
 * Create a standalone API request context that is independent of the browser.
 * Remember to call apiContext.dispose() in an afterAll/afterEach block.
 */
export async function createApiContext(): Promise<APIRequestContext> {
  return request.newContext({ baseURL: BACKEND_URL });
}
