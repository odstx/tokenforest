import { test, expect } from '@playwright/test';

test.describe('API Keys Management', () => {
  test.beforeEach(async ({ page }) => {
    const testUser = {
      username: `testuser_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
      password: 'TestPassword123!'
    };
    
    await page.goto('/register');
    await page.fill('#username', testUser.username);
    await page.fill('#password', testUser.password);
    await page.fill('#confirmPassword', testUser.password);
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/', { timeout: 15000 });
  });

  test('should display API Keys page with empty state', async ({ page }) => {
    await page.goto('/api-keys');
    
    await expect(page.locator('h1')).toContainText('API Keys');
    await expect(page.getByText('No API keys yet')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Create API Key' })).toBeVisible();
  });

  test('should create a new API key', async ({ page }) => {
    await page.goto('/api-keys');
    
    await page.click('button:has-text("Create API Key")');
    
    await page.waitForSelector('.modal-open', { timeout: 5000 });
    
    await page.fill('#key-name', 'Test API Key');
    await page.click('.modal-open button:has-text("Create")');
    
    await expect(page.locator('.alert-success')).toContainText('API Key Created!');
    await expect(page.locator('.modal-open code')).toBeVisible();
    
    const apiKeyText = await page.locator('.modal-open code').textContent();
    expect(apiKeyText).toMatch(/^tf-/);
    
    await page.click('.modal-open button:has-text("Done")');
    
    await expect(page.locator('.modal-open')).not.toBeVisible();
    await page.waitForTimeout(500);
    
    await expect(page.locator('table tbody tr')).toHaveCount(1, { timeout: 10000 });
    await expect(page.locator('table tbody td').first()).toContainText('Test API Key');
  });

  test('should toggle API key status', async ({ page }) => {
    await page.goto('/api-keys');
    
    await page.click('button:has-text("Create API Key")');
    await page.waitForSelector('.modal-open');
    await page.fill('#key-name', 'Toggle Test Key');
    await page.click('.modal-open button:has-text("Create")');
    await page.waitForSelector('.alert-success');
    await page.click('.modal-open button:has-text("Done")');
    
    await expect(page.locator('.modal-open')).not.toBeVisible();
    await page.waitForTimeout(500);
    
    await expect(page.locator('table tbody tr')).toHaveCount(1, { timeout: 10000 });
    await expect(page.locator('.badge-success')).toBeVisible();
    
    await page.click('table tbody tr button.btn-outline');
    await page.waitForSelector('ul.menu.bg-base-100 button:has-text("Disable")', { state: 'visible' });
    await page.click('ul.menu.bg-base-100 button:has-text("Disable")');
    await page.waitForResponse(resp => 
      resp.url().includes('/toggle') && resp.request().method() === 'PUT'
    );
    
    await expect(page.locator('.badge-error')).toBeVisible({ timeout: 5000 });
    
    await page.click('table tbody tr button.btn-outline');
    await page.waitForSelector('ul.menu.bg-base-100 button:has-text("Enable")', { state: 'visible' });
    await page.click('ul.menu.bg-base-100 button:has-text("Enable")');
    await page.waitForResponse(resp => 
      resp.url().includes('/toggle') && resp.request().method() === 'PUT'
    );
    
    await expect(page.locator('.badge-success')).toBeVisible({ timeout: 5000 });
  });

  test('should delete API key', async ({ page }) => {
    await page.goto('/api-keys');
    
    await page.click('button:has-text("Create API Key")');
    await page.waitForSelector('.modal-open');
    await page.fill('#key-name', 'Delete Test Key');
    await page.click('.modal-open button:has-text("Create")');
    await page.waitForSelector('.alert-success');
    await page.click('.modal-open button:has-text("Done")');
    
    await expect(page.locator('.modal-open')).not.toBeVisible();
    await page.waitForTimeout(500);
    
    await expect(page.locator('table tbody tr')).toHaveCount(1, { timeout: 10000 });
    
    page.on('dialog', dialog => dialog.accept());
    
    await page.click('table tbody tr button.btn-outline');
    await page.waitForSelector('ul.menu.bg-base-100 button:has-text("Delete")', { state: 'visible' });
    await page.click('ul.menu.bg-base-100 button:has-text("Delete")');
    await page.waitForResponse(resp => 
      resp.url().match(/\/api\/api-keys\/\d+$/) !== null && resp.request().method() === 'DELETE'
    );
    
    await expect(page.locator('table tbody tr')).toHaveCount(0, { timeout: 5000 });
    await expect(page.getByText('No API keys yet')).toBeVisible();
  });

  test('should create multiple API keys and display them in list', async ({ page }) => {
    await page.goto('/api-keys');
    
    for (let i = 1; i <= 3; i++) {
      await page.click('button:has-text("Create API Key")');
      await page.waitForSelector('.modal-open');
      await page.fill('#key-name', `Test Key ${i}`);
      await page.click('.modal-open button:has-text("Create")');
      await page.waitForSelector('.alert-success');
      await page.click('.modal-open button:has-text("Done")');
      await page.waitForTimeout(200);
    }
    
    await expect(page.locator('table tbody tr')).toHaveCount(3, { timeout: 10000 });
    
    const keyNames = await page.locator('table tbody td:first-child').allTextContents();
    expect(keyNames).toContain('Test Key 1');
    expect(keyNames).toContain('Test Key 2');
    expect(keyNames).toContain('Test Key 3');
  });
});
