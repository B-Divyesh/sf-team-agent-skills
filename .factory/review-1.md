# Adversarial first-read review 1 — Team Skills Registry

## Verdict

**FAIL.** The cold first screen and one-click demo are clear, and all 15 listed
claim commands pass. The product still has blocking historical regressions,
unlisted claims, incomplete 404 metadata, and unresolved copy and workflow
issues. A PASS requires zero findings.

Review date: 2026-08-29 UTC. Live target:
`https://team-agent-skills.sociobot.in`. Live `/health` reported build
`fcb5b047e2d3a1d7c41c3e860d2332c22d932f33`; product code is unchanged between
that build and repository HEAD `7f6bc83645f3c5d3543e4d8975d1fcf5145c0cc6`.

## Findings

### Blocking

#### F-1-1 — The known textarea focus defect is still present

- Exact location: `/demo` → **Publish a version** → **Summary** and the other
  publish textareas.
- Evidence: the focused textarea computes to the browser default
  `outline: auto 1px`; `styles.css` applies the designed 3 px focus treatment to
  `button`, `a`, `input`, `select`, and `.table-wrap`, but omits `textarea`.
- Why this fails: the current handoff explicitly records this as a known gap.
  The accessibility contract requires a designed, contrast-checked focus ring,
  and the history rule makes an unfixed prior gap blocking again.
- Fix: include `textarea:focus-visible` in the 3 px focus selector. Add a
  Playwright assertion for every publish-form textarea at desktop and 390 px.

#### F-1-2 — The landing page makes an unlisted three-package demo claim

- Exact quote: “Open three complete skill packages and their review records.”
- Location: landing hero, beside the primary action.
- Why this fails: `.factory/claims.json` has no claim for the count, package
  completeness, or presence of review records. `demo-separation` checks only
  one seeded package after reset. Earlier verification handoffs blocked the
  product for unlisted landing/README claims; this is the same defect class.
- Fix: add a `demo-sample-content` claim and one tagged test that asserts exactly
  three complete packages and their review records, or remove the count and
  completeness claim.

#### F-1-3 — MIT availability is an unlisted claim

- Exact quotes: “The self-hosted registry remains available under MIT.”
  (landing) and “The self-hosted MIT build remains available.” (README).
- Why this fails: `managed-plan-status` checks price, the inactive-billing
  notice, and absence of checkout only. It does not test build availability or
  licensing. The earlier unlisted-claim blocker has recurred.
- Fix: add a claim test that verifies `LICENSE` is MIT and the documented
  self-hosted build completes, or reduce the copy to a tested statement.

#### F-1-4 — The future billing integration sentence is unlisted and speculative

- Exact quote: “Deployment owners can connect the Sociobot billing API when the
  managed plan opens.”
- Location: landing managed-plan section.
- Why this fails: there is no active billing path and no test can confirm this
  future capability. It tells visitors something they cannot use now.
- Fix: delete the sentence. The adjacent sentence already says billing is not
  active.

#### F-1-5 — Demo teardown is tested but absent from the claims registry

- Exact quote: “Leaving deletes it.”
- Location: README, **Try the sandbox**.
- Why this fails: an untagged Playwright test verifies deletion, but no
  `.factory/claims.json` entry names the claim. The claims contract requires a
  listed entry and exactly one tagged test.
- Fix: add `demo-teardown` to `claims.json` and tag the existing teardown test.

#### F-1-6 — The trust-endpoint claim is unlisted

- Exact quote: “`/api/trust` exposes the signer key and fingerprint for
  consumer pinning.”
- Location: README, **Release rules**.
- Why this fails: no claim entry asserts the endpoint response. The untagged
  health test checks a health fingerprint, not `/api/trust` or its signer key.
- Fix: add a `trust-endpoint` claim and assert the response key, fingerprint,
  and their cryptographic relationship.

#### F-1-7 — The `PORT` configuration claim is unlisted

- Exact quote: “`PORT` changes the listener.”
- Location: README, **Run locally**.
- Why this fails: this is an operational statement a deployer relies on, but it
  has no claims entry.
- Fix: add a tagged test that starts the release binary on a selected free port
  and reaches `/health` there.

#### F-1-8 — The database-path configuration claim is unlisted

- Exact quote: “`DATABASE_PATH` changes the SQLite location.”
- Location: README, **Run locally**.
- Why this fails: this is an operational storage claim without a claims entry.
- Fix: add a tagged test that supplies a temporary explicit path and confirms
  the database and signing state are written there.

#### F-1-9 — The ingress and socket-peer behavior is only partly listed

- Exact quote: “Factory ingress overwrites `X-Forwarded-For`, and the service
  uses its first hop; local runs fall back to the socket peer.”
- Location: README, **Release rules**.
- Why this fails: `trusted-client-rate-limit` checks first-hop partitioning and
  the 40-request limit. It does not prove that deployment ingress overwrites
  the header or that direct local runs fall back to the socket peer.
- Fix: split the sentence. Keep the tested first-hop behavior under the existing
  claim; add deployment-header and direct-peer tests for the other two claims,
  or remove them from the README.

#### F-1-10 — The README’s blanket test-coverage assurance is false

- Exact quote: “Every statement above has an exact test in
  `.factory/claims.json`.”
- Location: README, after **Release rules**.
- Why this fails: F-1-5, F-1-6, and F-1-9 identify statements above it that do
  not have exact listed tests. Earlier handoffs already treated incomplete
  claim coverage as a release blocker.
- Fix: remove the blanket assurance, or list and tag every claim before
  restoring it.

#### F-1-11 — The deployment persistence instruction is unlisted

- Exact quote: “Build the root Dockerfile and mount `/data` for the database
  and signing identity.”
- Location: README, **Deploy**.
- Why this fails: persistence across restart is tested only by an untagged Rust
  test and is absent from `.factory/claims.json`.
- Fix: add a `persistent-data-and-signing-key` claim with a tagged container or
  release-binary restart test.

#### F-1-12 — The static 404 lacks metadata previously claimed as fixed

- Exact location: any unknown live URL, such as
  `/definitely-missing-review-route`, and `frontend/public/404.html`.
- Evidence: the route correctly returns 404 and has a designed page, title,
  one h1, header, footer, Privacy, and Terms. It has no meta description,
  canonical, Open Graph fields, Twitter card fields, theme color, favicon, or
  apple-touch icon. A prior repair handoff states that “Every route updates
  title, description, canonical, Open Graph, and Twitter metadata.”
- Why this fails: the live route contradicts that historical closure and the
  current site-structure contract.
- Fix: give `404.html` the complete route metadata and product icons. Add a
  direct-response test against an unknown URL, not only `/404.html` in Vite.

#### F-1-13 — Repo-native adapter delivery remains absent

- Exact locations: README **API workflow**, the real registry download action,
  and the initial handoff’s known gap: “vendor-specific adapter file generation
  [is] intentionally not automated.”
- Why this fails: the product downloads a custom JSON envelope. It does not
  generate or install `AGENTS.md`, `CLAUDE.md`, or another selected agent’s
  native instruction file, and it provides no repo-ready command or PR. A
  normal visitor reading “release them … across repositories” expects a usable
  handoff to those repositories, not an integration they must invent.
- Fix: for each assigned repository and agent, export a verified repo-native
  file plus a copyable install command, or open a reviewable Git PR. Keep the
  signed JSON as provenance and add claim tests that inspect the produced file.

#### F-1-14 — Lost workspace keys still have no recovery path

- Exact locations: `/registry` says “Save its owner key in a password manager”;
  an earlier handoff records account-based recovery as future work.
- Why this fails: losing the only capability key permanently strands a team’s
  packages, approvals, and receipts. This unresolved historical operational gap
  is material for a team registry.
- Fix: provide an explicit export/rotation/recovery flow. If account recovery is
  intentionally out of scope, support a separately generated recovery key and
  test rotation without exposing stored secrets.

### High

#### F-1-15 — Route focus can move to an off-screen heading

- Exact location: scroll to the landing footer, activate **Privacy**, then use
  Back.
- Evidence at 390 px: after the Privacy navigation, `scrollY` was 366 while the
  focused h1 top was `-172 px`. Back restored the landing scroll position, but
  focus was moved to the landing h1 more than 3,000 px above the viewport.
- Why this fails: visual position and keyboard focus disagree. A keyboard or
  screen-magnifier user can be focused on content they cannot see.
- Fix: distinguish new navigation from history traversal. New routes should
  focus and reveal the h1; Back/Forward should restore the prior scroll and
  previously focused element. Add a footer-navigation and Back regression.

#### F-1-16 — Social image metadata is incomplete

- Exact location: all SPA routes in `frontend/index.html`.
- Evidence: `og:image` is the relative value `/social-card.webp`, and there is
  no `twitter:image`. The underlying image is correctly 1200×630.
- Why this fails: social crawlers expect an absolute image URL and the Twitter
  card does not name an image.
- Fix: use
  `https://team-agent-skills.sociobot.in/social-card.webp` for both `og:image`
  and `twitter:image`; test both on every route.

### Copy and terminology

#### F-1-17 — “Safely” is an unmeasured marketing claim

- Exact quote: “Publish reviewed agent skills and release them safely across
  repositories.”
- Location: README opening sentence.
- Why this fails: “safely” does not name the enforced controls or a tested
  result.
- Rewrite: “Publish reviewed agent skill packages to assigned repositories.”

#### F-1-18 — The package-contents sentence uses unexplained internal terms

- Exact quote: “It contains instructions, agent-specific adapters, repository
  assignments, pilot membership, and secret reference names.”
- Location: README introduction.
- Why this fails: “adapter,” “pilot membership,” and “secret reference” are not
  explained before use.
- Rewrite: “It includes shared instructions, instructions for each agent,
  assigned repositories, pilot repositories, and credential reference names.”

#### F-1-19 — “Git blob identifier” is unexplained jargon

- Exact quote: “The service signs the complete package and its Git blob
  identifier before review.”
- Location: README introduction.
- Why this fails: a first-time engineering lead should not need Git object-model
  vocabulary to understand the guarantee.
- Rewrite: “Before review, the service signs the package and Git’s identifier
  for the exact source file.”

#### F-1-20 — “Source verifier” is an unexplained component name

- Exact quote: “The source verifier accepts GitHub repository URLs.”
- Location: README, **Run locally**.
- Why this fails: it names an internal component instead of the user-visible
  result.
- Rewrite: “The service loads skill packages from GitHub repository URLs.”

#### F-1-21 — One README sentence exceeds the 22-word hard cap

- Exact quote (23 words): “The server never stores or returns the credential
  value, and the reference cannot be used by another workspace or to read
  another repository.”
- Location: README, **Run locally**.
- Why this fails: it combines storage and two authorization boundaries.
- Rewrite: “The server never stores or returns the credential value. Another
  workspace cannot use its reference. The reference cannot read another
  repository.”

#### F-1-22 — “Compatible local verifier fixture” is unexplained jargon

- Exact quote: “Set `GIT_VERIFY_API_BASE` only when running a compatible local
  verifier fixture.”
- Location: README, **Run locally**.
- Why this fails: the reader is not told what compatibility means.
- Rewrite: “Set `GIT_VERIFY_API_BASE` only for a local test server that returns
  the GitHub commit and file responses used by this service.”

#### F-1-23 — “Consumer pinning” is unexplained jargon

- Exact quote: “`/api/trust` exposes the signer key and fingerprint for
  consumer pinning.”
- Location: README, **Release rules**.
- Why this fails: it does not tell the reader what to do with those values.
- Rewrite: “`/api/trust` returns the signer key and fingerprint so installers
  can verify that this service signed a package.”

#### F-1-24 — The ingress sentence is dense infrastructure jargon

- Exact quote: “Factory ingress overwrites `X-Forwarded-For`, and the service
  uses its first hop; local runs fall back to the socket peer.”
- Location: README, **Release rules**.
- Why this fails: “ingress,” “first hop,” and “socket peer” obscure which client
  receives the limit.
- Rewrite: “In production, the trusted proxy supplies the client IP used for
  rate limits. Local runs use the direct connection’s IP.”

#### F-1-25 — The private-source instruction stacks undefined modifiers

- Exact quote: “Before private publishes, the owner binds the
  deployment-managed, repository-pinned reference with
  `POST /api/git-credentials` …”
- Location: README, **API workflow**.
- Why this fails: “private publishes,” “deployment-managed,” and
  “repository-pinned” must be decoded before the action is clear.
- Rewrite: “Before publishing from a private repository, bind its configured
  credential name to that repository with `POST /api/git-credentials` …”

#### F-1-26 — “Adapter credential” conflicts with “install credential”

- Exact quote: “The owner … issues an adapter credential with
  `POST /api/skills/:id/install-credentials`.”
- Location: README, **API workflow**.
- Why this fails: elsewhere the same object is consistently called an “install
  credential.”
- Rewrite: “The owner … issues an install credential with
  `POST /api/skills/:id/install-credentials`.”

#### F-1-27 — The private key has three names

- Exact locations: landing “private access key”; `/registry` “private workspace
  key”; README “owner key.”
- Why this fails: a first-time visitor cannot tell whether these are one
  credential or three different credentials.
- Fix: choose one term, preferably **workspace owner key**, and use it on the
  landing page, registry, README, privacy page, errors, and claims.

#### F-1-28 — Release-ring buttons do not name their result

- Exact labels: “Draft,” “Review,” “Pilot,” and “All repos” in `/demo` and the
  real registry.
- Why this fails: these noun labels do not say whether they display a filter or
  change a release. “All repos” also suggests installation rather than
  availability.
- Rewrite: “Move to draft,” “Send for review,” “Release to pilot,” and “Release
  to all assigned repositories.”

#### F-1-29 — The hero figcaption is provenance copy, not user guidance

- Exact quote: “Original generated artwork showing the release workflow.”
- Location: landing hero figure.
- Why this fails: it tells the visitor how an illustration was made, not how to
  use or evaluate the registry.
- Fix: remove the visible caption. Keep provenance in `.factory/design.md`; keep
  the useful image alt text.

## Cold first read

No scrolling was used before recording this result.

| Viewport | What it does | For whom | First click | Result |
| --- | --- | --- | --- | --- |
| 390×844 | Controls reviewed coding-agent instruction releases across repositories. | Engineering leads managing instructions for coding agents. | **Try it with sample data**. | Pass |
| 1440×900 | Same answer; the product-specific artwork is visible beside the copy. | Same answer. | **Try it with sample data**. | Pass |

The exact first-screen text that supplied the answer was “Release reviewed
skills across repositories,” “For engineering leads who need one checked
instruction set for every coding agent,” and “Try it with sample data.” The
adjacent line says the action opens three sample packages and review records.

## Copy audit

Word count uses whitespace-delimited words; code tokens and hyphenated terms
count as one. Headings and actions are included so they can be checked out of
context. There are no attached-skill banned words and no landing sentence over
22 words. Flags refer to findings above.

### Landing page

| Sentence, heading, or action | Words | Result |
| --- | ---: | --- |
| Team skill releases | 3 | Pass |
| Release reviewed skills across repositories | 5 | Pass |
| For engineering leads who need one checked instruction set for every coding agent. | 13 | F-1-27 |
| Try it with sample data | 6 | Pass |
| Open three complete skill packages and their review records. | 9 | F-1-2 |
| Sample data stays in this browser. | 6 | Pass; listed claim |
| Receipts preserve the exact package version. | 7 | Pass; listed claim |
| Real workspaces use a private access key. | 7 | F-1-27 |
| A paper-cut release desk routes a skill packet through an approval stamp into repository drawers. | 15 | Pass as image alt |
| Original generated artwork showing the release workflow. | 7 | F-1-29 |
| Package preview | 2 | Pass |
| Check the package before an agent installs it | 8 | Pass |
| Release path | 2 | Pass |
| Publish, approve, then install | 4 | Pass |
| Publish an exact version | 4 | Pass |
| Commit one JSON skill package with its repository assignments. | 9 | Pass; listed claim |
| Record a review | 3 | Pass |
| Name the reviewer before the version enters pilot or full release. | 11 | Pass; listed claim |
| Install and record | 3 | Pass |
| Agents fetch one assigned package and save a signed receipt. | 10 | Pass; listed claim |
| Limits and privacy | 3 | Pass |
| Keep credentials out of instructions | 5 | Pass |
| The secret reference field accepts uppercase names such as GITHUB_TOKEN. | 10 | Pass; listed claim |
| The API rejects other formats. | 5 | Pass; listed claim |
| Managed plan | 2 | Pass |
| $149 per team each month | 5 | Pass; listed claim |
| Managed billing is not active in this release. | 8 | Pass; listed claim |
| The self-hosted registry remains available under MIT. | 8 | F-1-3 |
| No payment is collected here. | 5 | Pass; covered by inactive-billing test |
| Deployment owners can connect the Sociobot billing API when the managed plan opens. | 13 | F-1-4 |

### README

Code blocks are commands, not sentences, and are excluded. All prose headings
and sentences are listed.

| Sentence or heading | Words | Result |
| --- | ---: | --- |
| Team Skills Registry | 3 | Pass |
| Publish reviewed agent skills and release them safely across repositories. | 10 | F-1-17 |
| Team Skills Registry is for engineering leads who maintain instructions for different coding agents. | 14 | Pass |
| Each version is loaded from one JSON file at a verified GitHub commit. | 13 | Pass; listed claim |
| It contains instructions, agent-specific adapters, repository assignments, pilot membership, and secret reference names. | 13 | F-1-18 |
| The service signs the complete package and its Git blob identifier before review. | 13 | F-1-19 |
| Run locally | 2 | Pass heading |
| Requirements: Node 22+, current stable Rust, and Chromium for browser checks. | 11 | Pass |
| Open `http://localhost:8080`. | 2 | Pass |
| `PORT` changes the listener. | 4 | F-1-7 |
| `DATABASE_PATH` changes the SQLite location. | 5 | F-1-8 |
| Open `/registry` and create a workspace. | 6 | Pass |
| Save its owner key in a password manager. | 8 | F-1-27 |
| Publishing creates a separate reviewer key for that one package. | 10 | Pass; listed claim |
| The reviewer opens `/review` with only that key. | 8 | Pass; listed claim |
| Approval consumes the key. | 4 | Pass; listed claim |
| The source verifier accepts GitHub repository URLs. | 7 | F-1-20 |
| Public repositories need no extra configuration. | 6 | Pass under Git-source claim |
| For a private repository, the deployment owner sets a credential such as `GIT_CREDENTIAL_PRIVATE_GITHUB` and pins it with `GIT_CREDENTIAL_PRIVATE_GITHUB_REPOSITORY=https://github.com/acme/private-skills`. | 20 | Pass |
| The workspace binds the reference `PRIVATE_GITHUB` to that exact repository before publishing. | 12 | Pass; listed claim |
| The server never stores or returns the credential value, and the reference cannot be used by another workspace or to read another repository. | 23 | F-1-21 |
| It confirms the exact commit, reads the named JSON file at that commit, and signs the returned blob identifier. | 19 | F-1-19 |
| Browser fields cannot replace committed instructions or adapters. | 8 | Pass; listed claim |
| Set `GIT_VERIFY_API_BASE` only when running a compatible local verifier fixture. | 10 | F-1-22 |
| Try the sandbox | 3 | Pass heading |
| Open `/demo` or `/?demo=1`. | 4 | Pass |
| It loads three complete packages, approval records, repository assignments, and receipts. | 11 | F-1-2 |
| Demo storage uses `demo:team-agent-skills:v2`. | 5 | Pass; listed claim |
| Reset restores the sample. | 4 | Pass; listed claim |
| Leaving deletes it. | 3 | F-1-5 |
| Release rules | 2 | Pass heading |
| Pilot and full release require a recorded review. | 8 | Pass; listed claim |
| Pilot reaches only `pilot_repositories`; full release reaches all `repositories`. | 10 | Pass; listed claim |
| A receipt requires an installed release, assigned repository, and named agent. | 11 | Pass; listed claim |
| Downloads contain the signed payload, digest, Ed25519 signature, and signer key. | 11 | Pass; listed claim |
| An owner issues a separate install credential for each package, repository, and agent. | 13 | Pass; listed claim |
| That credential cannot list, publish, review, or release packages. | 9 | Pass; listed claim |
| `/api/trust` exposes the signer key and fingerprint for consumer pinning. | 10 | F-1-6, F-1-23 |
| Workspace identifiers are isolated, so teams may use the same package id. | 12 | Pass; listed claim |
| Each trusted client IP has a 40-request limit. | 8 | Pass; listed claim |
| Factory ingress overwrites `X-Forwarded-For`, and the service uses its first hop; local runs fall back to the socket peer. | 19 | F-1-9, F-1-24 |
| Every statement above has an exact test in `.factory/claims.json`. | 9 | F-1-10 |
| Test and build | 3 | Pass heading |
| API workflow | 2 | Pass heading |
| Create a workspace with `POST /api/session`. | 7 | Pass |
| Send its owner key as `Authorization: Bearer <key>` to registry endpoints. | 11 | F-1-27 |
| Publish at `POST /api/skills` with `git_url`, `git_commit`, `source_path`, and optionally `git_credential_ref`; the source file follows `examples/skill-package.json`. | 17 | Pass |
| Before private publishes, the owner binds the deployment-managed, repository-pinned reference with `POST /api/git-credentials` using `{ "reference": "PRIVATE_GITHUB", "git_url": "https://github.com/acme/private-skills" }`. | 21 | F-1-25 |
| The response contains the package's one-time reviewer key. | 9 | Pass; listed claim |
| A reviewer sends that key to `GET /api/review`, then approves with `POST /api/review/approve`. | 14 | Pass; listed claim |
| The owner releases it with `PATCH /api/skills/:id/ring`, then issues an adapter credential with `POST /api/skills/:id/install-credentials`. | 16 | F-1-26 |
| Assigned agents fetch with only that scoped credential from `GET /api/repositories/:repository/agents/:agent/install/:id`. | 12 | Pass; listed claim |
| Managed plan | 2 | Pass heading |
| The researched managed plan is $149 per team each month. | 10 | Pass; listed claim |
| Billing is not active in this release, so the product does not collect payment or claim a purchasable subscription. | 19 | Pass; listed claim |
| A future managed release must use the Sociobot billing API. | 10 | Policy statement; no current capability claimed |
| The self-hosted MIT build remains available. | 6 | F-1-3 |
| Deploy | 1 | Pass heading |
| Build the root Dockerfile and mount `/data` for the database and signing identity. | 13 | F-1-11 |
| Read the in-product privacy policy and terms. | 7 | Pass |
| License | 1 | Pass heading |
| MIT. | 1 | F-1-3 |
| See LICENSE. | 2 | Pass |

Terminology is not yet one-to-one: **skill**, **instruction set**, and **skill
package** overlap; **private access key**, **private workspace key**, and
**owner key** name the same credential; **adapter credential** and **install
credential** name the same credential. F-1-26 and F-1-27 give the required
normalization.

## Demo and sandbox verification

- Landing → **Try it with sample data** reached `/demo` in one click.
- The first demo viewport already showed three realistic packages, release
  rings, owners, versions, agents, and the receipt section.
- The persistent banner read “Demo — sample data, nothing is saved” and exposed
  **Reset demo** and **Start for real**.
- Changing Secure commit from Released to Review changed only
  `demo:team-agent-skills:v2`; Reset restored it to Released.
- A sentinel in the real key `team-agent-skills:workspace-key` was unchanged
  through entry, mutation, and reset. Demo activity made zero `/api/*` requests.
- **Start for real** deleted the demo key, preserved the real-key sentinel, and
  opened `/registry`.
- Live request logging observed only
  `https://team-agent-skills.sociobot.in`. The loaded sample remained visible
  after the browser context was set offline.

The runtime behavior passes. F-1-2 and F-1-5 concern missing claim-registry
coverage, not a reproduced sandbox leak.

## Claims verification

Every exact command in `.factory/claims.json` was run separately after
`npm ci` in clean clone `/tmp/team-agent-skills-review-iWRdtr`.

| Claim id | Exact command | Result |
| --- | --- | --- |
| `demo-separation` | `npx playwright test --grep @claim:demo-separation` | PASS |
| `execution-receipt` | `npx playwright test --grep @claim:execution-receipt` | PASS |
| `demo-local-data` | `npx playwright test --grep @claim:demo-local-data` | PASS |
| `review-required` | `npx playwright test --grep @claim:review-required` | PASS |
| `package-contents` | `npx playwright test --grep @claim:package-contents` | PASS |
| `private-workspace` | `cargo test --locked claim_private_workspace` | PASS |
| `secret-reference-format` | `cargo test --locked claim_secret_reference_format` | PASS |
| `git-signed-package` | `cargo test --locked claim_git_signed_package` | PASS |
| `independent-one-time-review` | `cargo test --locked claim_independent_one_time_review` | PASS |
| `governed-execution-receipt` | `cargo test --locked claim_governed_execution_receipt` | PASS |
| `pilot-ring-access` | `cargo test --locked claim_pilot_ring_access` | PASS |
| `scoped-install-credentials` | `cargo test --locked claim_scoped_install_credentials` | PASS |
| `trusted-client-rate-limit` | `cargo test --locked claim_trusted_client_rate_limit` | PASS |
| `private-git-source` | `cargo test --locked claim_private_git_source` | PASS |
| `managed-plan-status` | `npx playwright test --grep @claim:managed-plan-status` | PASS |

Summary: 15/15 listed commands passed; 0 listed tests failed. F-1-2 through
F-1-11 are claim-like copy that is not completely represented by those 15
entries.

## Structure, links, and accessibility

- `/`, `/demo`, `/registry`, `/review`, `/privacy`, and `/terms` returned 200.
  Every page had `lang=en`, one h1, one main landmark, a route-specific title,
  description, canonical, consistent header/footer, Privacy, and Terms.
- All landing links crawled successfully. The unknown-route response was a
  designed 404 with a way home, but F-1-12 records its missing metadata.
- Client-side navigation focused and announced the new h1. F-1-15 records the
  off-screen focus/scroll failure from a footer navigation and Back sequence.
- Live browser console: zero errors on the six normal routes. Security headers
  included CSP, HSTS, `nosniff`, Referrer-Policy, and Permissions-Policy.
- `/opt/fleet/lib/verify-url.sh` passed the live root: 611 ms, title, `lang`, one
  h1, main, no missing alt, no unlabeled button, and no console errors.
- Live axe-core 4.11.0 with CSP bypass for audit injection found zero serious or
  critical violations on `/`, `/demo`, `/registry`, `/review`, `/privacy`,
  `/terms`, and an unknown-route 404.
- The paper-cut release-desk artwork, palette, typography, borders, stamps, and
  physical-layer motion form a distinct product identity rather than a generic
  gradient/card SaaS template.
- Production build output was 30.90 KB JS raw / 9.91 KB gzip and 15.31 KB CSS
  raw / 4.11 KB gzip.

## Full local gates

Run in the clean clone:

| Command | Result |
| --- | --- |
| `npm test` | PASS — 2 Vitest and 4 Node tests |
| `npm run typecheck` | PASS |
| `npm run build` | PASS — `dist/` produced |
| `npx playwright test` | PASS — 19/19 |

## History audit

No `.factory/review-*.md` or `.factory/polish-*.md` existed in the repository
or its reachable history. All revisions of `.factory/handoff.md` were read.
Repeated repair statements are consolidated below, but every earlier finding
or gap is accounted for.

| Earlier finding or gap | Current verification |
| --- | --- |
| Initial: one global workspace, no identity or tenant authorization | Fixed: `private-workspace` passed; unauthenticated access fails and workspaces are isolated. |
| Initial: Git sources were not fetched | Fixed: `git-signed-package` passed with verified file/blob provenance. |
| Initial: vendor-specific adapter files were not generated | **Still open: F-1-13.** |
| Initial/current: Docker could not be built in the reviewer container | Environment limitation, not a reproduced product defect; Docker contract tests pass. |
| `0a82` blocker 1: no auth, isolation, repository ACL, or authorization | Fixed by private workspace, scoped install, pilot, and review claim tests. |
| `0a82` blocker 2: no instruction bodies, Git versions, adapters, review, assignments, or distribution | Core data and API boundaries fixed; repo-native installation remains open as F-1-13. |
| `0a82` blocker 3: rejected ring change shown as success | Fixed; full Playwright suite includes rollback/error feedback. |
| `0a82` blocker 4: checkout 404 and unverified licenses | Fixed honestly by removing checkout and marking billing inactive; price-state test passed. |
| `0a82` blocker 5: false or unlisted claims | **Regressed/still open: F-1-2 through F-1-11.** |
| `0a82` blocker 6: fmt/Clippy and Docker tag failures | Later repair evidence and current claim builds confirm the repaired code; no current source regression found. |
| `0a82` blocker 7: demo retention, leaked notice, small controls, missing state, caching, audit | Runtime demo teardown and current 19-test suite pass. |
| `9158` blocker 1: packages not Git-verified or fully signed | Fixed; cryptographic claim test passed. |
| `9158` blocker 2: reviewer key not independent or one-time | Fixed; independent one-time review test passed. |
| `9158` blocker 3: receipts accepted before release or for wrong agent | Fixed; governed receipt test passed. |
| `9158` blocker 4: globally colliding skill ids | Fixed; private-workspace test permits the same id in two workspaces. |
| `9158` blocker 5: missing claim tags and false/unlisted README claims | **Regressed/still open: F-1-2 through F-1-11.** |
| `9158` blocker 6: malformed demo/stale key had no recovery | Fixed; full Playwright suite passed those recovery cases. |
| `9158` blocker 7: 404 contrast and 24 px target | Fixed for contrast and target size; metadata closure is incomplete as F-1-12. |
| `9158` blocker 8: price absent and adapters could not differ | Fixed; price is explicit/inactive and per-agent adapter fields are tested. |
| `874257` blocker 1: submitted text signed without loading its Git file | Fixed by `git-signed-package`. |
| `874257` blocker 2: pilot and full had identical access | Fixed by `pilot-ring-access`. |
| `874257` blocker 3: installs used the owner key | Fixed by `scoped-install-credentials`. |
| `874257` blocker 4: caller-controlled forwarded IP bypassed limits | Fixed by `trusted-client-rate-limit` and the later trusted-ingress repair. |
| `874257` extra: flaky skip-link focus and v2 demo downloads | Fixed; 19/19 Playwright tests and package-content claim passed. |
| `e874` blocker 1: all public requests shared one rate-limit identity | Fixed by the per-client rate-limit claim. |
| `e874` blocker 2: no private Git source flow | Fixed by `private-git-source`. |
| Repair handoff: every route had complete metadata | **Not true for the static 404: F-1-12.** |
| Repair handoff: capability keys had no recovery service | **Still open: F-1-14.** |
| Current handoff: textarea used native rather than designed focus | **Still open: F-1-1.** |

## Missed leverage

AI is not an obvious fit for this registry’s core control and provenance job;
adding it would be decorative. No provider keys are embedded and no runtime AI
call was observed. The obvious missing leverage is deterministic repo-native
delivery, recorded as F-1-13: turn the already reviewed adapter content into the
actual instruction file or a reviewable repository change.

## What would make this perfect

Resolve every finding above, then rerun this whole review from a fresh live
context. In particular: close the three historical regressions, enumerate and
tag every public claim, make new-route focus visible while preserving history
state, standardize credential terminology and release-action labels, and prove
repo-native installation in the demo and real workflow. The next verdict should
not be requested until the finding count is zero.
