# Independent product verification — FAIL

Verified on 2026-08-28 against candidate
`3a7c56c22c63c59d0d62bf49f862b152e4879fad` and
`https://team-agent-skills.sociobot.in`.

## Decision

**FAIL. Do not release this candidate.** The live deployment is healthy and is
the exact candidate, all executable quality gates pass, the first-read/demo
gate passes, and the main routes are fast and accessible. The product still
does not provide the signed, Git-backed, independently reviewed provenance
layer in the researched brief. Its reviewer credential cannot be used by a
reviewer independently, its documented "one-time" reviewer key is reusable,
and receipts can be created for unreleased packages and unassigned agents.
The claims manifest also violates the required one-tag-per-claim contract, and
the real 404 has a serious axe contrast failure.

## Mandatory first-read gate — PASS

Cold desktop and 390 px mobile loads used fresh browser contexts.

- What it does: "Release reviewed skills across repositories."
- Who it is for: "For engineering leads who need one checked instruction set
  for every coding agent."
- What to click first: "Try it with sample data," followed by a plain
  explanation that it opens three complete packages and review records.
- The action is visible in the initial 390×844 viewport at y=400 and opens
  `/demo` in one click. The demo immediately contains three realistic packages
  and three receipts, plus the required persistent demo/reset/real-start bar.

Evidence:

- `.factory/qa-artifacts/live-first-read-desktop.png`
- `.factory/qa-artifacts/live-first-read-mobile.png`
- `.factory/qa-artifacts/live-demo-mobile.png`

## Mandatory claims gate

`.factory/claims.json` exists. After the documented `npm ci` install, every
listed command was run independently and passed:

| Claim | Exact command | Result |
| --- | --- | --- |
| `demo-separation` | `npx playwright test --grep @claim:demo-separation` | PASS — 1 test |
| `execution-receipt` | `npx playwright test --grep @claim:execution-receipt` | PASS — 1 test |
| `demo-local-data` | `npx playwright test --grep @claim:demo-local-data` | PASS — 1 test |
| `review-required` | `npx playwright test --grep @claim:review-required` | PASS — 1 test |
| `package-contents` | `npx playwright test --grep @claim:package-contents` | PASS — 1 test |
| `private-workspace` | `cargo test --locked release_binary_enforces_private_reviewed_repository_packages` | PASS — 1 test |
| `secret-reference-format` | `cargo test --locked release_binary_enforces_private_reviewed_repository_packages` | PASS — 1 test |

The manifest nevertheless fails the claims contract itself. The two Rust
claims have no tests tagged `@claim:private-workspace` or
`@claim:secret-reference-format`; both point to the same broadly named test.
The only claim tags in the repository are the five browser tags plus an
unlisted `@claim:demo-seed`. This is not the required exactly-one tagged test
per claim.

There are also unlisted or false claim-like statements in the README:

- The README calls the reviewer key "separate" and "one-time." A fresh client
  using only that key received 401, while the same reviewer key approved two
  different packages when paired with the owner key.
- Runtime, hash-storage, immutable-version, non-root-container, and build-SHA
  promises are stated in the README but have no claims entries.
- Two records with the same visible name and version but different content
  were both accepted, so "immutable version" does not identify one unique
  package without the hidden record id/digest.

## Release-blocking defects

### Critical — packages are neither Git-backed nor signed

The researched brief's distinct product is a signed, versioned compatibility
layer with Git-backed packages. The API only checks that `git_url` starts with
`https://` and that `git_commit` is 40 hexadecimal characters. It accepted a
non-verified example URL and an invented all-`a` commit with HTTP 201. There is
no repository fetch, commit existence check, signature, signer identity, or
signature verification in source or in the downloaded package.

The server's `package_digest` is not a digest of the complete package. It hashes
only version, instructions, adapters, Git URL, and commit. Two packages with
different ids, names, target agents, repository assignments, secret references,
owners, and summaries produced the identical digest
`acec56bc4866c5e20456e905346a76389b4c974cb8c66706d9c3457dd8b30b16`.
Two different records with the same displayed name/version were also accepted
with HTTP 201. A reviewer or execution receipt therefore cannot use the
advertised provenance fields to prove one complete immutable package.

### Critical — approval and execution receipts do not prove governed runs

The service tells the owner to give the reviewer key to the approver, but every
approval request first requires the owner workspace key. A separate browser
with only the reviewer key received `401 That workspace key is not active` and
could not see or approve the package. Sharing the owner key would give the
reviewer all owner read/write/release powers. The same reviewer key then
successfully approved a second package, contradicting the README's "one-time"
claim.

After approval but while the package was still in the non-installable `review`
ring, `POST /api/receipts` accepted agent `Unassigned Agent` even though the
package targeted only `Codex`; it returned 201. At that moment the install
endpoint correctly returned 404. A client can therefore create an execution
receipt for an agent/package combination that could not have been distributed
through the governed install endpoint. This breaks the brief's core attribution
and safe-rollout job.

### High — tenant identifiers are globally shared

`skills.id` is the database-wide primary key instead of a workspace-scoped
unique key. Workspace A published `shared-skill-id`; an isolated workspace B
then received 409 when publishing the same id, even though B's list was empty.
One tenant can reserve identifiers and interfere with another tenant's writes.
Read/install access remained isolated, but write availability and existence
information do not.

### High — claim coverage is structurally incomplete

The missing exact Rust claim tags and unlisted/false README claims described
above violate the explicit acceptance rule that every visitor-facing claim has
exactly one tagged sandbox test. Passing one broad integration test under two
manifest entries does not satisfy that rule.

### High — invalid persisted state has no recovery path

- Setting the documented demo key to malformed JSON and opening `/demo`
  produced only the skip link, zero h1s, no reset button, and an uncaught
  `SyntaxError`. The required reset/recovery path becomes inaccessible.
- A stale but correctly shaped workspace key shows "That workspace key is not
  active" inside the registry, but removes the create/restore forms and offers
  no clear, sign-out, forget, or change-workspace action. The user cannot
  recover without clearing browser storage manually.

### High — the real 404 fails the accessibility baseline

Axe-core 4.11.0 reports a serious `color-contrast` violation on the 404 header
wordmark at both 1440 px and 390 px. Its computed color is browser-default blue
`rgb(0, 0, 238)` over `rgb(16, 33, 44)`, only **1.75:1**. The same home link is
159×24 px, below the required 44 px touch height. The 404 otherwise has a title,
language, one h1, main/header/nav/footer, and a way home.

Evidence: `.factory/qa-artifacts/live-404-mobile.png`.

### Medium — monetization from the researched brief is absent

The researched contract specifies a `$149/team/month` governed private registry.
The candidate has no paid tier, price, checkout, subscription state, or
documented product deviation under known gaps. This avoids the previous dead
checkout, but it does not implement the stated monetization contract.

### Medium — the hosted authoring UI cannot create distinct vendor adapters

The API supports an adapter object and the demo shows vendor-specific adapter
text, but the real publish form has one "Adapter instructions" field and copies
that same value to every selected target agent. Engineering leads cannot author
different Codex/Claude/Cursor adapters through the product UI.

## End-to-end and backend evidence

The optimized release binary was started in a fresh directory with only
`PORT=4191`. It created `data/registry.db`, served the built frontend, and
reported `{"status":"ok","build_sha":"dev"}`.

The real UI workflow completed on desktop and 390 px:

1. Empty required workspace name was blocked with the browser's plain validity
   message.
2. Create private workspace → publish a complete package.
3. Pilot before review returned 409 and the UI retained draft.
4. Download before release returned 404 with a useful message.
5. Invalid reviewer key returned 403; the valid key recovered.
6. Approval → pilot → complete JSON package download → receipt all succeeded.
7. Package output contained the exact version, instructions, adapters, commit,
   repository, and 64-character digest.

The data survived a process stop/restart using the same default SQLite path.
Forty concurrent receipt writes produced 40 unique ids; five additional writes
in the same one-second window returned 429. A separate 100-request smoke
completed in 135 ms with 40×401 and 60×429; health remained 200.

Boundary checks also confirmed empty/81-character workspace names, invalid
secret references, secret-like text, a 39-character commit, non-HTTPS Git URL,
and non-object adapters return 400; a 100-character name succeeds and 101
characters fails. Unauthenticated reads return 401, wrong-repository receipts
return 403, pre-review pilot returns 409, and cross-workspace install returns
404.

Evidence screenshots:

- `.factory/qa-artifacts/local-real-flow-desktop.png`
- `.factory/qa-artifacts/local-real-flow-mobile.png`

## Deployment identity, rate limits, privacy, and headers

- `/health` returns the exact candidate SHA
  `3a7c56c22c63c59d0d62bf49f862b152e4879fad`.
- Live `index.html`, hashed JS, hashed CSS, and hero WebP SHA-256 hashes match
  the local `dist/` files exactly.
- A 60-request live burst to `/api/skills` produced 40×401 then 20×429. Every
  429 included `Retry-After: 1`; health stayed 200. Each local API route was
  independently burst-tested and showed the same allowance: 40 responses then
  429 with `Retry-After: 1`. Health is intentionally exempt.
- The cold landing and complete demo reset/receipt flow contacted only
  `team-agent-skills.sociobot.in`. Demo storage contained only
  `demo:team-agent-skills:v2`; the current demo remained usable offline.
- No analytics, third-party scripts/fonts, Azure model endpoint, license, or
  payment-provider request was observed.
- Responses include a same-origin CSP, HSTS, `nosniff`, referrer policy, and
  permissions policy. Hashed JS/CSS use one-year immutable caching; the hero
  image uses a one-week cache.
- All ordinary links crawl to 200. Root, demo, registry, privacy, terms,
  robots, sitemap, assets, APIs, health, and a real 404 return the expected
  status.

## Accessibility, keyboard, responsive layout, and performance

- Factory `verify-url.sh`: PASS in 583 ms with zero load errors, a title,
  `lang=en`, one h1, main, no missing alt, and no unlabeled buttons. Evidence:
  `.factory/qa-artifacts/verify-url/verify.json`.
- Axe serious/critical: zero on landing, demo, registry, privacy, and terms at
  desktop and 390 px; **one serious failure on the 404** as detailed above.
- The full demo tab order reached all 22 visible focusables. Every focused
  control had a designed 3 px outline. Skip-to-main, Space on reset, arrow-key
  select operation, client-route focus/announcement, and back navigation passed.
- Reduced motion yielded zero animated/transitional elements and
  `scroll-behavior: auto` on every tested main route.
- No horizontal page overflow occurred at 390 px; all visible main-route
  controls were at least 44 px high and stayed within the viewport.
- Fresh Lighthouse mobile: performance 100, accessibility 100, best practices
  100, SEO 100; FCP 1.2 s, LCP 1.4 s, TBT 0 ms, CLS 0, total 93 KiB.
- Production output: JS 23.82 KB raw / 8.12 KB gzip; CSS 14.50 KB raw /
  3.95 KB gzip; hero 54.9 KB; social image 43.1 KB at 1200×630.

Lighthouse JSON: `.factory/qa-artifacts/lighthouse-mobile.json`.

## Clean-checkout quality gates

| Command | Result |
| --- | --- |
| `npm ci` | PASS — 50 packages, 0 vulnerabilities |
| all seven exact claim commands | PASS after install |
| `npm audit` | PASS — 0 vulnerabilities |
| `npm test` | PASS — 2 Vitest + 3 Node contract tests |
| `npm run typecheck` | PASS |
| `npm run build` | PASS — exact `dist/` produced |
| `npx playwright test` | PASS — 12/12 |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS — API integration |
| `cargo build --release --locked` | PASS |

No Docker, Podman, or Buildah engine exists in this verifier, so a local image
build could not be repeated. The Docker contract tests pass, the release binary
was exercised directly, and live health plus exact frontend hashes remove
deployment-identity ambiguity. This is not a PWA, library, or CLI, so service
worker/offline-reload and clean-consumer package checks do not apply. The
product uses capability keys rather than a sign-in flow, so the conditional
Sociobot Entra check does not apply.
