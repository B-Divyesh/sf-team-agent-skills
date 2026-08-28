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
