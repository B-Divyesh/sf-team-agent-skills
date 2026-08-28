# Team Skills Registry

Publish reviewed agent skill packages and release exact versions to assigned repositories.

Team Skills Registry is for engineering leads who maintain instructions across
Codex, Claude Code, Cursor, and similar coding agents. Each immutable version
contains instructions, adapter content, a Git source commit, secret reference
names, repository assignments, an approval record, and a package digest.

## Run locally

Requirements: Node 22+, current stable Rust, and Chromium for browser checks.

```sh
npm ci
npm run build
cargo run
```

Open `http://localhost:8080`. The service creates `data/registry.db` and
needs no environment variables. `PORT` changes the listener and
`DATABASE_PATH` changes the SQLite file.

Open `/registry`, then create a private workspace. The browser receives a
random owner key and a separate one-time reviewer key. The server stores only
their SHA-256 hashes. Save the owner key in a password manager. Give the
reviewer key to the person who approves releases.

## Try the sandbox

Open `/demo` or `/?demo=1`. It loads three complete skill packages, approval
records, repository assignments, and receipts. Demo storage uses
`demo:team-agent-skills:v2`. It never calls the registry API. Reset reseeds
the sample; leaving the demo deletes its storage.

## Test and build

```sh
npm ci
npm test
npm run typecheck
npm run build
npx playwright test
cargo test --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
npm audit
```

The claim checks and their exact commands are in `.factory/claims.json`.

## API workflow

Create a workspace with `POST /api/session`. Send its returned key as
`Authorization: Bearer <key>` to every registry endpoint. Publish a complete
version at `POST /api/skills`, record review at
`POST /api/skills/:id/approve`, and release it with
`PATCH /api/skills/:id/ring`. Assigned agents fetch a reviewed pilot or full
release from `GET /api/repositories/:repository/install/:id`.

## Deploy

The multi-stage Dockerfile builds the Vite frontend and Rust service from the
lockfiles. It uses the current stable Rust image, runs as a non-root user,
listens on `PORT` (default `8080`), and reports `BUILD_SHA` at `/health`.
Mount `/data` for persistence.

```sh
docker build --build-arg BUILD_SHA=local -t team-agent-skills .
docker run --rm -p 8080:8080 -v team-skills-data:/data team-agent-skills
```

Read the in-product [privacy policy](/privacy) and [terms](/terms).

## License

MIT. See [LICENSE](LICENSE).
