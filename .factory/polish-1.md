# Perfection-loop polish 1

Repair code was deployed from `88350332d03467b1523bb7b33aa98605762a3fb6` to
<https://team-agent-skills.sociobot.in>. The live cold-browser audit is
[`evidence/polish-1/live/browser-audit.json`](evidence/polish-1/live/browser-audit.json).
Screenshots are the [live home at 390 px](evidence/polish-1/live/screenshot-mobile.png)
and [live demo at 390 px](evidence/polish-1/live/demo-mobile.png).

| Finding | Change made | Evidence | Live check |
| --- | --- | --- | --- |
| F-1-1 | Added the 3 px coral focus ring to every textarea. | Playwright `publish textareas use the designed focus ring` at 390 and 1366 px. | Audit measured every publish textarea at ≥3 px on `/demo`; demo screenshot above. |
| F-1-2 | Registered `demo-sample-content` and assert exactly three complete packages with approvals and source details. | Clean-clone `@claim:demo-sample-content`; [`clean-claims.log`](evidence/polish-1/clean-claims.log). | Cold `/demo` contained exactly three packages and their review records. |
| F-1-3 | Registered `mit-license-build`; its command completes both production builds and checks the MIT license and locked container stages. | Clean-clone `@claim:mit-license-build`. | Landing now says only that the self-hosted registry is MIT licensed. |
| F-1-4 | Removed the speculative future billing-integration sentence. | `@claim:managed-plan-status` and copy audit. | Cold `/` has no checkout and no future-capability sentence. |
| F-1-5 | Registered and tagged demo teardown. | Clean-clone `@claim:demo-teardown`. | **Start for real** removes the demo key and opens `/registry`. |
| F-1-6 | Registered `/api/trust` and verify the SHA-256 relationship between its key and fingerprint. | Clean-clone `claim_trust_endpoint`. | Live `/health` returns the deployed signer fingerprint. |
| F-1-7 | Registered a selected-port release-binary test. | Clean-clone `claim_port_configuration`. | Deployed container serves on the configured port through HTTPS. |
| F-1-8 | Registered a temporary explicit database-path and signing-state test. | Clean-clone `claim_database_path_configuration`. | Live container starts with the work-order data path contract. |
| F-1-9 | Removed unprovable proxy-overwrite and socket wording; retained only the tested per-client first-hop limit. | Clean-clone `claim_trusted_client_rate_limit`. | Live service sends security headers; no unsupported ingress claim remains. |
| F-1-10 | Replaced the false blanket assurance with a link to the complete claim registry; the inventory test enforces one tag per claim. | `npm test`, `every listed claim has exactly one matching source tag and command`. | Live and README claims match the 24-entry registry. |
| F-1-11 | Registered persistence across a real binary restart using one data directory. | Clean-clone `claim_persistent_data_and_signing_key`. | ACR built the root Dockerfile; `/health` reports build `8835033`. |
| F-1-12 | Added description, canonical, Open Graph, Twitter, theme, favicon, and apple icon metadata to the static 404. | Rust `unknown_route_returns_complete_static_404`; axe 404 scan. | Unknown URL returns HTTP 404 with complete metadata and zero serious/critical axe findings. |
| F-1-13 | Added repository and agent selectors, native `AGENTS.md`/`CLAUDE.md`/`GEMINI.md` output, provenance, and a copyable install command. Signed JSON remains available. | Clean-clone `@claim:repo-native-install`; Rust `claim_repo_native_install`. | Cold demo downloaded `CLAUDE.md` for `web-console` with adapter text and digest. |
| F-1-14 | Added a separate recovery key, hash-only storage, downloadable recovery kit, and an endpoint that rotates and invalidates the lost workspace owner key. | Clean-clone `claim_workspace_recovery`; Playwright recovery-kit test. | `/registry` shows recovery controls; live invalid-key call reached the endpoint and returned 401 without mutation. |
| F-1-15 | New navigation now focuses a visible h1 at scroll top; Back/Forward restores prior scroll and focused link with manual history restoration. | Playwright `footer navigation reveals the new heading and Back restores scroll and focus`. | Cold 390 px live audit passed the footer → Privacy → Back sequence. |
| F-1-16 | Made both social image fields absolute and added `twitter:image`. | Playwright `route metadata follows the current page`. | All six live routes expose the absolute 1200×630 image in both fields. |
| F-1-17 | Rewrote the README opening to “Publish reviewed agent skill packages to assigned repositories.” | `.factory/copy-audit.md`. | No “safely” copy remains in the live product. |
| F-1-18 | Replaced internal adapter, pilot-membership, and secret-reference language with named package contents. | README copy audit and terminology scan. | Live product consistently calls the artifact a skill package. |
| F-1-19 | Explained the signed Git identifier as the identifier for the exact source file. | README copy audit. | Live package detail shows verified commit, digest, and signature separately. |
| F-1-20 | Rewrote the component-centric sentence as “The service loads skill packages from GitHub repository URLs.” | README copy audit. | Publish form names the Git source URL directly. |
| F-1-21 | Split the 23-word credential sentence into three short boundary statements. | README copy audit; no sentence over 22 words. | Privacy route explains hash-only key handling in plain words. |
| F-1-22 | Explained the local Git test server by the responses it must return. | README copy audit. | No unexplained “compatible fixture” wording remains. |
| F-1-23 | Explained that installers compare the trust key and fingerprint to verify the signer. | README copy audit; `claim_trust_endpoint`. | Live trust relationship is cryptographically checked by the suite. |
| F-1-24 | Removed dense ingress/socket jargon and kept the tested client-limit statement. | README copy audit; `claim_trusted_client_rate_limit`. | No dense ingress sentence remains on the product or README. |
| F-1-25 | Rewrote private-source setup as a concrete repository and credential-name action. | README copy audit; `claim_private_git_source`. | Publish form describes the optional uppercase credential reference. |
| F-1-26 | Standardized “install credential” everywhere. | Terminology scan and README copy audit. | Live install section uses the same term and scoped workflow. |
| F-1-27 | Standardized “workspace owner key” across landing, registry, errors, legal pages, README, and claims. | `rg` terminology scan; `claim_private_workspace`. | Live `/`, `/registry`, `/privacy`, and `/terms` use the same term. |
| F-1-28 | Renamed release controls with verbs: move, send, and release to the named scope. | Playwright release-state assertions at desktop and 390 px. | Live demo exposes all four explicit action labels without horizontal overflow. |
| F-1-29 | Removed the visible provenance figcaption while keeping useful alt text and provenance in the design record. | Accessibility test and `.factory/design.md`. | Live first screen has useful alt text and no provenance caption. |

## Complete verification

- Every one of 24 claim commands passed from fresh clone
  `/tmp/team-agent-skills-final-claims-NmiL6a` at `986dabf`.
- `npm test`, `npm run typecheck`, `npm run build`, and 26 Playwright tests pass.
- `cargo fmt --all -- --check`, strict Clippy, 16 integration tests, and the
  locked release build pass.
- Playwright axe found zero serious or critical issues on every route and the
  real 404 at desktop/mobile coverage.
- Local mobile Lighthouse: performance 100, accessibility 100, LCP 1.4 s,
  CLS 0. Initial JS is 35.92 KB raw (11.10 KB gzip); CSS is 16.38 KB raw.
- A 100-request concurrent health smoke returned 100 HTTP 200 responses in
  376 ms; evidence is
  [`load-100-health.txt`](evidence/polish-1/local/load-100-health.txt).

No finding remains open.
