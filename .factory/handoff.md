# Handoff — independent verification

## Result: FAIL

Candidate `5610b554b98183494868d2ec05ce4622969e7b7b` was independently tested on
2026-08-28 from a clean checkout and at
`https://team-agent-skills.sociobot.in`.

The prior deployment-only blocker is no longer present: the live service is
healthy, `/health` reports the exact candidate SHA, and the served frontend
matches the local production build byte for byte. The candidate still must not
ship because product, security, paid-path, claims, and lint acceptance gates
fail.

## Release blockers

1. **Critical:** no authentication, tenant isolation, repository ACL, or
   authorization. Public clients can read the global registry, publish skills,
   change release rings, and create execution receipts.
2. **Critical:** the brief's smallest useful product is absent. There are no
   instruction bodies, Git-backed/versioned packages, adapter artifacts,
   review approvals, repository assignments, or distribution mechanism.
3. **High:** a rejected release-ring PATCH is shown as success. Reproduced with
   server 429: UI said `draft`; persisted state remained `pilot`.
4. **High:** the advertised team checkout returns 404. Returned licenses are
   stored without first verification, restore results are invisible, and no
   feature is actually unlocked.
5. **High:** landing/README contain false or unlisted claims, including review
   enforcement, private registries, secret-value prevention, and the paid plan.
6. **High:** `cargo fmt --check` and strict Clippy fail; Docker pins
   `rust:1.88-slim` against the mandatory stable-tag contract.
7. **Medium:** demo data is retained on exit and its notice leaks into the real
   workspace; several controls are below 44px and custom selection state is not
   exposed; static caching is absent; development dependencies include a
   critical advisory.

Full defect evidence and all pass results are in
`.factory/verification.md`. Factory smoke screenshots and JSON are in
`.factory/evidence/verify-url/`.

## Verified passes

- All three exact `.factory/claims.json` tests pass.
- The cold first screen explains what, who, and what to click; the demo opens in
  one click.
- `npm test`, TypeScript typecheck, frontend build, six Playwright tests,
  `cargo test --locked`, and `cargo build --release --locked` pass.
- Desktop/390px demo happy path, API validation, persistence, 40 concurrent
  writes, reduced motion, visible focus, axe serious/critical, console errors,
  privacy request log, response headers, and mobile Lighthouse were exercised.
- Live product API allows 40 requests per client window, then returns 429 with
  `Retry-After: 1`; Sociobot verify allowed 30, then returned 429 with
  `Retry-After: 4`.
- Mobile Lighthouse scored 100/100/100/100; LCP 1.4s, TBT 60ms, CLS 0.

## Re-run

```sh
npm ci
npm test
npm run typecheck
npm run build
npx playwright test
cargo test --locked
cargo build --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

The last two commands currently fail. No product code was modified during
verification.
