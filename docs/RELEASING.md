# Release Process

## Steps

### 1. Update the version

In `Cargo.toml`, bump `version` following [Semantic Versioning](https://semver.org):
- Patch (`0.1.x`): bug fixes, doc corrections, no API changes
- Minor (`0.x.0`): new backwards-compatible features
- Major (`x.0.0`): breaking changes

### 2. Update CHANGELOG.md

Move items from `## [Unreleased]` into a new dated section:

```markdown
## [0.1.1] - YYYY-MM-DD

### Fixed
- ...
```

Add a link reference at the bottom of the file:

```markdown
[0.1.1]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.0...v0.1.1
```

### 3. Update README.md

- Bump the version in the `[dependencies]` example to match the new release.
- Update the **Resources covered** section to reflect any new resource types added in this release.

### 4. Verify CI locally (all matrix configs)

The CI matrix runs three separate configurations. You **must** run all three locally and fix any failures before tagging. A failure in any one of them will break CI on the published tag.

```bash
# 0. Formatting (fails fast — do this first)
cargo fmt --check

# 1. Default features (async only)
cargo test
cargo clippy -- -D warnings

# 2. Blocking feature, no default features — futures crate is NOT available here
cargo test --no-default-features --features blocking
cargo clippy --no-default-features --features blocking -- -D warnings

# 3. Full feature set
cargo test --features full
cargo clippy --features full -- -D warnings
```

Common pitfall: test files that `use futures::StreamExt` or call `.collect::<Vec<_>>().await`
will compile under `--features full` (which enables `async` and thus `futures`) but fail under
`--no-default-features --features blocking`. Always use `collect_all().await?` instead.

### 5. Commit and push

Run `cargo update --workspace` first to update `Cargo.lock` to reflect the new version:

```bash
cargo update --workspace
git add Cargo.toml Cargo.lock CHANGELOG.md README.md <any other changed files>
git commit -m "Release v0.1.1"
git push
```

### 6. Tag and push the tag

```bash
git tag v0.1.1
git push origin v0.1.1
```

This triggers the `Publish` GitHub Actions workflow.

### 7. Approve the deployment

The publish job waits for manual approval due to the `crates-io` environment protection rule.

Go to **Actions → Publish → Review deployments → Approve**.

Or via CLI:
```bash
gh run list --repo RobertConde/canvas-lms-api --limit 5
# Find the waiting Publish run ID, then:
gh run review <run-id> --approve
```

### 8. Verify the publish run succeeded

```bash
gh run view <run-id> --repo RobertConde/canvas-lms-api
```

Confirm the job shows `✓ Publish to crates.io` with no errors or warnings.

Check the crate is live:
```bash
cargo search canvas-lms-api
```
