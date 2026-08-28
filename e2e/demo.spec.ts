import { expect, test } from '@playwright/test';

test('@claim:demo-separation demo changes reset to the separate sample workspace', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Review', exact: true }).click();
  await expect(page.getByText('Secure commit is now in review.')).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByText('Demo reset. The sample packages are back.')).toBeVisible();
  await expect(page.getByText('Released everywhere')).toBeVisible();
  const keys = await page.evaluate(() => Object.keys(localStorage));
  expect(keys).toContain('demo:team-agent-skills:v2');
  expect(keys.some(key => key.startsWith('real:'))).toBe(false);
});

test('@claim:execution-receipt demo records the selected version and repository', async ({ page }) => {
  await page.goto('/demo');
  await page.getByLabel('Repository').selectOption('atlas-api');
  await page.getByRole('button', { name: 'Record execution receipt' }).click();
  await expect(page.getByText('Recorded rcpt-')).toBeVisible();
  const row = page.locator('tbody tr').first();
  await expect(row).toContainText('Secure commit');
  await expect(row).toContainText('v2.4.0');
  await expect(row).toContainText('atlas-api');
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

test('leaving demo discards its storage and does not leak its notice', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await page.getByRole('link', { name: 'Start for real' }).click();
  await expect(page).toHaveURL('/registry');
  await expect(page.getByText('Demo reset. The sample packages are back.')).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem('demo:team-agent-skills:v2'))).toBeNull();
});

test('@claim:review-required new versions cannot enter pilot before review', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Publish a version' }).click();
  await page.getByLabel('Skill name').fill('Dependency check');
  await page.getByLabel('Summary').fill('Check dependency changes before release.');
  await page.getByLabel('Owner').fill('Mina Patel');
  await page.getByLabel('Instructions', { exact: true }).fill('Run the dependency audit and record every changed package.');
  await page.getByLabel('Codex adapter', { exact: true }).fill('Run the repository audit command.');
  await page.getByLabel('Git commit SHA').fill('7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2');
  await page.getByRole('button', { name: 'Publish draft version' }).click();
  await page.getByRole('button', { name: 'Pilot', exact: true }).click();
  await expect(page.getByText('Approve this exact version before release.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Draft', exact: true })).toHaveAttribute('aria-pressed', 'true');
});

test('malformed demo storage recovers to a resettable sample workspace', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('demo:team-agent-skills:v2', '{broken json'));
  await page.goto('/demo');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Review skill releases in one place');
  await expect(page.getByText('Damaged demo data was reset to a clean sample.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Reset demo' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Secure commit' })).toBeVisible();
});

test('publish form collects different adapter content for each target', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Publish a version' }).click();
  await page.getByLabel('Target agents').fill('Codex, Claude Code');
  await page.getByLabel('Codex adapter').fill('Read AGENTS.md.');
  await page.getByLabel('Claude Code adapter').fill('Read CLAUDE.md.');
  await page.getByLabel('Skill name').fill('Adapter check');
  await page.getByLabel('Summary').fill('Keep each agent adapter distinct.');
  await page.getByLabel('Owner').fill('Mina Patel');
  await page.getByLabel('Instructions', { exact: true }).fill('Run the repository checks.');
  await page.getByLabel('Git commit SHA').fill('7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2');
  await page.getByRole('button', { name: 'Publish draft version' }).click();
  await expect(page.locator('.package-content')).toContainText('Codex: Read AGENTS.md.');
  await expect(page.locator('.package-content')).toContainText('Claude Code: Read CLAUDE.md.');
});

test('@claim:package-contents downloaded package contains exact source and adapter data', async ({ page }) => {
  await page.goto('/demo');
  const [download] = await Promise.all([
    page.waitForEvent('download'),
    page.getByRole('button', { name: 'Download assigned package' }).click()
  ]);
  const content = await (await import('node:fs/promises')).readFile(await download.path() as string, 'utf8');
  const payload = JSON.parse(content);
  expect(payload.repository).toBe('atlas-api');
  expect(payload.package.version).toBe('2.4.0');
  expect(payload.package.instructions).toContain('Inspect the staged diff');
  expect(payload.package.adapters.Codex).toContain('AGENTS.md');
  expect(payload.package.git_commit).toHaveLength(40);
  expect(payload.package.package_digest).toHaveLength(64);
  expect(payload.package.package_signature).toHaveLength(128);
});
