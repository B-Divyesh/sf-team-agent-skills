# Team Skills Registry

Publish reviewed agent skills and release them safely across repositories.

Team Skills Registry is for engineering leads who maintain instructions for
different coding agents. Each version contains its instructions, agent-specific
adapters, verified GitHub commit, repository assignments, and secret reference
names. The service signs the complete package before review.

## Run locally

Requirements: Node 22+, current stable Rust, and Chromium for browser checks.

```sh
npm ci
npm run build
cargo run
```

Open `http://localhost:8080`. `PORT` changes the listener. `DATABASE_PATH`
changes the SQLite location.

Open `/registry` and create a workspace. Save its owner key in a password
manager. Publishing creates a separate reviewer key for that one package.
The reviewer opens `/review` with only that key. Approval consumes the key.

The source verifier accepts public GitHub repository URLs. It confirms the
exact commit through GitHub before signing. Set `GIT_VERIFY_API_BASE` only when
running a compatible local verifier fixture.

## Try the sandbox

Open `/demo` or `/?demo=1`. It loads three complete packages, approval records,
repository assignments, and receipts. Demo storage uses
`demo:team-agent-skills:v2`. Reset restores the sample. Leaving deletes it.

## Release rules

- Pilot and full release require a recorded review.
- A receipt requires an installed release, assigned repository, and named agent.
- Downloads contain the signed payload, digest, Ed25519 signature, and signer key.
- `/api/trust` exposes the signer key and fingerprint for consumer pinning.
- Workspace identifiers are isolated, so teams may use the same package id.

Every statement above has an exact test in [.factory/claims.json](.factory/claims.json).

## Test and build

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

## API workflow

Create a workspace with `POST /api/session`. Send its owner key as
`Authorization: Bearer <key>` to registry endpoints. Publish at
`POST /api/skills`. The response contains the package's one-time reviewer key.

A reviewer sends that key to `GET /api/review`, then approves with
`POST /api/review/approve`. The owner releases it with
`PATCH /api/skills/:id/ring`. Assigned agents fetch it from
`GET /api/repositories/:repository/install/:id`.

## Managed plan

The researched managed plan is $149 per team each month. Billing is not active
in this release, so the product does not collect payment or claim a purchasable
subscription. A future managed release must use the Sociobot billing API. The
self-hosted MIT build remains available.

## Deploy

Build the root Dockerfile and mount `/data` for the database and signing
identity.

```sh
docker build --build-arg BUILD_SHA=local -t team-agent-skills .
docker run --rm -p 8080:8080 -v team-skills-data:/data team-agent-skills
```

Read the in-product [privacy policy](/privacy) and [terms](/terms).

## License

MIT. See [LICENSE](LICENSE).
