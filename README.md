# Team Skills Registry

Publish reviewed agent skills and release them safely across repositories.

Team Skills Registry is for engineering leads who need one checked instruction
set for Codex, Claude Code, Cursor, and similar coding agents. A skill packet
holds a version, adapters, secret references, and a release ring. An execution
receipt records the released version, repository, agent, and time.

## Run locally

Requirements: Node 22+, Rust 1.88+, and a Chromium browser for claim checks.

```sh
npm ci
npm run build
cargo run
```

Open `http://localhost:8080`. The service creates `data/registry.db` when it
starts. It needs no environment variables. `PORT` changes the listen port and
`DATABASE_PATH` changes the SQLite location.

For frontend-only work, run `npm run dev`. Vite proxies API requests to port
8080.

## Try the sandbox

Open `/demo` or `/?demo=1`. It loads three sample skill packets and three
execution receipts. Demo storage uses the separate
`demo:team-agent-skills:v1` localStorage key. Resetting the demo deletes and
reseeds that key. The demo never calls the registry API.

## Test and build

```sh
npm test
npm run typecheck
npx playwright test
cargo test
npm run build
```

`npm run build` is the frontend build command and writes the deployable assets
to `dist/` with `index.html` at its root. The claim checks are named in
`.factory/claims.json` and run from a fresh browser context against `/demo`.

## Deploy

The supplied multi-stage Dockerfile builds the Vite frontend and Rust service
with the committed Node and Cargo lockfiles. Its Rust stage is 1.88 because the
locked ICU dependency requires that compiler floor.
It listens on `PORT` (default `8080`) and serves `GET /health` with the build
SHA. Mount `/data` if the registry should survive container replacement.

```sh
docker build --build-arg BUILD_SHA=local -t team-agent-skills .
docker run --rm -p 8080:8080 -v team-skills-data:/data team-agent-skills
```

## Plans and privacy

The team plan is $149 per team/month for private registries, approval rings,
and audit history. Checkout and license checks use the Sociobot billing service.
The app stores a returned license in browser storage. It does not put secret
values in skill packets; use names such as `GITHUB_TOKEN` instead.

Read the in-product [privacy policy](/privacy) and [terms](/terms). See
`.factory/design.md` for the visual system, `.factory/demo.md` for sandbox
details, and `.factory/handoff.md` for verification notes.

## License

MIT. See [LICENSE](LICENSE).
