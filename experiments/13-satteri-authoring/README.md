# Satteri authoring → Leaflet, offline

An isolated compiler experiment, not a production implementation. See
[REPORT.md](REPORT.md) for the verdict, exact identities and boundaries.

```sh
cd experiments/13-satteri-authoring
cargo fetch --locked
cargo run --locked --offline -- check fixtures/garden.mdx
cargo run --locked --offline -- demo fixtures/garden.mdx
cargo run --locked --offline -- check fixtures/rejected.mdx # expected exit 2, no plan
cargo nextest run --locked --offline --no-fail-fast
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

`check` reads one canonical MDX file and prints transient typed IR, planned native
mutation arguments and decoded fresh Yrs text. It never writes the source, a
companion JSON, a saved base or a CRDT state. `demo` simulates interrupted detached
staging, verified entry replacement, repeat and a source edit, retaining a synthetic
existing binding. No credentials or Leaflet transport exist in this program.

The native `<Key id="name" />` marker names the immediate next block in its own
root/list-item/quote/Page scope. Blank lines after it are optional; put a blank
line before it after earlier content. Paired Key wrappers are unsupported. Keys
are internal author names, not HTML anchors, remote UUIDs or Yjs character IDs.

This deliberately bounded profile supports headings, paragraphs, normal marks,
links, hard breaks, simple lists, one-paragraph quotes, fenced code, rules,
forward named linear Pages/PageLinks, exact Run data and keyed unpublished Polls.
Unsupported content fails before a usable document/plan exists. MDX is not
unchanged CommonMark: use fenced multiline code examples.

The additional independent Yjs/ProseMirror check uses the same pinned versions
as Leaflet's inspected lockfile; it also runs without application/network access:

```sh
pnpm install --frozen-lockfile
cargo build --locked --offline
pnpm test
```

Dependencies have exact public registry/Git pins and isolated lockfiles. The
archive checkout is evidence, not a machine-specific build dependency. Tested on
Linux with Rust 1.101 nightly and Node 26; minimum/cross-target builds are unproved.
