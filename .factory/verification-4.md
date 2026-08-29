# Independent product verification 4 — FAIL

Verified on 2026-08-29 against candidate
`81de7172e529b41093aada0fc114d6808cccc380` and
`https://team-agent-skills.sociobot.in`.

## Decision

**FAIL. Do not release this candidate.** The deployment is healthy and matches
the candidate exactly. The mandatory first-read gate passes, all 11 declared
claim tests pass after the locked install, and the product is fast and broadly
accessible. It still does not deliver the brief's Git-backed, repository-safe
ring rollout:

1. The server verifies only that a GitHub commit exists. It never reads the
   skill package from that commit or proves that submitted instructions and
   adapters came from it.
2. `pilot` and `all` have identical install access. Every assigned repository
   can install as soon as the package enters pilot.
3. Install consumers use the workspace owner key. There is no repository- or
   agent-scoped read credential, so distributing an install credential also
   grants registry administration across the workspace.
4. The public rate limit trusts a caller-supplied `X-Forwarded-For` first hop.
   One client bypassed the live 40-request allowance by changing that header.

These are core provenance, rollout, access-boundary, and mandatory backend
security failures. The passing claim suite is narrower than the researched
acceptance contract.

## Mandatory first-read gate — PASS

A cold 1440×900 browser context, with no prior storage, showed:

- What it does: “Release reviewed skills across repositories.”
- Who it is for: “For engineering leads who need one checked instruction set
  for every coding agent.”
- What to do first: “Try it with sample data,” beside “Open three complete
  skill packages and their review records.”
- Three plain facts about browser-local sample data, exact-version receipts,
  and private workspace access.

The one visible sample-data action opened `/demo` in one click. The resulting
screen already contained three packages and three receipts, plus the persistent
“Demo — sample data, nothing is saved” banner, **Reset demo**, and **Start for
real** controls.

Evidence:

- `.factory/verification-artifacts/live-cold-desktop.png`
- `.factory/verification-artifacts/live-mobile.png`
- `.factory/verification-artifacts/live-demo-after-one-click.png`

## Mandatory claims gate — PASS after install

The clean clone initially had no installed Playwright package, so the first
pre-install browser command could not load `@playwright/test`. `npm ci` then
installed the locked dependency set with zero audit findings. Every command in
`.factory/claims.json` was run independently before the rest of product QA:

| Claim | Exact command | Result |
| --- | --- | --- |
| `demo-separation` | `npx playwright test --grep @claim:demo-separation` | PASS — 1 test |
| `execution-receipt` | `npx playwright test --grep @claim:execution-receipt` | PASS — 1 test |
| `demo-local-data` | `npx playwright test --grep @claim:demo-local-data` | PASS — 1 test |
| `review-required` | `npx playwright test --grep @claim:review-required` | PASS — 1 test |
| `package-contents` | `npx playwright test --grep @claim:package-contents` | PASS — 1 test |
| `private-workspace` | `cargo test --locked claim_private_workspace` | PASS — 1 test |
| `secret-reference-format` | `cargo test --locked claim_secret_reference_format` | PASS — 1 test |
| `git-signed-package` | `cargo test --locked claim_git_signed_package` | PASS — 1 test |
| `independent-one-time-review` | `cargo test --locked claim_independent_one_time_review` | PASS — 1 test |
| `governed-execution-receipt` | `cargo test --locked claim_governed_execution_receipt` | PASS — 1 test |
| `managed-plan-status` | `npx playwright test --grep @claim:managed-plan-status` | PASS — 1 test |

The manifest/source-tag contract test also passes: each listed claim has one
matching `@claim:<id>` tag and no extra claim tag was found.

## Release-blocking defects

### Critical — submitted text is signed without being backed by the Git commit

The researched product requires Git-backed skill packages and provenance. The
publish request accepts instructions and adapters directly from the browser.
The source verifier calls only GitHub's commit endpoint and checks that the
response SHA equals the supplied SHA (`backend/src/main.rs:382-429`). It does
not request a tree, blob, package file, path, or content hash. After that check,
the server serializes the form input and signs it (`backend/src/main.rs:462-500`).

The real local UI accepted and signed the verifier's arbitrary “Release safety”
instructions against candidate commit `81de717…`; those words were supplied in
the form, not loaded from that commit. The package received a 64-character
digest, 128-character Ed25519 signature, and `source_verified_at` timestamp.
This proves only that the commit exists and that the server signed submitted
text. It does not prove the package is in Git or corresponds to that commit.

The `git-signed-package` claim test checks commit existence and verifies the
signature over submitted fields, so it does not catch this difference. Calling
the package Git-backed overstates the provenance layer that distinguishes this
product in the brief.

### Critical — rollout and install credentials do not preserve repository boundaries

There is no pilot cohort or repository ring assignment. Both install and
receipt queries accept `ring IN ('pilot','all')` (`backend/src/main.rs:683` and
`:767`). In a fresh local workspace, a reviewed package assigned to
`pilot-repo` and `later-repo` was changed to pilot. Both install URLs returned
HTTP 200 immediately:

```text
create 201; review 200; change to pilot 204
pilot-repo install 200
later-repo install 200
```

Therefore “Pilot” and “All repos” differ only in the stored label. This is not
the approval/ring rollout required by the brief.

All install routes also call `owner_workspace`; the same bearer token can
publish packages, approve ring changes, read every package, and create every
receipt. There is no repository-, adapter-, or agent-scoped install credential.
An agent that fetches its package must receive the workspace owner key, which
also lets it administer or fetch other repositories' records. This conflicts
with the explicit requirement to preserve repository access boundaries.

### High — the mandatory live rate limit is spoofable

With one fixed client identity, 100 concurrent live `GET /api/trust` requests
completed in 389 ms as 40×200 and 60×429; all 60 limit responses included
`Retry-After: 1`. The observed nominal allowance is therefore **40 requests per
client per one-second fixed window**; `/health` is exempt.

The server keys solely on the first caller-provided `X-Forwarded-For` value
(`backend/src/main.rs:798-827`). The public ingress preserved the test header.
From the same process and network client, 100 concurrent requests with 100
different `X-Forwarded-For` values completed in 415 ms as **100×200 and
0×429**. The limit can therefore be bypassed without changing the real client
IP. It does not satisfy the mandatory requirement that one client exceeding
the documented allowance receives 429.

## Other defects and gaps

### Medium — the integration suite has an intermittent keyboard failure

The first full `npx playwright test` run failed 1 of 19 tests because activating
**Skip to content** did not focus `<main>`. An isolated ten-repeat run reproduced
the failure once (9/10 pass). A later full rerun passed 19/19, and the exact
production server passed 50/50 repetitions; the live server passed 10/10.

This is a test/dev-server race rather than a consistently observed production
failure, but it means the advertised integration gate is not deterministic.

### Medium — the demo downloads the obsolete v1 schema

The live demo's download reports `schema: "team-agent-skill/v1"`, while the real
install endpoint returns `team-agent-skill/v2`. The branch is hard-coded at
`frontend/src/main.ts:201`. Other sample fields and signature lengths look
realistic, but the mandatory sandbox does not reproduce the production public
format.

### Medium — the managed subscription is not available

The landing page accurately states `$149 per team each month` and that billing
is inactive. It exposes no broken checkout or payment-provider integration.
That honesty passes its declared claim, but the researched subscription has no
usable purchase path in this release. This remains a documented product gap,
not a hidden failure.

## End-to-end behavior and input boundaries

The optimized release binary was copied to a fresh temporary directory and
started with only `PORT=4194` in an otherwise empty environment. It generated
`data/registry.db` and a signing identity, served the frontend, and returned
healthy build identity `dev`.

The real browser workflow completed on desktop and 390 px:

1. An empty workspace name was blocked with “Please fill out this field.”
2. A private workspace was created and returned an owner key.
3. A two-adapter package was published against the candidate's real GitHub
   commit and returned a separate reviewer key.
4. An invalid reviewer key was rejected; the valid package-only key opened the
   signed package without the owner key.
5. Approval consumed the reviewer key; reuse was rejected.
6. Pilot release, package download, and receipt creation completed.
7. The downloaded real package used schema v2 and contained version `1.0.0`,
   both target adapters, a 64-character digest, and a 128-character signature.

API boundary checks returned the expected statuses:

- workspace name: empty and 81 characters → 400; 80 characters → 201;
- unauthenticated package list → 401;
- 39-character commit, non-GitHub URL, secret-like value, missing adapter,
  duplicate target, empty repository list, and 101-character package name →
  400;
- before-review release/install failed and recovered after independent review;
- 40 concurrent receipt writes produced unique receipt ids in the Rust suite;
- database records, owner-key hash, and signing fingerprint survived restart.

Evidence:

- `.factory/verification-artifacts/local-e2e-desktop.png`
- `.factory/verification-artifacts/local-e2e-mobile.png`

## Live identity, privacy, headers, accessibility, and performance

- `/health` returned exact build SHA
  `81de7172e529b41093aada0fc114d6808cccc380` and HTTP 200.
- Live `index.html`, hashed JS, hashed CSS, hero WebP, social card, favicon, and
  apple-touch icon were byte-for-byte identical to local `dist/`.
- Root, demo, registry, review, privacy, terms, robots, sitemap, and all checked
  assets returned 200. An unknown route returned the designed 404.
- The cold landing and complete demo flow contacted only
  `team-agent-skills.sociobot.in`. Demo storage contained only
  `demo:team-agent-skills:v2`. No analytics, third-party font/script, model,
  payment, license, or identity-provider request occurred.
- Responses include HSTS, `nosniff`, referrer policy, permissions policy, and a
  same-origin CSP with `frame-ancestors 'none'`. Hashed JS/CSS use one-year
  immutable caching; the hero image uses one-week caching.
- Factory `verify-url.sh`: 596 ms, no load errors, title, `lang=en`, one h1,
  main landmark, no missing image alt, and no unnamed button.
- Axe-core 4.11: zero serious/critical findings on `/`, `/demo`, `/registry`,
  `/review`, `/privacy`, `/terms`, and the real 404 at 390 px.
- Mobile has no page overflow. Tested navigation, primary, and footer targets
  are at least 44 px high. Reduced motion yields zero animated/transitional
  elements and `scroll-behavior: auto`.
- Visible keyboard focus uses a 3 px coral outline. Route navigation focuses
  and announces the new h1. The intermittent dev-suite skip-link issue is
  reported above.
- Fresh Lighthouse mobile: performance 100, accessibility 100, best practices
  100, SEO 100; FCP 1.3 s, LCP 1.6 s, TBT 0 ms, CLS 0, total 99 KiB.
- Production output: JS 29.10 KB raw / 9.50 KB gzip; CSS 15.31 KB raw / 4.11
  KB gzip; hero 54.9 KB. All are well within the acceptance budgets.

Evidence:

- `.factory/verification-artifacts/live-health.json`
- `.factory/verification-artifacts/live-root-headers.txt`
- `.factory/verification-artifacts/live-js-headers.txt`
- `.factory/verification-artifacts/verify-url/verify.json`
- `.factory/verification-artifacts/lighthouse-mobile-2.json`

## Clean-checkout gates

| Command | Result |
| --- | --- |
| `npm ci` | PASS — 50 packages |
| `npm audit` | PASS — 0 vulnerabilities |
| all 11 claim commands | PASS independently after install |
| `npm test` | PASS — 2 Vitest + 4 Node contract tests |
| `npm run typecheck` | PASS |
| `npm run build` | PASS — exact `dist/` produced |
| `npx playwright test` | FLAKY — first 18/19, rerun 19/19 |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS — 7/7 integration tests |
| `cargo build --release --locked` | PASS |

No Docker-compatible engine is installed in the verifier. Docker contract
tests pass, the release binary was exercised directly from a fresh directory,
and live identity/build hashes match. This is not a PWA, library, or CLI, so
service-worker update/offline reload and clean-consumer package checks do not
apply. The product uses capability keys rather than user sign-in, so the
conditional Sociobot Entra authority check does not apply. Runtime AI is not
part of the product. The inactive managed plan makes checkout/license checks
not applicable.
