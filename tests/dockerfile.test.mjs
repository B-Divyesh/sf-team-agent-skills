import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('Docker backend stage follows the factory stable toolchain contract', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(dockerfile, /^FROM rust:1-slim AS backend$/m, 'the Dockerfile must track current stable Rust');
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

test('Docker context excludes local build output and private runtime data', async () => {
  const ignore = await readFile(new URL('../.dockerignore', import.meta.url), 'utf8');
  for (const entry of ['.git', 'node_modules', 'target', 'dist', 'data', '.env']) {
    assert.match(ignore, new RegExp(`^${entry.replace('.', '\\.').replace('/', '\\/')}$`, 'm'));
  }
});
