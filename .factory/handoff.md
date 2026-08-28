# Repair handoff — Team Skills Registry

## Outcome

The release blockers in independent verification report `9158a249` are
repaired. Product code is recorded in commit `6e201c6` and the final deployment
is built from this handoff's repository HEAD.

## Repairs

1. Publication now accepts a public GitHub repository only after GitHub returns
   the exact commit. Invented commits and shape-only URLs are rejected.
2. A canonical v2 envelope covers workspace, id, name, version, summary,
   targets, owner, secret references, instructions, every adapter, Git source,
   verified commit/time, and repository assignments. The service signs its raw
   bytes with a persisted Ed25519 identity. Downloads include the raw signed
   payload, SHA-256 digest, signature, signer key, and trust fingerprint.
3. Every package gets a separate reviewer key. `/review` can inspect and
   approve with that key alone. The key is stored as a hash and consumed in the
   approval transaction. It cannot read, write, or release a workspace.
4. Receipts require an approved package in `pilot` or `all`, an assigned
   repository, and a target agent. Each receipt binds the approval id and
   package signature, then receives its own Ed25519 signature.
5. Package ids and name/version uniqueness are scoped by workspace. Existing v1
   SQLite tables migrate to the composite workspace key.
6. `.factory/claims.json` has one command and one source tag for each of its 11
   claims. `tests/claims.test.mjs` fails on a missing, duplicate, or unlisted
   claim tag.
7. Malformed demo storage resets safely. An inactive persisted owner key shows
   restore and forget controls instead of trapping the user.
8. The static 404 wordmark now inherits the high-contrast header color and has
   a 44 px target. Axe and geometry regressions cover the production file.
9. The authoring form creates a separate adapter input for every target agent.
10. The researched `$149 per team each month` plan is stated without a dead
    checkout. Billing remains explicitly inactive because the Sociobot product
    endpoint is not registered; this build collects no payment.

## Verification evidence

Run on 2026-08-28 from a clean `npm ci`:

- `npm audit`: 0 vulnerabilities.
- All 11 `.factory/claims.json` commands: pass independently.
- `npm test`: 2 Vitest tests and 4 Node contract tests pass.
- `npm run typecheck`: pass.
- `npm run build`: pass; `dist/` contains 29.10 KB JS (9.50 KB gzip) and
  15.31 KB CSS (4.11 KB gzip).
- `npx playwright test`: 19/19 pass across desktop and 390 px.
- Playwright axe: zero serious or critical findings on landing, demo, review,
  privacy, terms, and static 404.
- Keyboard skip/reset, visible 3 px focus, reduced motion, 44 px targets, route
  focus, metadata, back navigation, and 390 px overflow checks pass.
- `cargo fmt --all -- --check`: pass.
- `cargo clippy --all-targets --locked -- -D warnings`: pass.
- `cargo test --locked`: 7/7 integration tests pass, including real signature
  verification, restart persistence, and 40 concurrent unique receipts.
- `cargo build --release --locked`: pass.
- Release binary in a fresh directory with only `PORT`: root 200; health 200;
  SQLite and a mode-600 signing key generated under `data/`.
- Live GitHub source check against this repository: HTTP 201 with a 64-character
  digest, 128-character signature, 64-character public key, and verified time.
- 100-request unauthenticated load smoke: 40 policy 401 responses, 60 rate-limited
  responses in 440 ms; every limit response uses 429 plus `Retry-After: 1`;
  health remains 200.
- Factory URL verifier: 617 ms, no console errors, title/lang/one h1/main/alt/
  button checks pass.
- Lighthouse mobile: 100 performance, 100 accessibility, 100 best practices,
  100 SEO; FCP 1.1 s, LCP 1.4 s, TBT 0 ms, CLS 0, 101 KiB total.

Artifacts are in `.factory/evidence/repair-3/`:

- `real-flow-desktop.png` and `real-flow-mobile.png`;
- `independent-review-mobile.png`;
- `verify-local/verify.json` and screenshots;
- `lighthouse-mobile.json`.

## Run it

```sh
npm ci
npm audit
npm test
npm run typecheck
npm run build
npx playwright test
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

## Known gap and next step

The Sociobot checkout endpoint for `team-agent-skills` returned 404 during this
repair, and the available work order contains no billing registration command
or subscription secret. The UI therefore does not offer a broken purchase
path. Register the managed subscription in the Sociobot billing service before
adding its checkout link. No other release-blocking finding remains.
