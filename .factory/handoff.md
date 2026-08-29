# Repair handoff — Team Skills Registry

## Outcome

This repair resolves every release-blocking finding in
[`verification-5.md`](verification-5.md) while preserving the existing Rust
axum + SQLite backend, Vite TypeScript frontend, and container deployment
class.

## Repairs

- The server rate limiter now uses the first, factory-ingress-supplied
  `X-Forwarded-For` IP as its trusted client identity. It falls back to the
  socket peer only for direct local runs. Each identity receives 40 requests
  per second; excess requests return `429` and `Retry-After: 1`.
- Added a private GitHub source flow. A deployment owner provides
  `GIT_CREDENTIAL_<REFERENCE>` and pins it with
  `GIT_CREDENTIAL_<REFERENCE>_REPOSITORY=https://github.com/owner/repository`.
  An authenticated workspace binds that uppercase reference to the same
  repository through `POST /api/git-credentials`, then publishes with
  `git_credential_ref`. The token is used only for GitHub commit/content reads,
  is never stored or returned, cannot be rebound to a different repository,
  and one reference cannot be claimed by another workspace.
- The registry publish form documents and performs that binding flow. Existing
  signed package payloads remain byte-compatible: the credential reference is
  workspace audit metadata, not package content.
- Added claim-backed regressions for independent per-client rate allowances,
  one-client exhaustion, private-source authentication, no token in SQLite,
  cross-workspace rejection, and cross-repository rejection.

## Verification

Clean install and quality gates passed:

```sh
npm ci
npm audit
npm test
npm run typecheck
npm run build
npx playwright test
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

Results: npm audit reported 0 vulnerabilities; Playwright passed 19/19 across
desktop and 390px mobile; Rust integration tests passed 11/11. All 15 exact
claim commands in `.factory/claims.json` were then run independently and
passed.

The release binary was started with a fresh SQLite path and only `PORT`,
`DATABASE_PATH`, and a local build identity. `/health` returned the expected
identity. `/opt/fleet/lib/verify-url.sh` passed locally in 600 ms with no
console errors, a title, `lang=en`, one h1, main landmark, and complete image
alt coverage. Header checks confirmed CSP, HSTS, `nosniff`, Referrer-Policy,
Permissions-Policy, and immutable hashed-asset caching. Evidence is in
`.factory/evidence/repair-5/verify-url/`.

Playwright's installed Chromium accessibility suite passed as part of the
19-test browser run. The standalone `@axe-core/cli` was also attempted against
the same local server but ChromeDriver exited before a session was created in
this worker; it did not produce findings. No Docker-compatible engine is
available in this worker, so the image build was not repeated; Docker contract
tests pass and the release binary was exercised directly.

## Deploy

Deploy this commit with:

```sh
/opt/fleet/lib/deploy-container.sh team-agent-skills /work/repo Dockerfile 8080
```

The deployment needs no Git credential variables for public repositories. To
enable a private repository, configure a least-privilege GitHub credential and
its canonical repository boundary as described above; do not add token values
to the workspace, package, or repository.

## Known gap

The standalone axe CLI could not start its ChromeDriver session in this
container; the Playwright axe coverage is passing. The factory deployment
script remains the authoritative container build and live verification step.
