# Team Skills Registry

Publish reviewed agent skill packages to assigned repositories.

Team Skills Registry is for engineering leads who maintain instructions for
different coding agents. Each version comes from one JSON file at a verified
GitHub commit. It includes shared instructions, instructions for each agent,
assigned repositories, pilot repositories, and credential reference names.
Before review, the service signs the package and Git's identifier for the exact
source file.

## Run locally

Requirements: Node 22+, current stable Rust, and Chromium for browser checks.

```sh
npm ci
npm run build
cargo run
```

Open `http://localhost:8080`. `PORT` changes the service listener.
`DATABASE_PATH` selects the SQLite and signing-state directory.

Open `/registry` and create a workspace. Save its workspace owner key and
recovery key in a password manager. The recovery key replaces a lost workspace
owner key. Publishing creates a separate reviewer key for that one package.
The reviewer opens `/review` with only that key. Approval consumes the key.

The service loads skill packages from GitHub repository URLs. Public
repositories need no extra configuration. For a private repository, the
deployment owner configures a named credential and its allowed repository.
The workspace then binds that credential name to the same repository.
The server never stores or returns the credential value. Another workspace
cannot use its reference. The reference cannot read another repository.
The service confirms the commit and reads the named JSON file there. It then
signs the returned file identifier. Browser fields cannot replace committed
instructions for the package or agent. Set `GIT_VERIFY_API_BASE` only for a
local test server that returns the GitHub commit and file responses used here.

## Try the sandbox

Open [`/demo`](https://team-agent-skills.sociobot.in/demo) or `/?demo=1`.
It loads three complete packages, approval records, repository assignments,
and receipts. Demo storage uses `demo:team-agent-skills:v2`. Reset restores the
sample. Leaving the demo deletes its sample workspace.

## Release rules

- Pilot and full release require a recorded review.
- Pilot reaches only `pilot_repositories`; full release reaches all `repositories`.
- A receipt requires an installed release, assigned repository, and named agent.
- Downloads include repository-native instructions and the signed JSON record.
- An owner issues one install credential for each package, repository, and agent.
- That credential cannot list, publish, review, or release packages.
- `/api/trust` returns the signer key and its matching SHA-256 fingerprint.
- Workspace identifiers are isolated, so teams may use the same package id.
- Each trusted client IP has its own 40-request limit.

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

Every product claim and its exact command are listed in
[`.factory/claims.json`](.factory/claims.json).

## API workflow

Create a workspace with `POST /api/session`. Send its workspace owner key as
`Authorization: Bearer <key>` to registry endpoints. Publish with
`POST /api/skills` and the source fields shown in
[`examples/skill-package.json`](examples/skill-package.json).

Before publishing from a private repository, bind its configured credential
name with `POST /api/git-credentials`. The publish response contains the
package's one-time reviewer key. A reviewer uses it at `GET /api/review` and
`POST /api/review/approve`.

The owner changes the release with `PATCH /api/skills/:id/ring`. The owner then
issues an install credential at `POST /api/skills/:id/install-credentials`.
The assigned agent uses that credential at
`GET /api/repositories/:repository/agents/:agent/install/:id`. The response
includes `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, or a general instruction file.

## Managed plan

The researched managed plan is $149 per team each month. Billing is not active
in this release, so the product does not collect payment. A future managed
release must use the Sociobot billing API. The self-hosted build is available
under the MIT license.

## Deploy

Build the root Dockerfile and mount `/data`. The mounted directory preserves
the database and signing identity across restarts.

```sh
docker build --build-arg BUILD_SHA=local -t team-agent-skills .
docker run --rm -p 8080:8080 -v team-skills-data:/data team-agent-skills
```

Read the in-product [privacy policy](/privacy) and [terms](/terms).

## License

MIT. See [LICENSE](LICENSE).
