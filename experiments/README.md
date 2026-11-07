# experiments/

Protocol probes and evidence for springleaf. Unlike the `.test-agent/`
convention, this directory is **tracked**: protocol evidence against
leaflet.pub is load-bearing for the build, and each probe's REPORT.md is the
receipt for later crate design.

Rules:

1. **No credentials.** Read tokens at runtime from
   `~/.config/springleaf/auth-token` (or env); never print, log, or commit
   token values.
2. **Synthetic fixtures.** Committed HTML/JSON fixtures use fabricated ids,
   handles, and content mimicking the real shape. Real responses get
   redacted into synthetic examples inside REPORT.md.
3. **Read-only against production.** GETs and documented read RPCs only —
   no putRecord, no push mutations, no state changes on leaflet.pub or any
   PDS without explicit operator go-ahead.
4. **Promote findings.** Load-bearing conclusions graduate into `design/`
   when a build stage consumes them.
5. One subdir per topic: `NN-topic/REPORT.md` + scripts + fixtures.
