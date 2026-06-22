# CI / Release workflows

✅ The GitHub Actions workflows are active in `.github/workflows/`
(`ci.yml`, `merge-release.yml`, `release.yml`). This file documents what each
one does. There is no manual activation step.

## Workflow details

### `ci.yml`

Runs on pushes to `main` and on PRs:
- `cargo fmt --all --check` — formatting
- `cargo clippy --workspace --all-targets -- -D warnings` — lints
- `cargo test --workspace` — all tests

### `release.yml`

Runs on `v*` tags (e.g. `v0.1.0`):
- Publishes the libraries to crates.io in dependency order
  (`webfetch-core` → `webfetch` → `websearch`), skipping any version already
  on the index so a partial run can be retried (needs the `CRATES_IO_TOKEN`
  secret)
- Builds `webtools` for Linux (`x86_64-unknown-linux-gnu`) and macOS (`aarch64-apple-darwin`)
- Packages each as a `.tar.gz` and attaches to the GitHub release
- Generates release notes automatically

### `merge-release.yml`

Runs when a PR with a `cargo:patch`, `cargo:minor`, or `cargo:major` label is merged:
- Reads the current version from `Cargo.toml`
- Bumps the version based on the label
- Updates `Cargo.lock`
- Commits the version change
- Creates a `v*` git tag
- Pushes to `main`

This triggers `release.yml` which builds and publishes binaries.

## PR-based release flow

The recommended release process uses the `/pr` command (see `.agents/commands/pr.md`):

1. **Agent runs `/pr patch`** (or `minor`/`major`) → Creates PR with `cargo:<bump>` label
2. **PR gets merged** → Triggers `merge-release.yml`
3. **Merge workflow** → Bumps version, tags, pushes
4. **Tag push** → Triggers `release.yml` → Builds binaries

This ensures version bumps are reviewable and tied to specific changes.

## Cutting a release (manual)

If you need to release without a PR:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers the release workflow directly.
