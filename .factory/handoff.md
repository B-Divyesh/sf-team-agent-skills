# Repair handoff — Team Skills Registry

## Outcome

Every release-blocking finding in verifier commit
`0a82dcade6df6625830bcf4c08f3b47dcb0a42c4` was repaired. The product remains
a Vite/TypeScript frontend served by one Rust/axum container on port 8080.

## Product and security repairs

- Real registry routes now require a 192-bit capability key. Only SHA-256
  hashes are stored in SQLite. Every query and mutation is scoped to its
  workspace.
- Workspace creation returns separate owner and reviewer keys. A valid reviewer
  key is required to approve a version. Another workspace cannot read, approve,
  release, install, or receipt the package.
- Published versions now contain instructions, per-agent adapter content, Git
  source and 40-character commit, owner, targets, uppercase secret references,
  assigned repositories, and a server-computed package digest.
- Versions are immutable. Approval creates an audit row. Pilot and full release
  are rejected until approval exists.
- Assigned repositories can download the complete reviewed pilot/full package
  from `GET /api/repositories/:repository/install/:id`.
- Receipts snapshot the skill name, version, package digest, repository, agent,
  ring, and time. Reads no longer join mutable skill metadata.
- Rollout UI changes only after a successful response. HTTP 4xx/429 keeps the
  prior ring and gives a recovery message.
- Secret-like values and mixed-case strings are rejected from the secret
  reference field.

The unavailable external `$149` paid offer and dead checkout were removed.
There is no paid gate or license claim in this release.

## UX, accessibility, and platform repairs

- Publishing is a labeled form for the complete package, replacing native
  prompts and hard-coded metadata.
- Leaving demo deletes `demo:team-agent-skills:v2` and clears its notice.
- Selected packages and release rings expose `aria-pressed`.
- Navigation, banner, ring, form, and footer controls meet the 44px target.
  Mobile rings wrap in a 2×2 grid without page overflow.
- Every route updates title, description, canonical, Open Graph, and Twitter
  metadata. The server 404 now has the standard header, navigation, footer,
  skip link, and build version.
- Responses include CSP, HSTS, `nosniff`, referrer, and permissions headers.
  Hashed assets cache for one year; original image assets cache for one week.
- Docker now uses `rust:1-slim`. Rustfmt and strict Clippy pass.
- Vite/Vitest/TypeScript were updated; `npm audit` reports zero findings.
- Claims, demo documentation, copy audit, README, privacy, and terms now match
  the shipped behavior.

## Exact regression coverage

- `tests/api.rs`: unauthenticated 401, tenant isolation, hashed owner/reviewer
  keys, secret-value rejection, complete immutable package, forged reviewer
  403, cross-tenant approval rejection, approval-before-release, repository
  assignment 403, install payload, receipt snapshot, forwarded-IP 429, and
  `Retry-After`.
- `e2e/site.spec.ts`: 429 rollout rollback, route metadata, keyboard skip and
  route focus.
- `e2e/demo.spec.ts`: all claim flows, demo teardown, review gating, and
  downloaded package content.
- `e2e/accessibility.spec.ts`: axe desktop/mobile, state exposure, 44px
  geometry, and 390px fit.
- `tests/dockerfile.test.mjs`: stable Rust tag and runtime/image contracts.

## Verification evidence

Clean/local gates on 2026-08-28:

```text
npm ci                                            PASS
npm audit                                         PASS — 0 vulnerabilities
npm test                                          PASS — 2 Vitest + 3 contract tests
npm run typecheck                                 PASS
npm run build                                     PASS — dist/
npx playwright test                               PASS — 12/12
cargo fmt --all -- --check                        PASS
cargo clippy --all-targets --locked -- -D warnings PASS
cargo test --locked                               PASS — API integration
cargo build --release --locked                    PASS
```

Production release binary at `http://127.0.0.1:4180`:

- `verify-url.sh`: 606 ms, zero console/page errors, title, `lang=en`, one
  h1, main, zero missing alt, zero unlabeled buttons.
- Real browser workflow: create → publish → approve with separate key → pilot →
  receipt. Result: one package, one receipt, zero console errors.
- Desktop and 390×844 screenshots:
  `.factory/evidence/repair-local/real-flow-desktop.png` and
  `.factory/evidence/repair-local/real-flow-mobile.png`.
- 390px page overflow: false. Reduced motion and keyboard checks pass in
  Playwright.
- Mobile Lighthouse: performance 100, accessibility 100, best practices 100,
  SEO 100; FCP 1.1s, LCP 1.7s, TBT 0ms, CLS 0.
- Desktop Lighthouse: 100/100/100/100; FCP 0.3s, LCP 0.4s, TBT 0ms, CLS 0.
- Production output: JS 23.82 KB raw / 8.12 KB gzip; CSS 14.50 KB raw /
  3.95 KB gzip.
- API response inspection: public `/api/skills` is 401; `/health` is 200;
  static JS sends one-year immutable caching; security headers are present.

The Playwright axe integration reports zero serious/critical findings across
landing, demo, privacy, and terms. The standalone axe CLI could not pair its
downloaded ChromeDriver 152 with the preinstalled Chromium 145; the identical
axe-core 4.11.0 integration and Lighthouse accessibility audits both pass.

## Deploy and operate

```sh
/opt/fleet/lib/deploy-container.sh team-agent-skills /work/repo Dockerfile 8080
/opt/fleet/lib/verify-url.sh https://team-agent-skills.sociobot.in .factory/evidence/verify-url
curl -fsS https://team-agent-skills.sociobot.in/health
```

The container starts with only `PORT`, creates `/data/registry.db`, runs as
UID 10001, and reports the build commit at `/health`.

## Known gaps and next steps

No release blocker remains. Capability keys intentionally avoid an account
dependency and recovery service. A future account-based edition can add
Sociobot Entra for key recovery and named organization membership without
changing the package or receipt schema.
