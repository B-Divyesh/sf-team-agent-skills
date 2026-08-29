# Independent product verification 6 — PASS

Verified 2026-08-29 against candidate
`fcb5b047e2d3a1d7c41c3e860d2332c22d932f33` and
`https://team-agent-skills.sociobot.in`.

## Decision

**PASS. This candidate is ready for release.** Fresh live `/health` reports the
exact candidate SHA, the built frontend files are byte-for-byte identical to
the deployed files, all 15 declared claims pass after the documented lockfile
install, and the smallest useful Git-backed review/release/install/receipt flow
works both locally and on the deployed service.

No critical, high, or medium defects remain. One low-severity focus-style
consistency observation is listed below.

## First-read and one-click demo — PASS

A cold, storage-free desktop visit answered all three required questions above
the fold:

- What it does: “Release reviewed skills across repositories.”
- Who it is for: “For engineering leads who need one checked instruction set
  for every coding agent.”
- What to do first: “Try it with sample data,” followed by “Open three complete
  skill packages and their review records.”

The action is visible without scrolling. One click opens `/demo`, which already
contains three realistic packages and receipts. The persistent banner says
“Demo — sample data, nothing is saved” and exposes **Reset demo** and **Start
for real**. The only storage key was `demo:team-agent-skills:v2`, and the live
browser contacted only the product origin. Evidence:
[`first-read/live-home-cold.png`](qa-artifacts/first-read/live-home-cold.png) and
[`live-browser-audit.json`](qa-artifacts/live-browser-audit.json).

## Declared claims — PASS (15/15)

`.factory/claims.json` exists. After `npm ci`, every exact command from the
manifest was run independently from the unchanged candidate checkout.

| Claim id | Exact command | Result |
| --- | --- | --- |
| `demo-separation` | `npx playwright test --grep @claim:demo-separation` | PASS, 1 test |
| `execution-receipt` | `npx playwright test --grep @claim:execution-receipt` | PASS, 1 test |
| `demo-local-data` | `npx playwright test --grep @claim:demo-local-data` | PASS, 1 test |
| `review-required` | `npx playwright test --grep @claim:review-required` | PASS, 1 test |
| `package-contents` | `npx playwright test --grep @claim:package-contents` | PASS, 1 test |
| `private-workspace` | `cargo test --locked claim_private_workspace` | PASS, 1 test |
| `secret-reference-format` | `cargo test --locked claim_secret_reference_format` | PASS, 1 test |
| `git-signed-package` | `cargo test --locked claim_git_signed_package` | PASS, 1 test |
| `independent-one-time-review` | `cargo test --locked claim_independent_one_time_review` | PASS, 1 test |
| `governed-execution-receipt` | `cargo test --locked claim_governed_execution_receipt` | PASS, 1 test |
| `pilot-ring-access` | `cargo test --locked claim_pilot_ring_access` | PASS, 1 test |
| `scoped-install-credentials` | `cargo test --locked claim_scoped_install_credentials` | PASS, 1 test |
| `trusted-client-rate-limit` | `cargo test --locked claim_trusted_client_rate_limit` | PASS, 1 test |
| `private-git-source` | `cargo test --locked claim_private_git_source` | PASS, 1 test |
| `managed-plan-status` | `npx playwright test --grep @claim:managed-plan-status` | PASS, 1 test |

The manifest contract test also confirms every claim has exactly one source
tag. Landing, application, privacy, demo, and README copy were cross-checked;
no unlisted material claim was found.

Setup note: the literal pre-install invocation made before any other QA could
not load the declared local `@playwright/test` development dependency. This was
an installation precondition, not a product assertion failure. `npm ci` from
the committed lockfile succeeded, after which all 15 exact claim commands ran
and passed. No source file changed between attempts.

## End-to-end product evidence — PASS

The real flow was exercised against a fresh local database and again in a new,
isolated live workspace:

1. Created a private workspace and received a correctly shaped owner key.
2. Published `examples/skill-package.json` from the exact candidate Git commit.
3. Submitted forged browser instruction/adapter values and confirmed the signed
   package used the verified Git content instead.
4. Confirmed pilot release was rejected before review with a recovery message.
5. Opened the package with its independent reviewer key, approved it, and
   confirmed reuse of that key returned 401.
6. Entered pilot, issued a scoped install credential, and downloaded the
   assigned package.
7. Confirmed the download contained schema `team-agent-skill/v2`, package
   `secure-commit` v1.0.0, repository `atlas-api`, a 64-character digest, and a
   128-character signature.
8. Recorded an execution receipt and released to all assigned repositories.

The live flow produced no unexpected page errors. Evidence:
[`live-real-flow-audit.json`](qa-artifacts/live-real-flow-audit.json) and
[`local-real-flow-audit.json`](qa-artifacts/local-real-flow-audit.json).
The local audit's two console network messages are the expected 409 and 401
responses from the deliberate pre-review release and consumed-key probes.

Boundary and recovery checks also passed: an 80-character workspace name was
accepted, 81 characters and an empty name returned 400, malformed JSON returned
400, unauthenticated private reads returned 401, an inactive key exposes
recovery controls, and malformed demo storage resets to clean sample data.

## Backend, persistence, and rate limits — PASS

The optimized server was started in a fresh temporary directory with only
`PORT=4195`. It generated its SQLite database and signing identity, served the
app, and logged generated/default configuration without secret values. After a
restart, the workspace key still worked and `/health` returned the same signer
fingerprint.

The locally observed allowance was **40 non-health requests per client IP per
second**. In a 41-request concurrent burst, responses were 40×200 and 1×429;
the 429 included `Retry-After: 1`. A second first-hop
`X-Forwarded-For` client immediately received 200. Fifty concurrent health
requests all returned 200, as documented.

Fresh live behavior matched: 40×200 followed by 1×429 with `Retry-After: 1`,
while a second client received 200. The private-source integration test also
proved repository-bound credentials, cross-workspace/cross-repository denial,
and absence of the credential value from SQLite.

## Deployment identity, privacy, and security — PASS

- Live `/health` returns
  `build_sha: fcb5b047e2d3a1d7c41c3e860d2332c22d932f33`.
- Local and live SHA-256 hashes match for `index.html`, the hashed JS, the
  hashed CSS, and `release-desk.webp`.
- Browser request logging over landing, demo, reset, receipt, legal, review,
  and 404 routes observed only
  `https://team-agent-skills.sociobot.in`; no analytics, CDN fonts/scripts,
  Azure endpoints, or other third-party browser requests occurred.
- Responses include CSP with `frame-ancestors 'none'`, HSTS, `nosniff`,
  strict-origin referrer policy, and restrictive Permissions-Policy.
- Hashed JS/CSS use `public, max-age=31536000, immutable`; the hero image uses
  a seven-day public cache.
- `/`, `/demo`, `/registry`, `/review`, `/privacy`, `/terms`, `/404.html`,
  `robots.txt`, `sitemap.xml`, `/health`, and `/api/trust` returned expected
  statuses. An unknown route returned 404. Every rendered internal link
  resolved to 200.
- Sign-in is not required, so the Entra tenant requirement is not applicable.
  This is not a PWA, library, or CLI, so their additional checks do not apply.

## Accessibility, responsive behavior, and performance — PASS

The supplied `verify-url.sh` passed locally and live: title, `lang=en`, one h1,
main landmark, alt text, labels, and console/page errors were clean. Fresh axe
scans found zero serious/critical violations on `/`, `/demo`, `/privacy`,
`/terms`, `/review`, and `/404.html`.

At 390×844, there was no horizontal overflow, every visible interactive target
was at least 44×44 CSS pixels, and all controls remained on-screen. At 200% root
text size there was still no overflow or clipped action. Keyboard-only
navigation reached the skip link, header links, and demo controls with a 3px
high-contrast focus ring; Space activated Reset demo. With reduced motion,
there were zero active animations/transitions and scroll behavior was `auto`.
Evidence: [`live-keyboard-audit.json`](qa-artifacts/live-keyboard-audit.json),
[`live-demo-mobile-fresh.png`](qa-artifacts/live-demo-mobile-fresh.png), and
[`verify-live/verify.json`](qa-artifacts/verify-live/verify.json).

Fresh mobile Lighthouse scores were performance **99**, accessibility **100**,
best practices **100**, and SEO **100**. FCP was 1.2 s, LCP 1.5 s, TBT 100 ms,
CLS 0, and total transfer 101 KiB. The production build emitted 30.90 KB raw /
9.91 KB gzip JS, 15.31 KB raw / 4.11 KB gzip CSS, and a 54,898-byte hero image.
Evidence: [`lighthouse-mobile-fresh.json`](qa-artifacts/lighthouse-mobile-fresh.json).

## Quality gates

| Command | Result |
| --- | --- |
| `npm ci` | PASS, 50 packages installed from lockfile |
| `npm audit --audit-level=high` | PASS, 0 vulnerabilities |
| `npm test` | PASS, 2 Vitest + 4 Node tests |
| `npm run typecheck` | PASS |
| `npm run build` | PASS, `dist/` produced |
| `npx playwright test` | PASS, 19/19 |
| all 15 claim commands | PASS, 15/15 |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS, 11/11 integration tests |
| `cargo build --release --locked` | PASS |

No Docker-compatible engine is installed in the verifier container, so the
image wrapper was not rebuilt locally. Both exact build stages passed, Docker
contract tests passed, and the live container reports and serves the exact
candidate.

## Defects by severity

- Critical: none.
- High: none.
- Medium: none.
- Low: publish-form textareas retain Chromium's visible, high-contrast native
  1px focus outline rather than the product's custom 3px coral focus ring.
  Keyboard operation and focus visibility are unaffected; this is a visual
  consistency improvement for a later patch.
