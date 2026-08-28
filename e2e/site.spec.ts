import { expect, test } from '@playwright/test';

test('landing has a keyboard path and one main heading at phone width', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const errors: string[] = [];
  const apiRequests: string[] = [];
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  page.on('request', request => { if (new URL(request.url()).pathname.startsWith('/api/')) apiRequests.push(request.url()); });
  await page.goto('/');
  await expect(page.locator('main')).toBeVisible();
  await expect(page.locator('h1')).toHaveCount(1);
  await page.keyboard.press('Tab');
  await expect(page.getByText('Skip to content')).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('main')).toBeFocused();
  expect(apiRequests).toEqual([]);
  expect(errors).toEqual([]);
});

test('client-side navigation moves focus to the page heading and announces it', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('navigation').getByRole('link', { name: 'Privacy' }).click();
  const heading = page.getByRole('heading', { level: 1, name: 'Privacy for the registry' });
  await expect(heading).toBeFocused();
  await expect(page.locator('#route-announcer')).toHaveText('Privacy — Team Skills Registry');
});
