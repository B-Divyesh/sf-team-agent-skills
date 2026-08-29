import axe from 'axe-core';
import { expect, test } from '@playwright/test';

async function expectNoSeriousAxeViolations(page: import('@playwright/test').Page, path: string, viewport: { width: number; height: number }) {
  await page.setViewportSize(viewport);
  await page.goto(path);
  await page.addScriptTag({ content: axe.source });
  const violations = await page.evaluate(async () => {
    const results = await window.axe.run(document, {
      resultTypes: ['violations'],
      rules: { 'color-contrast': { enabled: true } }
    });
    return results.violations
      .filter(violation => violation.impact === 'serious' || violation.impact === 'critical')
      .map(violation => ({ id: violation.id, nodes: violation.nodes.map(node => node.target) }));
  });
  expect(violations, `${path} at ${viewport.width}px`).toEqual([]);
}

test('accessibility smoke has no serious or critical axe violations on desktop and mobile', async ({ page }) => {
  await expectNoSeriousAxeViolations(page, '/', { width: 1366, height: 900 });
  await expectNoSeriousAxeViolations(page, '/demo', { width: 390, height: 844 });
  await expectNoSeriousAxeViolations(page, '/privacy', { width: 390, height: 844 });
  await expectNoSeriousAxeViolations(page, '/terms', { width: 390, height: 844 });
  await expectNoSeriousAxeViolations(page, '/review', { width: 390, height: 844 });
  await expectNoSeriousAxeViolations(page, '/404.html', { width: 390, height: 844 });
});

test('static 404 home link has contrast and a 44px target', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/404.html');
  const box = await page.getByRole('link', { name: 'Team Skills Registry home' }).boundingBox();
  expect(box?.height).toBeGreaterThanOrEqual(44);
  await expect(page).toHaveTitle('Page not found — Team Skills Registry');
});

test('demo supports reduced motion, keyboard reset, and 390px layout', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  const layout = await page.evaluate(() => ({
    overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    scrollBehavior: getComputedStyle(document.documentElement).scrollBehavior,
    moving: Array.from(document.querySelectorAll('*')).filter(element => {
      const style=getComputedStyle(element);
      return style.animationName !== 'none' || style.transitionDuration !== '0s';
    }).length
  }));
  expect(layout).toEqual({ overflow: 0, scrollBehavior: 'auto', moving: 0 });
  const reset = page.getByRole('button', { name: 'Reset demo' });
  await reset.focus();
  const outline = await reset.evaluate(element => getComputedStyle(element).outlineWidth);
  expect(parseFloat(outline)).toBeGreaterThanOrEqual(3);
  await page.keyboard.press('Space');
  await expect(page.getByText('Demo reset. The sample packages are back.')).toBeVisible();
});

test('mobile controls expose state, fit, and meet the 44px target', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  const controls = page.locator('nav a, .demo-banner button, .demo-banner a, .track button, .receipt-form select, .receipt-form button, footer a');
  const boxes = await controls.evaluateAll(elements => elements.map(element => {
    const box = element.getBoundingClientRect();
    return { text: element.textContent?.trim(), width: box.width, height: box.height, right: box.right };
  }));
  expect(boxes.every(box => box.height >= 44 && box.right <= 390), JSON.stringify(boxes)).toBe(true);
  await expect(page.getByRole('button', { name:'Secure commit' })).toHaveAttribute('aria-pressed','true');
  await expect(page.getByRole('button', { name:'Release to all assigned repositories' })).toHaveAttribute('aria-pressed','true');
});

for (const width of [390, 1366]) {
  test(`publish textareas use the designed focus ring at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/demo');
    await page.getByRole('button', { name: 'Publish a version' }).click();
    for (const textarea of await page.locator('.publish-form textarea').all()) {
      await textarea.focus();
      const style = await textarea.evaluate(element => ({ width: getComputedStyle(element).outlineWidth, color: getComputedStyle(element).outlineColor }));
      expect(parseFloat(style.width)).toBeGreaterThanOrEqual(3);
      expect(style.color).not.toBe('rgb(0, 0, 0)');
    }
  });
}
