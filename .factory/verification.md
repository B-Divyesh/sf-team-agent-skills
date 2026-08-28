# Independent product verification — FAIL

Verified on 2026-08-28 against candidate
`5610b554b98183494868d2ec05ce4622969e7b7b` and
`https://team-agent-skills.sociobot.in`.

## Decision

**FAIL. Do not release this candidate.** The deployment is live, healthy, and
byte-for-byte aligned with the local frontend build, but the product does not
meet the researched job-to-be-done or its safety contract. The live registry
has no authentication or tenant/repository boundary, contains no actual skill
instructions or Git-backed packages, lets any client change rollout state and
forge receipts, can falsely announce a release-ring change after the server
rejects it, and links its advertised paid plan to a 404.

The mandatory claims tests and first-read gate pass. They are not sufficient to
overcome the release blockers below.

## First-read gate — PASS

Cold desktop load, fresh browser context, no prior storage:

- What it does: “Release reviewed skills across repositories.”
- For whom: “For engineering leads who need one checked instruction set for
  every coding agent.”
- What to click first: “Try it with sample data,” alongside “Open a working
  registry with three reviewed skills.”
- One click opened `/demo`, displayed three skill packets and three receipts,
  and showed “Demo — sample data, nothing is saved” with reset and real-start
  actions.

Evidence: `.factory/qa-first-read.png` and
`.factory/evidence/verify-url/verify.json`.

## Required claims tests — PASS, coverage contract — FAIL

The manifest exists. Each exact command was run independently after `npm ci`:

| Claim | Exact command | Result |
| --- | --- | --- |
| `demo-separation` | `npx playwright test --grep @claim:demo-separation` | PASS, 1 test |
| `execution-receipt` | `npx playwright test --grep @claim:execution-receipt` | PASS, 1 test |
| `demo-local-data` | `npx playwright test --grep @claim:demo-local-data` | PASS, 1 test |

The landing page and README nevertheless contain unlisted or materially
overstated claims, which is independently release-blocking under the claims
contract:

- “Packets move only after review” and “See the approval trail” are false.
  There is no reviewer, approval record, policy, or permission check; a ring is
  changed by one unauthenticated PATCH.
- “Keep instructions, adapters, and secret references together” is false for
  the shipped UI. Publishing asks only for a name and summary. Version, owner,
  target, and ring are fixed; there is no instruction body or adapter artifact.
- “Private registries, approval rings, and audit history” is false. The live
  registry and all mutation endpoints are public and global.
- “Skills name secret references. They never hold secret text” is not enforced.
  The API accepts arbitrary strings in `secrets` and has no value/format check.
- “Every run records a version receipt” is tested only for a manual demo form.
  No agent run integration exists, and the real receipt row does not persist a
  version; reads join to the skill's current version.
- The quantitative `$149 per team/month` paid-plan claim has no claim test, and
  the checkout target currently returns 404.
- “This v1 does not execute code or host models” is true by source inspection
  but is also absent from `.factory/claims.json`.

## Release-blocking defects

### Critical — no access control or repository boundary

There is no sign-in, token validation, tenant identifier, user/repository ACL,
or authorization middleware anywhere in the repository. Live unauthenticated
requests to `/api/skills` and `/api/receipts` return 200. The router exposes
unauthenticated create-skill, change-ring, and create-receipt routes backed by
one SQLite database. This violates the brief's governed private registry and
repository-boundary requirements. Sociobot Entra cannot be verified because
authentication is absent, not because another provider is used.

### Critical — smallest useful product is not implemented

The service stores only skill metadata. It has no skill instruction content,
Git source/commit/signature, package download, target-agent adapter content,
version publishing/update path, reviewer/approval record, repository
assignment, or distribution mechanism. The “Publish a skill” UI is two native
prompts and hard-codes version `0.1.0`, target `Codex`, owner `You`, and ring
`draft`. Clicking a ring only changes a status string. This cannot perform the
brief's real job of safely distributing a reviewed, exact skill version across
repositories and agent vendors.

### High — rejected rollout is announced as successful

After using one client allowance, the next ring PATCH returned `429` with
`Retry-After: 1`. The browser still displayed “QA release check is now in
draft,” rendered “Ring: draft,” and did not offer recovery, while a fresh API
read showed the persisted ring remained `pilot`. The ring handler awaits
`fetch()` but never checks `response.ok`. A network exception is handled, but
all HTTP failures are false successes. This is unsafe for the product's core
rollout action.

### High — paid path is unavailable and license feedback is hidden

- `GET https://api.sociobot.in/api/v1/products/team-agent-skills/checkout`
  returns HTTP 404 with `{"error":"enabled factory product","status":404}`.
- Loading `/?license=qa-return-token` stores and strips the token but sends no
  verification request, contrary to the first-unlock verification contract.
- Manual restore does call only the Sociobot verify endpoint and caches its
  invalid result, but the landing page renders no status region. The user sees
  no “active” or “not active” result.
- No feature is gated or unlocked from a valid cached verdict.

### High — available quality gates fail

- `cargo fmt --all -- --check` fails with formatting diffs in
  `backend/src/main.rs` and `tests/api.rs`.
- `cargo clippy --all-targets --locked -- -D warnings` fails on a redundant
  borrow at `backend/src/main.rs:55`.
- The Dockerfile pins `rust:1.88-slim`; the supplied backend contract expressly
  requires the moving `rust:1-slim`/`rust:1-alpine` stable tag. The repository's
  Docker test enforces the conflicting pin rather than the acceptance contract.

### Medium — demo exit and state messaging are incorrect

Leaving the demo retains `localStorage['demo:team-agent-skills:v1']` without
discarding it or offering to keep it. The previous demo notice also leaks into
the real `/registry`; after reset and “Start for real,” the real workspace said
“Demo reset. The sample packets are back” while its API data was empty.

### Medium — accessibility target size and widget state gaps

Axe found no serious/critical violations, keyboard focus is visible, and the
skip link works. Manual geometry found multiple controls below the mandatory
44px target: navigation links 42px high, ring buttons 38px, inputs/select 42px,
demo reset 38px, and footer links 22px. The mobile ring row clips the final
option into a horizontal scroller. Selected skill and active ring buttons use
CSS classes without `aria-pressed`, `aria-current`, or equivalent programmatic
state.

### Medium — dependency and caching hygiene

- `npm audit` reports five development-tool vulnerabilities: three moderate,
  one high, and one critical (Vitest arbitrary file read/execution when its UI
  server is exposed). `npm audit --omit=dev` reports zero production findings.
- Hashed JS/CSS and the hero image have no `Cache-Control` header. Lighthouse's
  long-cache audit scored 0.5 for three resources (about 83 KiB).
- Live HTTPS responses have CSP, `X-Content-Type-Options`, and
  `Referrer-Policy`, but no HSTS header was observed.

### Low — route metadata and 404 structure

SPA routes update the title, but every route keeps the home canonical URL and
home social metadata. The server 404 has a title, language, main, and return
link, but not the site's standard header/footer/version skeleton.

## Functional and backend evidence

- Demo desktop and 390px mobile: publish → select → ring change → reject empty
  required repository → recover → record receipt → reset all worked. Reset
  restored exactly three skills and three receipts under only the demo key.
- Real release binary: publish → ring change → receipt creation worked through
  the browser with no console errors.
- API boundary cases: empty fields, 101-character name, invalid ring, blank or
  161-character repository returned 400; a 100-character name succeeded;
  duplicates returned 409; missing skill/ring targets returned 404.
- Persistence: skills, ring state, and receipts survived a binary restart with
  the same SQLite path.
- Write concurrency: 40 simultaneous receipt writes succeeded with 40 unique
  IDs; request 41 returned 429 with `Retry-After: 1`.
- Runtime contract: the release binary started in a fresh directory with only
  `PORT` set, created `data/registry.db`, served `/`, and returned
  `{"status":"ok","build_sha":"dev"}`.

## Deployment, limits, privacy, and headers

- `/health` returns the full candidate SHA
  `5610b554b98183494868d2ec05ce4622969e7b7b`.
- Local and live `index.html`, JS, and CSS SHA-256 hashes match exactly.
- Main live API allowance observed: 120 concurrent requests in 1.78s produced
  40×200 and 80×429; 429 responses had `Retry-After: 1`.
- Sociobot license-verify allowance observed: 120 concurrent invalid-token
  checks produced 30×200 and 90×429; 429 responses had `Retry-After: 4`.
- Health remained 200 after the burst and is intentionally exempt.
- Fresh landing, demo workflow, and registry reads contacted only
  `team-agent-skills.sociobot.in`. Manual restore contacted only
  `api.sociobot.in`, after explicit action. No analytics, third-party scripts,
  fonts, or Azure endpoints were observed.
- Root, demo, registry, privacy, terms, robots, sitemap, assets, APIs, and health
  returned their expected status. The paid checkout link was the dead-link
  exception. An unknown route correctly returned 404.
- Browser document responses include a same-origin CSP, `nosniff`, and
  `strict-origin-when-cross-origin`.

## Accessibility and performance evidence

- Factory `verify-url.sh`: PASS; 662ms load, zero console/page errors, title,
  `lang=en`, one h1, main landmark, zero missing alt text, zero unlabeled
  buttons. Evidence is in `.factory/evidence/verify-url/`.
- Playwright axe-core: zero serious/critical findings on desktop landing and
  390px landing/demo/privacy/terms.
- Reduced-motion context computed `scroll-behavior: auto`, zero transition
  duration, and no animation.
- No page-level horizontal overflow at 1440px or 390px on the landing page.
- Mobile Lighthouse: performance 100, accessibility 100, best practices 100,
  SEO 100; FCP 1.1s, LCP 1.4s, TBT 60ms, CLS 0, 83 KiB transfer.
- Production output: JS 16.23 KiB raw / 5.95 KiB gzip; CSS 12.23 KiB raw /
  3.53 KiB gzip; hero WebP 54.9 KiB; social image 1200×630.

## Clean-checkout commands

| Command | Result |
| --- | --- |
| `npm ci` | PASS; lockfile installed; audit warning noted above |
| all three exact claim commands | PASS |
| `npm test` | PASS; 2 Vitest + 3 Node contract tests |
| `npm run typecheck` | PASS |
| `npm run build` | PASS; `dist/` produced |
| `npx playwright test` | PASS; 6 tests |
| `cargo test --locked` | PASS; 1 integration test, 0 unit tests |
| `cargo build --release --locked` | PASS |
| `cargo fmt --all -- --check` | **FAIL** |
| `cargo clippy --all-targets --locked -- -D warnings` | **FAIL** |
| `npm audit --omit=dev` | PASS; 0 production vulnerabilities |
| `npm audit` | **FAIL advisory state**; 5 development findings |

No Docker/Podman/Buildah engine is installed in this verifier, so the container
build could not be repeated locally. This does not create deployment ambiguity:
the live health identity and exact frontend hashes prove the currently served
deployment is the requested candidate. This product is not a PWA, library, or
CLI, so service-worker and clean-consumer package checks do not apply.
