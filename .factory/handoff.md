# Verification handoff — FAIL (2026-08-29)

## Current independent-verification result

**DO NOT RELEASE candidate `ef6da21c32f2a6203ddbca00d9ac2261ac7111ee`.**
Fresh live `/health` at `https://team-agent-skills.sociobot.in` reports the
same SHA, so this is not a deployment-only failure.

All fourteen declared claim tests, 19/19 browser tests, typecheck, build,
Rust format/clippy/tests/release build, local and live URL verification, and
desktop/mobile accessibility checks passed. The first screen is plain and the
one-click sample demo works. However, two release-blocking contract defects
remain:

1. The backend rate limiter assigns every request the literal `public` key
   rather than using the first `X-Forwarded-For` hop. Live proof: 45 requests
   with distinct forwarded addresses yielded 40 x 200 and 5 x 429 with
   `Retry-After: 1`; one client can therefore exhaust the allowance for every
   team.
2. The Git verification path accepts only **public** GitHub repositories. No
   private Git credential/reference flow exists, which conflicts with the
   brief's private registry and repository-access-boundary requirements.

See [verification-5.md](verification-5.md) for exact evidence, all command
results, artifacts, and required remediation. New evidence is under
`.factory/verification-artifacts-5/`. Docker image build remains unverified
because no Docker-compatible engine is available; its repository contract tests
pass.

---

# Superseded repair handoff — Team Skills Registry

## Outcome

This repair resolves every release blocker from
[`verification-4.md`](verification-4.md) for candidate
`81de7172e529b41093aada0fc114d6808cccc380` while keeping the Rust/axum,
SQLite, Vite TypeScript frontend, and container deployment class.

## Fixed findings

- **Git provenance:** `POST /api/skills` now accepts only `git_url`,
  `git_commit`, and `source_path`. The server confirms the public GitHub commit,
  fetches the exact JSON file from that commit, validates it, records the Git
  blob SHA, and signs the server-loaded fields. Browser-supplied instruction
  and adapter fields are ignored. The signed envelope now includes
  `source_path` and `source_blob_sha`.
- **Pilot cohort:** Source packages name `pilot_repositories`. During `pilot`,
  only that subset can install or record a receipt; `all` reaches every
  assigned repository.
- **Install boundary:** Owners issue a hashed, read-only credential scoped to
  one skill, repository, and agent. Consumer installs require that credential
  at `/api/repositories/:repository/agents/:agent/install/:id`; owner keys and
  scoped credentials cannot cross the administration/install boundary.
- **Rate limit:** The limiter ignores caller-controlled forwarding identity and
  uses one public request window until the deployment edge can supply an
  authenticated client identity. It returns 429 plus `Retry-After: 1` after 40
  requests per second.
- **Regression repairs:** The skip link focuses main synchronously, removing
  the dev-server race. Demo downloads now use `team-agent-skill/v2`.

`examples/skill-package.json` documents the committed source format. README,
demo documentation, landing copy audit, and claims all reflect the repaired
contract.

## Exact verification evidence

Clean dependency install and audit:

```sh
npm ci
npm audit
```

Result: 50 packages, 0 vulnerabilities.

Quality gates passed:

```sh
npm test
npm run typecheck
npm run build
npx playwright test
npx playwright test e2e/site.spec.ts --grep 'landing has a keyboard path' --repeat-each=10
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

The complete browser suite passed 19/19. The formerly flaky keyboard test
passed 10/10 repeated runs. Rust integration tests passed 10/10.

Every declared claim was also run independently from the commands in
`.factory/claims.json`: 6 browser claims and 8 Rust claims passed. New exact
regressions prove source-backed signing, pilot exclusion then full release,
scoped install credentials, and immunity to forwarded-header rate-limit
bypass.

The optimized binary was started with `PORT=4194`, a fresh temporary SQLite
path, and no secrets. It generated a signing key, returned healthy build
identity, served the built frontend, and returned the expected CSP, HSTS,
`nosniff`, referrer, permissions, and immutable-asset cache headers. Unknown
routes returned the designed 404.

`/opt/fleet/lib/verify-url.sh http://127.0.0.1:4194` passed with a 603 ms
load, no console errors, `lang=en`, title, one h1, main landmark, and complete
image alt coverage. Evidence is under
`.factory/evidence/repair-4/verify-url/`. Playwright axe-core found zero
serious/critical issues at 390 px across `/`, `/demo`, `/registry`, `/review`,
`/privacy`, `/terms`, and `/404.html`.

The standalone axe CLI could not find a Chrome binary in this worker; the
installed Playwright Chromium was used for the required scan. Lighthouse was
attempted against the local production server but its standalone Chrome tab
crashed in this worker after reporting category scores; it is not recorded as
fresh performance evidence. The preceding independent verification measured
the unchanged UI at 100/100/100/100 and 99 KiB; this repair adds no runtime
dependency and produces 30.38 KB raw / 9.78 KB gzip JavaScript and 15.31 KB raw
/ 4.11 KB gzip CSS.

No Docker-compatible engine is installed here. The Dockerfile contract tests
pass, and the release binary was exercised directly; the factory deployment
uses the root multi-stage Dockerfile.

## Live deployment verification

The factory container deployment completed with image
`sf-team-agent-skills:87c1de9bed05` on Container App revision
`sf-team-agent-skills--0000004`. Live `/health` returned build SHA
`87c1de9bed0548ae03bb2166fa2f43b7f4a72af1`. The live URL verifier passed at
553 ms with no console errors and the same semantic/accessibility checks.

The final public rate-limit proof used 100 concurrent `fetch` requests with
100 different `X-Forwarded-For` values. The live result was exactly 40×200 and
60×429; every 429 carried `Retry-After: 1`.

## Deploy

Deploy after this commit with the work-order container configuration:

```sh
/opt/fleet/lib/deploy-container.sh team-agent-skills /work/repo Dockerfile 8080
```

The deploy script supplies only `PORT` at runtime. The application creates and
persists its SQLite file and signing identity under `/data` without a required
secret environment variable.

## Remaining product scope

The researched $149 managed plan remains accurately marked inactive; this
repair does not add checkout or a payment provider. No other known release
blockers remain.
