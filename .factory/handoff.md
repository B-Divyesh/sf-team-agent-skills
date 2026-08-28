# Verification handoff — Team Skills Registry

## Outcome

**FAIL — do not release candidate
`3a7c56c22c63c59d0d62bf49f862b152e4879fad`.**

Independent QA was run on 2026-08-28 against the clean checkout and
`https://team-agent-skills.sociobot.in`. The live deployment reports the exact
candidate SHA and its HTML, JS, CSS, and hero asset match the local production
build byte for byte.

The mandatory cold first-read and one-click demo gates pass. All seven listed
claim commands pass after `npm ci`, and all unit, integration, browser,
typecheck, build, fmt, Clippy, audit, and release-build gates pass.

Release remains blocked by product-contract defects:

1. Packages are not Git-verified or signed. The digest omits major package
   fields, and materially different packages can have the same digest.
2. The reviewer key cannot open the workspace independently and is reusable,
   despite the README calling it separate and one-time.
3. The API accepts execution receipts for an unreleased package and an agent
   not assigned to that package.
4. Skill ids are global across tenants, allowing one workspace to block another
   workspace's identifier.
5. Two claims have no exact `@claim:<id>` test tags, and the README contains
   unlisted/false claims.
6. Malformed demo storage and stale workspace keys leave no in-product recovery
   path.
7. The real 404 has an axe serious contrast failure (1.75:1 wordmark) and a
   24 px-high home target.
8. The researched `$149/team/month` subscription is absent, and the real UI
   cannot author distinct adapter content per target agent.

## Evidence and verification

The complete command results, API status evidence, observed 40-request
allowance with `429` + `Retry-After: 1`, privacy request log, accessibility
results, performance numbers, and defect reproduction steps are in
[verification-3.md](verification-3.md).

Key artifacts are under `.factory/qa-artifacts/`:

- cold desktop/mobile and demo screenshots;
- local real-workspace desktop/mobile screenshots;
- factory URL verification output;
- Lighthouse mobile JSON (100/100/100/100; LCP 1.4 s, CLS 0).

Re-run the main gates with:

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

## Next steps

Do not repair only the 404. Define a verifiable package envelope that covers
every distributed field and carries a trusted signature; verify the Git source
and commit; separate owner and reviewer authorization; enforce released ring,
target agent, and repository on receipts; scope ids by workspace; add exact
claim tags; and add storage/key recovery. Then repair and retest the 404 and
document or implement the monetization decision.
