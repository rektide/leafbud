# Direct Rust test of Vercel's mdxjs fork

Isolated experiment; no Leaflet access or production dependency changes.
The package is **`mdxjs` 1.0.4 from Git**, not a registry crate named
`mdxjs-rs-turbopack`. Exact source:
`vercel-labs/mdxjs-rs-turbopack@700e192fbdbad5874208addfbf421dcf1eca7a20`.

Initial smoke command (network only for dependency setup):

```sh
cd experiments/12-mdxjs-turbopack-identity
cargo run --locked --offline
```

It calls the fork's public `mdast_util_from_mdx` directly, without Turbopack.
Modern Serde 1.0.229 / serde_json 1.0.154 build without patches or old pins.
The full behavioral corpus and report follow in this same directory.
