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

test('route metadata follows the current page', async ({ page }) => {
  await page.goto('/privacy');
  await expect(page).toHaveTitle('Privacy — Team Skills Registry');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://team-agent-skills.sociobot.in/privacy');
  await expect(page.locator('meta[property="og:title"]')).toHaveAttribute('content', 'Privacy — Team Skills Registry');
});

test('rejected rollout remains unchanged and explains recovery', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('team-agent-skills:workspace-key', 'tsr_test_workspace_key_12345678901234567890'));
  const skill = {
    id:'secure-commit',name:'Secure commit',version:'2.4.0',summary:'Check releases.',targets:['Codex'],
    ring:'pilot',updated:'now',owner:'Mina',secrets:[],instructions:'Run tests.',adapters:{Codex:'Read AGENTS.md.'},
    git_url:'https://github.com/example/repo',git_commit:'7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2',
    package_digest:'a'.repeat(64),repositories:['atlas-api'],approved_by:'Nora',approved_at:'now'
  };
  await page.route('**/api/skills', route => route.fulfill({json:[skill]}));
  await page.route('**/api/receipts', route => route.fulfill({json:[]}));
  await page.route('**/api/skills/secure-commit/ring', route => route.fulfill({status:429,headers:{'Retry-After':'1'},json:{error:'Too many requests. Wait one second.'}}));
  await page.goto('/registry');
  await page.getByRole('button', {name:'Draft',exact:true}).click();
  await expect(page.getByText('Release unchanged. Too many requests. Wait one second.')).toBeVisible();
  await expect(page.getByRole('button', {name:'Pilot',exact:true})).toHaveAttribute('aria-pressed','true');
  await expect(page.getByRole('button', {name:'Draft',exact:true})).toHaveAttribute('aria-pressed','false');
});
