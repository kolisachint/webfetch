# Install & Development

## Install the CLI

Grab a prebuilt binary from the [Releases](../../releases) page, or build from
source:

```bash
cargo build --release --bin webtools
# binary at target/release/webtools
```

## Use as a library

The libraries are published on crates.io as `webtools-fetch` and
`webtools-search` (with shared primitives in `webtools-core`). Their library
import paths remain `webfetch` and `websearch`:

```toml
[dependencies]
webfetch = { package = "webtools-fetch", version = "0.1" }
websearch = { package = "webtools-search", version = "0.1" }
```

Or pull straight from git:

```toml
[dependencies]
webfetch = { package = "webtools-fetch", git = "https://github.com/kolisachint/webtools" }
websearch = { package = "webtools-search", git = "https://github.com/kolisachint/webtools" }
```

```rust
use webfetch::types::{ContentType, FetchOptions};
use websearch::types::SearchOptions;

// ── Fetch: convert HTML without network I/O ──────────────────────────
let opts = FetchOptions {
    content_type: ContentType::Text,
    ..Default::default()
};
let result = webfetch::convert_html(html, "https://example.com/page", &opts);

// Access the compact content and recover URLs:
println!("{}", result.content);          // text with [N] markers
for r in &result.references {
    println!("[{}] {}", r.index, r.url); // recover full URLs
}
println!("~{} tokens", result.token_estimate);

// ── Fetch: network request with retry/backoff ────────────────────────
let result = webfetch::fetch_and_convert(FetchOptions {
    url: "https://docs.example.com/api".into(),
    ..opts
}).await?;

// ── Search: zero-infrastructure DuckDuckGo Lite ─────────────────────
let search_opts = SearchOptions {
    query: "rust async runtime".into(),
    max_results: Some(5),
    ..Default::default()
};
let output = websearch::run_search(search_opts).await?;
for hit in &output.results {
    println!("{} [{}]", hit.title, hit.ref_index);
}
```

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cargo run --release --example latency   # offline latency benchmark
```

Before committing code changes, run all three checks and fix every error,
warning, and info:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Releasing

Releases are label-driven. Open a PR with the `/pr <patch|minor|major>` command
(see `.agents/commands/pr.md`); merging a PR labeled `cargo:<bump>` triggers
`.github/workflows/merge-release.yml`, which bumps every crate version, tags
`v<version>`, and pushes. The tag then triggers
`.github/workflows/release.yml`, which publishes the libraries to crates.io and
attaches Linux + macOS binaries to the GitHub release. See
[`ci/README.md`](../ci/README.md) for details.

Manual fallback (only if needed):

```bash
git tag v0.1.0
git push origin v0.1.0
```
