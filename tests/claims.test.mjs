import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('every listed claim has exactly one matching source tag and command', async () => {
  const claims = JSON.parse(await readFile(new URL('../.factory/claims.json', import.meta.url), 'utf8'));
  const sources = await Promise.all([
    '../e2e/demo.spec.ts',
    '../e2e/site.spec.ts',
    '../tests/api.rs'
  ].map(path => readFile(new URL(path, import.meta.url), 'utf8')));
  const combined = sources.join('\n');
  const listed = new Set(claims.map(claim => claim.id));
  assert.equal(listed.size, claims.length, 'claim ids must be unique');
  for (const claim of claims) {
    const tag = `@claim:${claim.id}`;
    assert.equal(combined.split(tag).length - 1, 1, `${tag} must occur exactly once`);
    assert.match(claim.test, new RegExp(claim.id.replaceAll('-', '[-_]')), `${claim.id} command must select its own test`);
  }
  const discovered = [...combined.matchAll(/@claim:([a-z0-9-]+)/g)].map(match => match[1]);
  assert.deepEqual(new Set(discovered), listed, 'unlisted claim tags are not allowed');
});
