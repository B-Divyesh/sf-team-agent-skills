# Demo sandbox

Open `/demo` or `/?demo=1` for the isolated registry. It seeds three realistic
skill packages and three execution receipts: Secure commit, Migration review, and
Incident note. Downloads use the same `team-agent-skill/v2` envelope as real
installs.

Demo mode stores data only at `localStorage['demo:team-agent-skills:v2']`.
It never reads or writes the real registry API. **Reset demo** deletes that key
and reseeds it. **Start for real** deletes that demo key, then opens the private
workspace start screen. Real workspaces use the SQLite-backed API.

Run `npx playwright test` from a fresh clone to verify demo claims.
