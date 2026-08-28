# Handoff — Team Skills Registry repair

## Repair delivered

- Reproduced the release blocker with the candidate lockfile:
  `cargo +1.85.1 build --release --locked` rejects ICU 2.3 (Rust 1.88
  required) and `idna_adapter` (Rust 1.86 required).
- Raised the documented and enforced compiler floor to Rust 1.88. The
  Dockerfile uses `rust:1.88-slim`, builds with `--locked`, and the manifest
  declares `rust-version = "1.88"`. Node builds use `npm ci`.
- Added `.dockerignore`; the final ACR upload was 2.473 MiB (2.981 MB Docker
  context), excluding local dependencies, build output, data, logs, and Git.
- Fixed a first-load console-error regression: landing and legal pages no
  longer request workspace APIs. The real registry still loads its API and the
  isolated demo remains local-only.
- Fixed accessibility findings: the coral token now meets text contrast, the
  horizontally scrollable receipt table is named, keyboard-focusable, and has a
  focus ring, and client-side navigation moves focus to the page heading and
  announces the new title.

## Regression coverage

- `tests/dockerfile.test.mjs` checks the Rust >=1.88 Docker backend stage,
  locked Cargo build, runtime arguments/non-root/port contract, and Docker
  context exclusions. It runs in `npm test`.
- `tests/api.rs` launches the compiled server, verifies `/health` returns its
  build SHA, then verifies 40 successful forwarded-IP requests, a 429 with
  `Retry-After: 1`, and a separate forwarded IP still succeeds.
- `e2e/site.spec.ts` verifies 390px keyboard skip navigation, no landing API
  requests or console errors, and focus plus live announcement on SPA routing.
- `e2e/accessibility.spec.ts` injects axe-core through Playwright Chromium and
  checks no serious/critical violations on desktop landing and 390px demo,
  privacy, and terms routes.

## Verification evidence

Ran from a clean `npm ci` install:

```sh
npm test                         # 2 Vitest + 3 Docker contract tests passed
npm run typecheck                # passed
npm run build                    # dist/ created; JS 5.95 KB gzip, CSS 3.53 KB gzip
cargo +1.88.0 test --locked      # passed; runtime health/rate-limit test passed
cargo test --locked              # passed with the default toolchain
npx playwright test              # 6 passed (desktop, 390px, keyboard, demo, privacy, axe)
```

Production-binary smoke test (same release binary copied into the runtime
image) on port 8080:

- `/health` returned `{"status":"ok","build_sha":"repair-qa"}`.
- 100 parallel `/api/skills` requests using one `X-Forwarded-For` value:
  40 returned 200 and 60 returned 429. The next response included
  `Retry-After: 1`.
- `verify-url.sh` recorded 538 ms load, no console errors, a title and
  language, one h1, one main landmark, and no missing image alt text.
- The standalone axe CLI could not start because this runner has no system
  Chrome. The Playwright axe regression suite above uses the preinstalled
  Chromium and passed.

ACR remote multi-stage build passed (run `chk9`) for source commit
`2c16b21bbea399677c1abcb81474fa3ab5c30979`:

```text
sociobotregistry.azurecr.io/team-agent-skills:2c16b21
sha256:511e66fe434ae8a0fccd8496a5a5e1e985a48441920f989780898ff66a73afd1
```

Its logs show `npm ci`, `cargo build --release --locked` succeeding under
`rust:1.88-slim`, and a non-root `app` runtime exposing 8080.

## Run and deploy

```sh
npm ci
npm run build
cargo +1.88.0 run --locked
```

The pushed image is ready for the factory container deployment on `PORT=8080`.
The Azure subscription has no `sf-team-agent-skills` Container App or other
product-specific runtime target, so no live revision could be created without
creating infrastructure (outside repository authority). The required image is
in ACR at the digest above.
