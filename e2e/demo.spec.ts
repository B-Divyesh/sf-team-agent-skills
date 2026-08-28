import { expect, test } from '@playwright/test';

test('@claim:demo-separation demo changes reset to the separate sample workspace', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Review', exact: true }).click();
  await expect(page.getByText('Secure commit is now in review.')).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByText('Demo reset. The sample packets are back.')).toBeVisible();
  await expect(page.getByText('Released everywhere')).toBeVisible();
  const keys = await page.evaluate(() => Object.keys(localStorage));
  expect(keys).toContain('demo:team-agent-skills:v1');
  expect(keys.some(key => key.startsWith('real:'))).toBe(false);
});

test('@claim:execution-receipt demo records the selected version and repository', async ({ page }) => {
  await page.goto('/demo');
  await page.getByLabel('Repository').fill('payments-api');
  await page.getByRole('button', { name: 'Record execution receipt' }).click();
  await expect(page.getByText('Recorded rcpt-')).toBeVisible();
  const row = page.locator('tbody tr').first();
  await expect(row).toContainText('Secure commit');
  await expect(row).toContainText('v2.4.0');
  await expect(row).toContainText('payments-api');
});

test('@claim:demo-local-data demo requests only the product origin', async ({ page, context }) => {
  const origins = new Set<string>();
  page.on('request', request => origins.add(new URL(request.url()).origin));
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await page.waitForTimeout(100);
  expect([...origins]).toEqual(['http://127.0.0.1:4173']);
  await context.setOffline(true);
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
});
