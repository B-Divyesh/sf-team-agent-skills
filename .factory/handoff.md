# Verification handoff — Team Skills Registry

## Outcome

**PASS — candidate `fcb5b047e2d3a1d7c41c3e860d2332c22d932f33` is ready for release.**

Independent verification completed on 2026-08-29 against
`https://team-agent-skills.sociobot.in`. Live `/health` reports the exact
candidate SHA, and local/deployed frontend hashes match. The prior release
blockers are resolved: rate limits are independent per trusted first-hop client,
and private Git sources use repository-bound deployment credentials without
storing tokens.

The full report is [`.factory/verification-6.md`](verification-6.md).

## What was verified

- All 15 exact claim commands pass after `npm ci`.
- `npm audit`, unit/contract tests, TypeScript, production Vite build, and all
  19 Playwright tests pass.
- Rust format, Clippy with warnings denied, all 11 integration tests, and the
  optimized release build pass.
- Fresh local and live authenticated workflows complete publish → independent
  review → pilot → scoped install/download → receipt → full release.
- A 41-request local and live burst confirms an allowance of 40 requests per
  client per second, then 429 with `Retry-After: 1`; another client remains
  unaffected.
- Fresh live request logging is same-origin only. Security headers, routing,
  404 behavior, immutable asset caching, persistence, and exact build identity
  pass.
- Fresh axe checks report zero serious/critical issues on all routes. Desktop,
  390px mobile, keyboard-only, 200% text, and reduced-motion checks pass.
- Mobile Lighthouse: performance 99, accessibility 100, best practices 100,
  SEO 100; LCP 1.5 s, TBT 100 ms, CLS 0, total transfer 101 KiB.

## Reproduce

```sh
npm ci
npm audit --audit-level=high
npm test
npm run typecheck
npm run build
npx playwright test
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

Then run every exact `test` command in `.factory/claims.json`. Start the release
binary with only `PORT=8080`; open `/demo` for the isolated sandbox and
`/registry` for a private workspace.

## Known gaps

- Low: publish-form textareas use the browser's visible native focus outline
  instead of the custom 3px focus style used by the other controls.
- No Docker-compatible engine was present in the verifier container. The two
  production build stages and Docker contract tests passed, and the live
  container identifies as the exact candidate.

No product code was changed during verification.
