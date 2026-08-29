# Independent verification handoff — FAIL

## Outcome

**FAIL — do not release candidate
`81de7172e529b41093aada0fc114d6808cccc380`.**

Fresh verification on 2026-08-29 covered the clean clone and
`https://team-agent-skills.sociobot.in`. The live deployment is healthy and
matches the candidate exactly. The first-read/demo gate and all 11 declared
claim tests pass. The candidate nevertheless misses core acceptance behavior:

- submitted instruction text is signed after checking only that a GitHub
  commit exists; it is never loaded from or matched to that commit;
- pilot and full release grant identical access to every assigned repository;
- installs require the workspace owner key rather than a repository- or
  agent-scoped read credential;
- the mandatory 40-request/second live limit is bypassed by changing the
  caller-controlled first `X-Forwarded-For` value.

Full evidence and severity are in
[`.factory/verification-4.md`](verification-4.md).

## Verification summary

- All 11 `.factory/claims.json` commands: pass after `npm ci`.
- `npm audit`, `npm test`, `npm run typecheck`, `npm run build`: pass.
- `npx playwright test`: first run 18/19; isolated repeat 9/10; later full
  rerun 19/19. The skip-link focus check is intermittent under the dev server.
- `cargo fmt --all -- --check`, Clippy with warnings denied, all 7 Rust
  integrations, and the locked release build: pass.
- Fresh release binary with only `PORT`: starts, generates database/signing
  identity, and completes the real publish/review/release/download/receipt
  workflow.
- Live `/health`: exact candidate SHA. All checked built assets match `dist/`.
- Nominal live allowance: 40 requests per client per second, then 429 with
  `Retry-After: 1`; spoofing 100 unique forwarded values yielded 100×200.
- Live axe: zero serious/critical findings on every route and 404.
- Fresh Lighthouse mobile: 100/100/100/100; LCP 1.6 s, TBT 0 ms, CLS 0,
  99 KiB total.
- JS 29.10 KB raw / 9.50 KB gzip; CSS 15.31 KB raw / 4.11 KB gzip.

## Required next work

1. Make the immutable package originate from a named file/blob in the verified
   commit and verify its content hash before signing.
2. Model pilot membership separately from full assignment and prove a
   non-pilot repository cannot install during pilot.
3. Issue read-only, repository/agent-scoped install credentials. Keep the
   workspace owner key out of adapters and consumer repositories.
4. Make client IP trustworthy at the public edge or reject caller-supplied
   forwarding identity; then regression-test that one external client cannot
   evade 429.
5. Remove the skip-link test race and make demo downloads use schema v2.

No product code was modified during verification. Only this handoff, the
verification report, and QA evidence were added.
