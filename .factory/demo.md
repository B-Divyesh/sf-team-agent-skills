# Demo sandbox

Open `/demo` or `/?demo=1` for the isolated registry. It seeds three realistic
skill packets and three execution receipts: Secure commit, Migration review, and
Incident note.

Demo mode stores data only at `localStorage['demo:team-agent-skills:v1']`.
It never reads or writes the real registry API. **Reset demo** deletes that key
and reseeds it. **Start for real** takes the visitor to `/registry`, which uses
the SQLite-backed API.

Run `npx playwright test` from a fresh clone to verify demo claims.
