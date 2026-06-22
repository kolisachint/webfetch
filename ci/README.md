# CI / Release workflow templates

These are the GitHub Actions workflows for the project. They live here rather
than in `.github/workflows/` because the session that generated them pushed
over an OAuth token **without `workflow` scope**, which GitHub refuses for any
commit touching `.github/workflows/`.

To activate them, move both files into place with a `workflow`-scoped token (or
do it from the GitHub web UI):

```bash
mkdir -p .github/workflows
git mv ci/ci.yml      .github/workflows/ci.yml
git mv ci/release.yml .github/workflows/release.yml
git commit -m "Enable CI and release workflows"
git push
```

- **`ci.yml`** — fmt + clippy (`-D warnings`) + `cargo test --workspace` on
  pushes to `main` and on PRs.
- **`release.yml`** — on a `v*` tag, builds `webtools` for Linux
  (`x86_64-unknown-linux-gnu`) and macOS (`aarch64-apple-darwin`) and attaches
  the tarballs to the GitHub release.

Cutting a release once the workflow is in place:

```bash
git tag v0.1.0
git push origin v0.1.0
```
