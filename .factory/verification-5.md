# Independent product verification — FAIL

Verified on 2026-08-29 against candidate
`ef6da21c32f2a6203ddbca00d9ac2261ac7111ee` and
`https://team-agent-skills.sociobot.in`.

## Decision

**FAIL. Do not release this candidate.** The deployment is fresh and matches
the candidate: live `/health` reports exactly
`ef6da21c32f2a6203ddbca00d9ac2261ac7111ee`. The implementation is otherwise
substantially functional, but it fails two mandatory backend/brief boundaries:
the deployment uses a single shared 40-request bucket instead of a per-client
limiter keyed by the first `X-Forwarded-For` hop, and it will only verify skill
packages from public GitHub repositories. This prevents the private,
repository-boundary-preserving registry required by the brief.

## First-read and one-click demo — PASS

A cold, storage-free desktop visit answered all required questions in plain
words:

- What it does: “Release reviewed skills across repositories.”
- For whom: “For engineering leads who need one checked instruction set for
  every coding agent.”
- First action: “Try it with sample data,” with the result stated immediately:
  “Open three complete skill packages and their review records.”

One click opened `/demo`, showed complete sample packages and receipts, and
displayed the persistent “Demo — sample data, nothing is saved” banner with
Reset demo and Start for real. Evidence:
`.factory/verification-artifacts-5/live-cold-desktop.png`.

## Required claim tests — PASS

`.factory/claims.json` exists and every listed command was run from this clean
checkout after `npm ci`; all fourteen passed.

| Claims | Exact command | Result |
| --- | --- | --- |
| demo separation | `npx playwright test --grep @claim:demo-separation` | PASS (1) |
| execution receipt | `npx playwright test --grep @claim:execution-receipt` | PASS (1) |
| demo local data | `npx playwright test --grep @claim:demo-local-data` | PASS (1) |
| review required | `npx playwright test --grep @claim:review-required` | PASS (1) |
| package contents | `npx playwright test --grep @claim:package-contents` | PASS (1) |
| managed plan status | `npx playwright test --grep @claim:managed-plan-status` | PASS (1) |
| eight backend claims | each `cargo test --locked claim_<id>` command in the manifest | PASS (1 each) |

The manifest contract test in `npm test` also passed: every claim has exactly
one matching source tag and command.

## Release-blocking defects

### High — rate limiting is global, not per client/IP

`backend/src/main.rs` discards both request forwarding information and socket
identity, then assigns every request the literal key `"public"`. The mandatory
backend contract requires limits to be keyed by the first `X-Forwarded-For`
hop behind factory ingress. Consequently any one client can exhaust the whole
service's allowance, denying all other teams, and the service does not enforce
the documented *per-client* boundary.

Fresh live proof: 45 concurrent `GET /api/trust` requests with 45 distinct
`X-Forwarded-For` values returned **40 x 200 and 5 x 429**, each 429 carrying
`Retry-After: 1`. This verifies a 40-request shared-per-second bucket, not a
client-IP bucket. The behaviour is also documented in the source comment and
README as deliberately ignoring forwarded identity. Health is exempt.

### High — package provenance only supports public GitHub, not private repos

The acceptance brief requires governed **private** registries and preservation
of repository access boundaries. The only source-verification path explicitly
requires a “public GitHub repository URL” (`backend/src/main.rs`), and README
documents that restriction. It uses unauthenticated GitHub API/raw-file reads;
there is no installation token, customer-provided credential reference, or
other private-Git mechanism. Engineering teams therefore must make the
complete instruction package public to use the advertised Git-backed signing
flow, which is incompatible with private registry use and the brief's access
boundary constraint.

## Functional, privacy, and deployment evidence

- Live build identity: `/health` returned the candidate SHA above. A fresh
  local release binary started with only `PORT=4195`, created its default
  SQLite/signing state, and returned healthy JSON (`build_sha: "dev"` locally
  without a supplied build argument).
- The full browser suite passed **19/19**. Representative demo flow covered
  publish, review gating, approval, pilot/full rings, package download,
  receipt recording, reset, malformed-storage recovery, and demo exit.
- All eight backend claim tests passed, covering private workspace isolation,
  secret-reference validation, source-backed signing, one-time review,
  governed receipts, pilot cohort scope, install credentials, and rate-limit
  responses.
- Cold landing and demo request logs contacted only
  `https://team-agent-skills.sociobot.in`; demo reset/offline UI remained
  available. Demo data was removed on Start for real. No analytics, fonts,
  scripts, Azure endpoints, or third-party origins were observed.
- Live root and hashed JS responses send CSP, `nosniff`,
  `strict-origin-when-cross-origin`, HSTS, Permissions-Policy, and
  `Cache-Control: public, max-age=31536000, immutable`. Built JS is 30,381 B
  raw / 9.78 KiB gzip and CSS 15,309 B raw / 4.11 KiB gzip; the 54,898 B hero
  image is within the stated budgets.
- `/`, `/demo`, `/registry`, `/review`, `/privacy`, `/terms`, robots, sitemap,
  404, health, and trust endpoint returned expected status; an unknown route
  returned 404.
- The supplied `verify-url.sh` passed locally (565 ms) and live (591 ms): one
  h1, title, `lang=en`, main landmark, alt text, no unlabeled buttons, and no
  console/page errors. Outputs are in
  `.factory/verification-artifacts-5/verify-{local,live}/`.
- Fresh Playwright axe scans found zero serious/critical issues on desktop
  landing and 390px demo. Keyboard skip link and focus states passed in the
  19-test browser suite. Under reduced motion, desktop and mobile had zero
  horizontal overflow, `scroll-behavior: auto`, and no animations/transitions.

## Quality gates

| Command | Result |
| --- | --- |
| `npm ci` / `npm audit` | PASS; 0 vulnerabilities |
| `npm test` | PASS; 2 Vitest + 4 Node tests |
| `npm run typecheck` | PASS |
| `npm run build` | PASS; `dist/` produced |
| `npx playwright test` | PASS; 19 tests |
| every claims-manifest command | PASS; 14 claims |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS; 10 integration tests |
| `cargo build --release --locked` | PASS |
| Docker image build | NOT RUN; no Docker-compatible engine is installed. Docker contract tests passed. |

## Required remediation

1. Trust the ingress-supplied first `X-Forwarded-For` hop (with the factory
   ingress configured to overwrite client values) and rate-limit it separately;
   retain 429 plus `Retry-After`, then add a regression proving independent
   client allowances as well as one-client exhaustion.
2. Add a private Git source path that keeps repository credentials and package
   contents within the appropriate workspace boundary (for example a
   server-held/secret-referenced GitHub App installation credential), or
   change the product scope. The researched brief requires the former.
