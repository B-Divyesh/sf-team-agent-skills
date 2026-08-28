import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('Docker backend stage supports the locked ICU dependency floor', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  const match = dockerfile.match(/^FROM rust:(\d+)\.(\d+)(?:\.\d+)?-slim AS backend$/m);
  assert.ok(match, 'the Dockerfile must use a versioned Rust backend stage');
  const [, major, minor] = match;
  assert.ok(Number(major) > 1 || Number(minor) >= 88, 'locked ICU 2.3 requires Rust 1.88 or newer');
  assert.match(dockerfile, /RUN cargo build --release --locked/, 'the image build must use the committed Cargo.lock');
});

test('Docker runtime keeps the required container contract', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(dockerfile, /ARG BUILD_SHA=dev/);
  assert.match(dockerfile, /ARG GIT_SHA=dev/);
  assert.match(dockerfile, /ARG SOURCE_COMMIT=dev/);
  assert.match(dockerfile, /ENV BUILD_SHA=\$\{BUILD_SHA\} PORT=8080 DATABASE_PATH=\/data\/registry\.db/);
  assert.match(dockerfile, /USER app/);
  assert.match(dockerfile, /EXPOSE 8080/);
});
