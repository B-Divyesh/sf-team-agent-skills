import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('every listed claim has exactly one matching source tag and command', async () => {
  const claims = JSON.parse(await readFile(new URL('../.factory/claims.json', import.meta.url), 'utf8'));
  const sources = await Promise.all([
    '../e2e/demo.spec.ts',
    '../e2e/site.spec.ts',
    '../tests/api.rs',
    '../tests/claims.test.mjs'
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

test('@claim:mit-license-build self-hosted build is MIT licensed and has a production build command', async () => {
  const license = await readFile(new URL('../LICENSE', import.meta.url), 'utf8');
  const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(license, /Permission is hereby granted, free of charge/);
  assert.match(license, /THE SOFTWARE IS PROVIDED "AS IS"/);
  assert.equal(packageJson.scripts.build, 'vite build');
  assert.match(dockerfile, /RUN npm run build/);
  assert.match(dockerfile, /RUN cargo build --release --locked/);
});
