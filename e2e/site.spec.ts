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
  const primary = await page.getByRole('link', { name: 'Try it with sample data' }).boundingBox();
  expect(primary?.y).toBeLessThan(844);
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

test('@claim:managed-plan-status pricing is exact and does not expose an inactive checkout', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: '$149 per team each month' })).toBeVisible();
  await expect(page.getByText('Managed billing is not active in this release.')).toBeVisible();
  await expect(page.getByRole('link', { name: /buy|checkout|subscribe/i })).toHaveCount(0);
  await expect(page.getByRole('button', { name: /buy|checkout|subscribe/i })).toHaveCount(0);
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

test('inactive persisted owner key has visible recovery controls', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('team-agent-skills:workspace-key', 'tsr_stale_workspace_key_12345678901234567890'));
  await page.route('**/api/skills', route => route.fulfill({ status: 401, json: { error: 'That workspace key is not active.' } }));
  await page.route('**/api/receipts', route => route.fulfill({ status: 401, json: { error: 'That workspace key is not active.' } }));
  await page.goto('/registry');
  await expect(page.getByText('That workspace key is not active.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Forget inactive key' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Restore workspace' })).toBeVisible();
  await page.getByRole('button', { name: 'Forget inactive key' }).click();
  expect(await page.evaluate(() => localStorage.getItem('team-agent-skills:workspace-key'))).toBeNull();
});

test('review route uses only its one-time package key', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('team-agent-skills:workspace-key', 'tsr_owner_key_123456789012345678901234'));
  let reviewAuthorization = '';
  await page.route('**/api/review', async route => {
    reviewAuthorization = await route.request().headerValue('authorization') || '';
    await route.fulfill({ json: {
      id:'review-me',name:'Review me',version:'1.2.3',summary:'Check this package.',targets:['Codex'],ring:'draft',updated:'now',owner:'Mina',secrets:[],instructions:'Run tests.',adapters:{Codex:'Read AGENTS.md.'},git_url:'https://github.com/example/repo',git_commit:'7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2',package_digest:'a'.repeat(64),package_signature:'b'.repeat(128),signer_public_key:'c'.repeat(64),repositories:['atlas-api'],approved_by:null,approved_at:null
    }});
  });
  await page.goto('/review');
  await page.getByLabel('One-time reviewer key').fill('tsr_review_package_key_12345678901234567890');
  await page.getByRole('button', { name: 'Open package for review' }).click();
  await expect(page.getByRole('heading', { name: 'Review me v1.2.3' })).toBeVisible();
  expect(reviewAuthorization).toBe('Bearer tsr_review_package_key_12345678901234567890');
});
