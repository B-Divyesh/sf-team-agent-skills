# Handoff — Team Skills Registry v1

## Delivered

- A Rust/axum + SQLite registry on port 8080 with skill-packet creation,
  release-ring changes, execution receipts, health reporting, validation,
  security headers, and per-client request limiting (40 requests/second;
  429 responses include `Retry-After`).
- A Vite TypeScript interface with a paper-cut release-desk visual system.
  The landing page explains the job in plain words, then leads directly to a
  working registry.
- A sandboxed `/demo` and `?demo=1` workspace. It is seeded with three
  realistic skill packets and receipts. Its browser storage key is
  `demo:team-agent-skills:v1`; it does not call the real registry API.
- Release rings (draft, review, pilot, all repositories), agent adapters,
  secret references, receipt history, empty states, offline/error feedback,
  mobile layout, keyboard skip link, `/privacy`, `/terms`, and a designed 404.
- Sociobot checkout link plus returned-license storage and restore/verify flow.
- Original image asset generated with the factory image model. Source PNG,
  prompt sidecar, and optimized 54 KB WebP are under `assets/src/`.

## Verification

Ran successfully:

```sh
npm test                 # 2 unit tests passed
npx playwright test      # 4 browser tests passed
npm run build            # dist/ created; JS 5.86 KB gzip, CSS 3.52 KB gzip
cargo test               # 1 integration test passed
```

Claim checks in `.factory/claims.json` passed from `/demo`:

- demo reset restores its isolated sample workspace;
- recording a run places the selected version and repository in a receipt;
- demo browser requests stay on the product origin and the loaded demo remains
  usable after the context goes offline.

Runtime smoke test on port 8181 passed: `/health` returned the build SHA;
40 requests returned 200 and the next 5 returned 429. The response included
`Retry-After: 1`.

`verify-url.sh` against the Rust server reported 640 ms load time, no console
errors, title and language present, exactly one h1, a main landmark, and no
images missing alt text. The landing page was also inspected at 390 px.

Lighthouse was not installed in this container. The produced asset sizes are
well below the first-load budgets. The standalone axe CLI could not launch its
Selenium Chrome session in this image; semantic and keyboard browser checks are
included in Playwright instead.

## Run and deploy

Use `npm install && npm run build && cargo run`. The Dockerfile performs the
same production build and starts without required environment variables.
`PORT` defaults to 8080. Mount `/data` to persist SQLite across container
replacement.

## Known gaps / next steps

- This first container uses one workspace database and no identity provider.
  Add team authentication and repository-scoped authorization before treating a
  deployment as a multi-tenant private registry.
- Git source synchronization and vendor-specific adapter file generation are
  intentionally not automated in v1; packets record their reviewed metadata and
  release state, but do not fetch from Git providers.
- Docker could not be built here because the container image has no Docker
  daemon or client. The Dockerfile follows the required multi-stage, non-root
  contract and should be built by the factory.
