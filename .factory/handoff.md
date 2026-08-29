# Review handoff — Team Skills Registry

## Outcome

Adversarial first-read review 1 is complete. Verdict: **FAIL** with 29 findings,
including 14 blocking findings. The complete evidence, copy inventory, claim
results, history audit, and required fixes are in
[`review-1.md`](review-1.md).

No product code was changed.

## What was checked

- Cold live first read at 390×844 and 1440×900 before scrolling.
- One-click demo entry, realistic seeded state, banner, reset, offline state,
  same-origin request log, real-key isolation, and demo teardown.
- Every landing and README sentence with word counts and plain-language flags.
- Every exact command in `.factory/claims.json`, run separately from a clean
  clone: 15/15 passed.
- Direct live routes, titles, headings, metadata, canonical links, 404, link
  crawl, header/footer, History API behavior, focus, and scroll restoration.
- Live axe-core serious/critical scan on all routes and the 404: zero findings.
- All historical `.factory/handoff.md` revisions. No earlier review or polish
  file exists in reachable repository history.
- Product-specific visual identity and missed-leverage review.

## Reproduce

```sh
npm ci
npm test
npm run typecheck
npm run build
npx playwright test
```

Then run every `test` value in `.factory/claims.json` separately from a clean
clone. For the deployed smoke test:

```sh
VERIFY_NODE_MODULES=/work/repo/node_modules \
  /opt/fleet/lib/verify-url.sh \
  https://team-agent-skills.sociobot.in \
  /tmp/team-agent-skills-verify
```

## What remains

Do not accept this round. Highest-priority repairs are the unresolved textarea
focus treatment, complete claim inventory, full metadata on the real 404,
repo-native adapter delivery, and workspace-key recovery. The next reviewer
must rerun the full checklist rather than checking only these findings.
