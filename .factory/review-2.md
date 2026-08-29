# Adversarial first-read review 2 — Team Skills Registry

## Verdict

**PASS.** There are no findings, no untested declared claims, and no repeat of
the 29 findings from review 1.

Reviewed 2026-08-29 UTC against
`https://team-agent-skills.sociobot.in`. Live `/health` reported build
`761e4c095b752622e0b3287694be6418780e8f13`, matching the review checkout.

## Cold first read

No scrolling or stored browser data was used before answering these questions.

| Viewport | What it does | For whom | First click | Result |
| --- | --- | --- | --- | --- |
| 390 × 844 | Releases reviewed skill packages to repositories. | Engineering leads maintaining coding-agent instructions. | **Try it with sample data**. | Pass |
| 1440 × 900 | Same. | Same. | **Try it with sample data**. | Pass |

The exact first-screen copy is: “Release reviewed skills across repositories,”
“For engineering leads who need one checked instruction set for every coding
agent,” and “Try it with sample data.” The action was visible at 433 px on the
phone viewport and 644 px on desktop. It opens three complete packages and
their review records.

## Demo and privacy sandbox

The landing action reached `/demo` in one click. Its first phone viewport already
showed three named, realistic packages (`Secure commit`, `Migration review`, and
`Incident note`), each with an owner, version, target agents, release state, and
the selected package’s instructions and approval record.

The persistent banner read “Demo — sample data, nothing is saved” and included
both **Reset demo** and **Start for real**. In a fresh context with a sentinel
real workspace key:

- Demo wrote only `demo:team-agent-skills:v2`; the sentinel real key did not
  change.
- Changing a release state changed the demo key; Reset restored the seeded
  state.
- While in demo, there were no `/api/` requests and every request was to the
  product origin. The sample remained visible after `context.setOffline(true)`.
- **Start for real** removed the demo key, kept the sentinel real key, and then
  opened `/registry`. The subsequent real-registry API requests were therefore
  outside demo mode.

This confirms the required separate browser storage namespace, reset, teardown,
and same-origin privacy behavior.

## Claims

I created a new clone at `/tmp/team-agent-skills-review2-LVsopK`, ran `npm ci`,
then ran every command in `.factory/claims.json` independently. All 24 passed.

| Claim id | Result |
| --- | --- |
| `demo-separation` | PASS |
| `demo-sample-content` | PASS |
| `demo-teardown` | PASS |
| `execution-receipt` | PASS |
| `demo-local-data` | PASS |
| `review-required` | PASS |
| `package-contents` | PASS |
| `private-workspace` | PASS |
| `workspace-recovery` | PASS |
| `secret-reference-format` | PASS |
| `git-signed-package` | PASS |
| `independent-one-time-review` | PASS |
| `governed-execution-receipt` | PASS |
| `pilot-ring-access` | PASS |
| `scoped-install-credentials` | PASS |
| `repo-native-install` | PASS |
| `trusted-client-rate-limit` | PASS |
| `private-git-source` | PASS |
| `trust-endpoint` | PASS |
| `port-configuration` | PASS |
| `database-path-configuration` | PASS |
| `persistent-data-and-signing-key` | PASS |
| `mit-license-build` | PASS |
| `managed-plan-status` | PASS |

The clean-clone sweep included the production Vite and locked Rust release
builds through `mit-license-build`. `npm test`, `npm run typecheck`,
`npm run build`, and the full 26-test Playwright suite also passed locally.

Every claim-like landing and README statement was checked against the manifest.
The listed tests cover the demo count, local storage, teardown, release review,
receipt provenance, private-workspace boundaries, recovery, Git verification,
private Git credentials, native installation, rate limit, deployment settings,
license, and managed-plan status. No unlisted material claim remains.

## Copy audit

Counts use whitespace-delimited words; a code command with a space has one word
per token. Headings, visible actions, and image alt text are included to check
the plain-language requirements. There are no sentences over 22 words, banned
marketing adjectives, unexplained marketing slogans, inconsistent product terms,
or non-result primary action buttons. Header navigation labels are navigation,
not workflow actions.

### Landing page

| Text | Words | Result |
| --- | ---: | --- |
| Team Skills Registry | 3 | Pass: wordmark |
| Demo / Registry / Review / Privacy | 1 each | Pass: navigation labels |
| Team skill releases | 3 | Pass: section label |
| Release reviewed skills across repositories | 5 | Pass: job headline |
| For engineering leads who need one checked instruction set for every coding agent. | 13 | Pass: audience and outcome |
| Try it with sample data | 6 | Pass: result-naming action |
| Open three complete skill packages and their review records. | 9 | Pass: `demo-sample-content` |
| Sample data stays in this browser. | 6 | Pass: `demo-local-data` |
| Receipts preserve the exact package version. | 7 | Pass: `execution-receipt` |
| Real workspaces use a workspace owner key. | 7 | Pass: `private-workspace` |
| A paper-cut release desk routes a skill packet through an approval stamp into repository drawers. | 15 | Pass: useful image alt |
| Package preview | 2 | Pass: section label |
| Check the package before an agent installs it | 8 | Pass: section heading |
| skill.json / Secure commit / v2.4.0 / APPROVED | 1 / 2 / 1 / 1 | Pass: sample labels |
| Review / Pilot / Assigned repositories | 1 / 1 / 2 | Pass: release-path labels |
| Execution receipt / rcpt-7F3A / atlas-api · Codex / Secure commit v2.4.0 | 2 / 1 / 2 / 3 | Pass: sample receipt labels |
| Release path | 2 | Pass: section label |
| Publish, approve, then install | 4 | Pass: process heading |
| Publish an exact version | 4 | Pass: step heading |
| Commit one JSON skill package with its repository assignments. | 9 | Pass: `git-signed-package` |
| Record a review | 3 | Pass: step heading |
| Name the reviewer before the version enters pilot or full release. | 11 | Pass: `review-required` |
| Install and record | 3 | Pass: step heading |
| Agents fetch one assigned package and save a signed receipt. | 10 | Pass: `governed-execution-receipt` |
| Limits and privacy | 3 | Pass: section label |
| Keep credentials out of instructions | 5 | Pass: privacy heading |
| The secret reference field accepts uppercase names such as GITHUB_TOKEN. | 10 | Pass: `secret-reference-format` |
| The API rejects other formats. | 5 | Pass: `secret-reference-format` |
| Managed plan | 2 | Pass: section label |
| $149 per team each month | 5 | Pass: `managed-plan-status` |
| Managed billing is not active in this release. | 8 | Pass: `managed-plan-status` |
| The self-hosted registry is licensed under MIT. | 8 | Pass: `mit-license-build` |
| No payment is collected here. | 5 | Pass: `managed-plan-status` |
| Reviewed instructions for coding agents. | 5 | Pass: footer description |
| Built by Param Factory / v1.2.0 | 4 / 1 | Pass: footer attribution and version |

### README

| Text | Words | Result |
| --- | ---: | --- |
| Team Skills Registry | 3 | Pass: title |
| Publish reviewed agent skill packages to assigned repositories. | 8 | Pass: purpose |
| Team Skills Registry is for engineering leads who maintain instructions for different coding agents. | 14 | Pass: audience |
| Each version comes from one JSON file at a verified GitHub commit. | 12 | Pass: `git-signed-package` |
| It includes shared instructions, instructions for each agent, assigned repositories, pilot repositories, and credential reference names. | 16 | Pass: package contents |
| Before review, the service signs the package and Git's identifier for the exact source file. | 15 | Pass: `git-signed-package` |
| Run locally | 2 | Pass: heading |
| Requirements: Node 22+, current stable Rust, and Chromium for browser checks. | 11 | Pass: setup requirement |
| Open `http://localhost:8080`. | 2 | Pass: local address |
| `PORT` changes the service listener. | 5 | Pass: `port-configuration` |
| `DATABASE_PATH` selects the SQLite and signing-state directory. | 7 | Pass: `database-path-configuration` |
| Open `/registry` and create a workspace. | 6 | Pass: `private-workspace` |
| Save its workspace owner key and recovery key in a password manager. | 12 | Pass: user guidance |
| The recovery key replaces a lost workspace owner key. | 9 | Pass: `workspace-recovery` |
| Publishing creates a separate reviewer key for that one package. | 10 | Pass: `independent-one-time-review` |
| The reviewer opens `/review` with only that key. | 8 | Pass: `independent-one-time-review` |
| Approval consumes the key. | 4 | Pass: `independent-one-time-review` |
| The service loads skill packages from GitHub repository URLs. | 9 | Pass: `git-signed-package` |
| Public repositories need no extra configuration. | 6 | Pass: `git-signed-package` |
| For a private repository, the deployment owner configures a named credential and its allowed repository. | 15 | Pass: `private-git-source` |
| The workspace then binds that credential name to the same repository. | 11 | Pass: `private-git-source` |
| The server never stores or returns the credential value. | 9 | Pass: `private-git-source` |
| Another workspace cannot use its reference. | 6 | Pass: `private-git-source` |
| The reference cannot read another repository. | 6 | Pass: `private-git-source` |
| The service confirms the commit and reads the named JSON file there. | 12 | Pass: `git-signed-package` |
| It then signs the returned file identifier. | 7 | Pass: `git-signed-package` |
| Browser fields cannot replace committed instructions for the package or agent. | 10 | Pass: `git-signed-package` |
| Set `GIT_VERIFY_API_BASE` only for a local test server that returns the GitHub commit and file responses used here. | 18 | Pass: test-only guidance |
| Try the sandbox | 3 | Pass: heading |
| Open `/demo` or `/?demo=1`. | 4 | Pass: demo entry |
| It loads three complete packages, approval records, repository assignments, and receipts. | 11 | Pass: `demo-sample-content` |
| Demo storage uses `demo:team-agent-skills:v2`. | 5 | Pass: `demo-separation` |
| Reset restores the sample. | 4 | Pass: `demo-separation` |
| Leaving the demo deletes its sample workspace. | 7 | Pass: `demo-teardown` |
| Release rules | 2 | Pass: heading |
| Pilot and full release require a recorded review. | 8 | Pass: `review-required` |
| Pilot reaches only `pilot_repositories`; full release reaches all `repositories`. | 9 | Pass: `pilot-ring-access` |
| A receipt requires an installed release, assigned repository, and named agent. | 11 | Pass: `governed-execution-receipt` |
| Downloads include repository-native instructions and the signed JSON record. | 9 | Pass: `repo-native-install` |
| An owner issues one install credential for each package, repository, and agent. | 12 | Pass: `scoped-install-credentials` |
| That credential cannot list, publish, review, or release packages. | 9 | Pass: `scoped-install-credentials` |
| `/api/trust` returns the signer key and its matching SHA-256 fingerprint. | 10 | Pass: `trust-endpoint` |
| Workspace identifiers are isolated, so teams may use the same package id. | 12 | Pass: `private-workspace` |
| Each trusted client IP has its own 40-request limit. | 9 | Pass: `trusted-client-rate-limit` |
| Test and build | 3 | Pass: heading |
| Every product claim and its exact command are listed in `.factory/claims.json`. | 11 | Pass: manifest contract test |
| API workflow | 2 | Pass: heading |
| Create a workspace with `POST /api/session`. | 6 | Pass: `private-workspace` |
| Send its workspace owner key as `Authorization: Bearer <key>` to registry endpoints. | 12 | Pass: `private-workspace` |
| Publish with `POST /api/skills` and the source fields shown in `examples/skill-package.json`. | 11 | Pass: `git-signed-package` |
| Before publishing from a private repository, bind its configured credential name with `POST /api/git-credentials`. | 14 | Pass: `private-git-source` |
| The publish response contains the package's one-time reviewer key. | 9 | Pass: `independent-one-time-review` |
| A reviewer uses it at `GET /api/review` and `POST /api/review/approve`. | 10 | Pass: `independent-one-time-review` |
| The owner changes the release with `PATCH /api/skills/:id/ring`. | 8 | Pass: `pilot-ring-access` |
| The owner then issues an install credential at `POST /api/skills/:id/install-credentials`. | 10 | Pass: `scoped-install-credentials` |
| The assigned agent uses that credential at `GET /api/repositories/:repository/agents/:agent/install/:id`. | 9 | Pass: `scoped-install-credentials` |
| The response includes `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, or a general instruction file. | 12 | Pass: `repo-native-install` |
| Managed plan | 2 | Pass: heading |
| The researched managed plan is $149 per team each month. | 10 | Pass: `managed-plan-status` |
| Billing is not active in this release, so the product does not collect payment. | 14 | Pass: `managed-plan-status` |
| A future managed release must use the Sociobot billing API. | 10 | Pass: policy statement, not a current capability |
| The self-hosted build is available under the MIT license. | 9 | Pass: `mit-license-build` |
| Deploy | 1 | Pass: heading |
| Build the root Dockerfile and mount `/data`. | 7 | Pass: deployment instruction |
| The mounted directory preserves the database and signing identity across restarts. | 12 | Pass: `persistent-data-and-signing-key` |
| Read the in-product privacy policy and terms. | 7 | Pass: navigation guidance |
| License | 1 | Pass: heading |
| MIT. | 1 | Pass: `mit-license-build` |
| See [LICENSE](LICENSE). | 2 | Pass: document link |

The same terms are used throughout: **skill package**, **release ring**,
**execution receipt**, **demo**, **credential reference**, **workspace owner
key**, **workspace recovery key**, and **install credential**. The terms that
are necessarily technical for this audience (JSON, Git commit, SQLite, API,
and SHA-256) are either named implementation details or paired with their
concrete purpose.

## Structure, routing, accessibility, and visual identity

The live `/`, `/demo`, `/registry`, `/review`, `/privacy`, and `/terms` routes
all returned 200. Each had one h1, a main landmark, `lang="en"`, a route-specific
title, description, canonical, OG title, absolute social image, and no page
script errors. All rendered internal links returned 200; a deliberately unknown
route returned the designed static 404 with 404 status, title, description,
canonical, icons, social metadata, header, footer, Privacy, Terms, and a home
link. Chrome reports the expected network diagnostic for the 404 document itself;
the normal routes had no console errors.

At 390 px, footer → Privacy focused the visible Privacy h1 at y=194. Browser
Back restored the footer Privacy link as focus and restored the prior scroll
position exactly (3,130 px before and after). The full Playwright suite also
checks skip navigation, 44 px controls, reduced motion, textareas’ 3 px focus
ring, mobile overflow, and serious/critical axe violations.

The paper-cut release desk, deep shadows, stamps, warm paper palette, Georgia
headings, and original on-thesis artwork conform to `.factory/design.md` and are
visibly product-specific rather than a generic SaaS template. No missing AI,
import/export, or sync feature is implied by the brief: export to repository
native instruction files is present and tested; AI would be decorative for this
provenance-and-control workflow. No provider keys are embedded or requested.

## History audit

Every earlier factory review, polish record, and handoff was read. The previous
review's findings were verified against both source and live behavior rather
than accepted from their “fixed” labels.

| Earlier finding | Current confirmation |
| --- | --- |
| F-1-1 | `textarea:focus-visible` has the designed 3 px coral ring; full browser suite passes. |
| F-1-2 | `demo-sample-content` is declared and passes against exactly three complete packages. |
| F-1-3 | `mit-license-build` passes the license and production-build checks. |
| F-1-4 | The speculative billing sentence is absent. |
| F-1-5 | `demo-teardown` is declared, tagged, and passes. |
| F-1-6 | `trust-endpoint` is declared and passes its fingerprint test. |
| F-1-7 | `port-configuration` is declared and passes. |
| F-1-8 | `database-path-configuration` is declared and passes. |
| F-1-9 | The unprovable ingress/socket wording is absent; the tested client-limit wording remains. |
| F-1-10 | The README now points to the complete claim manifest; the manifest contract test passes. |
| F-1-11 | `persistent-data-and-signing-key` is declared and passes. |
| F-1-12 | Direct live unknown URLs return a metadata-complete designed 404. |
| F-1-13 | Demo exports selected repository-native instruction files with provenance; `repo-native-install` passes. |
| F-1-14 | A separate recovery key, recovery UI, and rotation test are present. |
| F-1-15 | Live navigation and Back focus/scroll behavior pass. |
| F-1-16 | Live OG and Twitter images are absolute and present on every SPA route. |
| F-1-17 | The unsupported “safely” wording is absent. |
| F-1-18 | Package contents use concrete, consistent terms. |
| F-1-19 | README explains Git’s file identifier without “blob” jargon. |
| F-1-20 | README describes the user-visible Git loading behavior. |
| F-1-21 | The credential boundary is split into short sentences. |
| F-1-22 | The local verifier description explains the test server’s responses. |
| F-1-23 | Trust copy states the signer key/fingerprint result plainly. |
| F-1-24 | Dense ingress/socket wording is absent. |
| F-1-25 | Private-source setup states a concrete bind action. |
| F-1-26 | “Install credential” is used consistently. |
| F-1-27 | “Workspace owner key” is used consistently. |
| F-1-28 | Release controls name their results. |
| F-1-29 | Artwork provenance is in the design record, not visible landing copy. |

## What would make this perfect

No remediation is required for the reviewed scope. Maintain this standard by
keeping every new material claim in `.factory/claims.json`, preserving the
isolated `/demo` path, and repeating this full cold-context review after each
material product change.
