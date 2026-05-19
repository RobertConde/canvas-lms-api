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

### 3. Commit and push

Run `cargo update --workspace` first to update `Cargo.lock` to reflect the new version:

```bash
cargo update --workspace
git add Cargo.toml Cargo.lock CHANGELOG.md <any other changed files>
git commit -m "Release v0.1.1"
git push
```

### 4. Tag and push the tag

```bash
git tag v0.1.1
git push origin v0.1.1
```

This triggers the `Publish` GitHub Actions workflow.

### 5. Approve the deployment

The publish job waits for manual approval due to the `crates-io` environment protection rule.

Go to **Actions → Publish → Review deployments → Approve**.

Or via CLI:
```bash
gh run list --repo RobertConde/canvas-lms-api --limit 5
# Find the waiting Publish run ID, then:
gh run review <run-id> --approve
```

### 6. Verify the publish run succeeded

```bash
gh run view <run-id> --repo RobertConde/canvas-lms-api
```

Confirm the job shows `✓ Publish to crates.io` with no errors or warnings.

Check the crate is live:
```bash
cargo search canvas-lms-api
```
