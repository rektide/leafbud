---
type: Experiment
title: "Passive Leaflet realtime connection probe"
description: "Isolated deployment bootstrap, receive-only presence and maintained Phoenix lifecycle probes; no production watch implementation or document writes."
resource: /experiments/15-live-connection/README.md
status: completed-probe
generated:
  at: 2026-11-09
  by: model:openai/gpt-6.1-sol#high
sources:
  - resource: "opencode://ses_e41f53633ffeI4wwnXfMRs1LeT"
    title: "Independent passive deployment and Rust transport-library probe, synthetic lifecycle checks and watch integration handoff"
  - resource: /design/keyed-build-path0.gpt6a.md
    title: "Accepted connected watch boundary and exploratory presence commission"
---

# Passive Leaflet realtime connection probe

Findings and integration contract: [live connection findings](live-connection0.gpt61s.md).
This standalone Cargo workspace does not change the production workspace, lock,
source or shared design. `~/l` is read only. No deployed IDs, responses, keys or
cookies belong in this directory.

## Run

From the Leafbud repository root:

```sh
# Writes private config under ignored .test-agent/live-connection only.
python3 experiments/15-live-connection/bootstrap.py --workspace ~/l

# Maintained Phoenix client: two passive connected epochs and explicit reconnect.
CARGO_TARGET_DIR=.test-agent/live-connection/target cargo run \
  --manifest-path experiments/15-live-connection/Cargo.toml --bin managed

# Independent minimal protocol v1 control: two fresh passive socket sessions.
CARGO_TARGET_DIR=.test-agent/live-connection/target cargo run \
  --manifest-path experiments/15-live-connection/Cargo.toml --bin leafbud-live-probe

# Offline contracts and real loopback socket lifecycle fixtures.
CARGO_TARGET_DIR=.test-agent/live-connection/target cargo nextest run \
  --manifest-path experiments/15-live-connection/Cargo.toml
CARGO_TARGET_DIR=.test-agent/live-connection/target cargo clippy \
  --manifest-path experiments/15-live-connection/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path experiments/15-live-connection/Cargo.toml --check
```

The bootstrap helper scans a bounded set of same-origin public browser assets,
selects an unambiguous matching Supabase project URL and `anon` JWT, then validates
one existing local binding using `get_leaflet_data`. It uses
`LEAFLET_AUTH_TOKEN`, otherwise the existing XDG `leafbud/auth-token` file, only
for that same-origin HTTP read. This is an experiment-specific locator reader,
not the production inventory/parser. Ambiguous discovery fails instead of guessing.

Both socket probes send only joins, heartbeats, leaves and socket close. Neither
sends `track`, `hello`, cursor awareness, content updates or document mutations.
Joining still consumes a connection/subscription; receive-only is not zero-cost.
Output is limited to categories and aggregate counts. The probes are bounded,
not production daemon/reconnect/adoption implementations.

[`fixtures/events.json`](fixtures/events.json) is entirely synthetic. Tests
exercise passive presence readiness/ref removal, dirty coalescing, real TCP loss,
heartbeat timeout, rejected/timed-out joins, `phx_error`, `phx_close`, and correct
`phx_leave`. A fake server is not deployed delivery proof.
