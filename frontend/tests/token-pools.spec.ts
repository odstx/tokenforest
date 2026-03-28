import { test, expect } from '@playwright/test';

test.describe('Token Pools Management', () => {
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

  test('should display Token Pools page with empty state', async ({ page }) => {
    await page.goto('/token-pools');
    
    await expect(page.locator('h1')).toContainText('Token Pools');
    await expect(page.getByText('No token pools')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Add Token Pool' })).toBeVisible();
  });

  test('should create a new token pool', async ({ page }) => {
    await page.goto('/token-pools');
    
    await page.click('button:has-text("Add Token Pool")');
    
    await page.waitForSelector('.modal-open', { timeout: 5000 });
    
    await page.fill('#pool-name', 'Test Pool');
    await page.fill('#pool-url', 'https://api.openai.com/v1');
    await page.fill('#pool-key', 'sk-test-key-12345');
    
    await page.click('#pool-model');
    await page.waitForSelector('ul.menu:visible button:has-text("GPT-4o")', { timeout: 5000 });
    await page.click('ul.menu:visible button:has-text("GPT-4o")');
    await page.waitForTimeout(100);
    
    const addButton = page.locator('.modal-open button.btn-primary');
    await expect(addButton).not.toBeDisabled({ timeout: 3000 });
    
    await Promise.all([
      page.waitForResponse(resp => 
        resp.url().includes('/api/token-pools') && resp.request().method() === 'POST' && resp.status() === 200
      ),
      addButton.click()
    ]);
    
    await expect(page.locator('.modal-open')).not.toBeVisible({ timeout: 5000 });
    
    await expect(page.locator('table tbody tr')).toHaveCount(1, { timeout: 10000 });
    await expect(page.locator('table tbody td').first()).toContainText('Test Pool');
  });

  test('should toggle token pool status', async ({ page }) => {
    await page.goto('/token-pools');
    
    await page.click('button:has-text("Add Token Pool")');
    await page.waitForSelector('.modal-open');
    await page.fill('#pool-name', 'Toggle Test Pool');
    await page.fill('#pool-url', 'https://api.openai.com/v1');
    await page.fill('#pool-key', 'sk-test-key-12345');
    await page.click('#pool-model');
    await page.waitForSelector('.fixed ul.menu button:has-text("GPT-4o")');
    await page.click('.fixed ul.menu button:has-text("GPT-4o")');
    await page.click('.modal-open button:has-text("Add")');
    await page.waitForResponse(resp => 
      resp.url().includes('/api/token-pools') && resp.request().method() === 'POST' && resp.status() === 200
    );
    await expect(page.locator('.modal-open')).not.toBeVisible({ timeout: 5000 });
    
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

  test('should delete token pool', async ({ page }) => {
    await page.goto('/token-pools');
    
    await page.click('button:has-text("Add Token Pool")');
    await page.waitForSelector('.modal-open');
    await page.fill('#pool-name', 'Delete Test Pool');
    await page.fill('#pool-url', 'https://api.openai.com/v1');
    await page.fill('#pool-key', 'sk-test-key-12345');
    await page.click('#pool-model');
    await page.waitForSelector('.fixed ul.menu button:has-text("GPT-4o")');
    await page.click('.fixed ul.menu button:has-text("GPT-4o")');
    await page.click('.modal-open button:has-text("Add")');
    await page.waitForResponse(resp => 
      resp.url().includes('/api/token-pools') && resp.request().method() === 'POST' && resp.status() === 200
    );
    await expect(page.locator('.modal-open')).not.toBeVisible({ timeout: 5000 });
    
    await expect(page.locator('table tbody tr')).toHaveCount(1, { timeout: 10000 });
    
    page.on('dialog', dialog => dialog.accept());
    
    await page.click('table tbody tr button.btn-outline');
    await page.waitForSelector('ul.menu.bg-base-100 button:has-text("Delete")', { state: 'visible' });
    await page.click('ul.menu.bg-base-100 button:has-text("Delete")');
    await page.waitForResponse(resp => 
      resp.url().match(/\/api\/token-pools\/\d+$/) !== null && resp.request().method() === 'DELETE'
    );
    
    await expect(page.locator('table tbody tr')).toHaveCount(0, { timeout: 5000 });
    await expect(page.getByText('No token pools')).toBeVisible();
  });

  test('should create multiple token pools and display them in list', async ({ page }) => {
    await page.goto('/token-pools');
    
    for (let i = 1; i <= 3; i++) {
      await page.click('button:has-text("Add Token Pool")');
      await page.waitForSelector('.modal-open');
      await page.fill('#pool-name', `Test Pool ${i}`);
      await page.fill('#pool-url', `https://api${i}.example.com/v1`);
      await page.fill('#pool-key', `sk-test-key-${i}`);
      await page.click('#pool-model');
      await page.waitForSelector('.fixed ul.menu button:has-text("GPT-4o")');
      await page.click('.fixed ul.menu button:has-text("GPT-4o")');
      await page.click('.modal-open button:has-text("Add")');
      await page.waitForResponse(resp => 
        resp.url().includes('/api/token-pools') && resp.request().method() === 'POST' && resp.status() === 200
      );
      await page.waitForTimeout(200);
    }
    
    await expect(page.locator('table tbody tr')).toHaveCount(3, { timeout: 10000 });
    
    const poolNames = await page.locator('table tbody td:first-child').allTextContents();
    expect(poolNames).toContain('Test Pool 1');
    expect(poolNames).toContain('Test Pool 2');
    expect(poolNames).toContain('Test Pool 3');
  });
});
