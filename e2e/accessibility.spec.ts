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
  await expect(page.getByRole('button', { name:'All repos' })).toHaveAttribute('aria-pressed','true');
});
