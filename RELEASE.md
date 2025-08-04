# Release Process

This document describes the release process for KeepKey Bitcoin-Only Vault.

## Quick Start

### Creating a New Release (e.g., 2.2.6)

```bash
# Method 1: Create release branch and update version in one command
make release-branch VERSION=2.2.6

# Method 2: Auto-increment version and create branch
make release-patch-branch  # For patch release (2.2.5 -> 2.2.6)
make release-minor-branch  # For minor release (2.2.5 -> 2.3.0)
make release-major-branch  # For major release (2.2.5 -> 3.0.0)
```

## Release Workflow

### 1. Create Release Branch and Update Version

```bash
# Create release branch and update all version files
make release-branch VERSION=2.2.6
```

This command will:
- Create a new branch named `release-2.2.6`
- Update version in all relevant files:
  - `VERSION`
  - `projects/vault-v2/package.json`
  - `projects/vault-v2/src-tauri/tauri.conf.json`
  - `projects/vault-v2/src-tauri/Cargo.toml`
  - `projects/keepkey-rust/Cargo.toml`
  - `projects/vault-v2/VERSION`
  - `projects/keepkey-rust/VERSION`
  - Lock files (`package-lock.json`, `Cargo.lock`)

### 2. Review Changes

```bash
git diff
```

### 3. Build and Test

```bash
# Build the application
make vault-build

# Run tests
make test-keepkey-rust
```

### 4. Commit Changes

```bash
git add .
git commit -m "chore: release v2.2.6"
```

### 5. Push Release Branch

```bash
git push -u origin release-2.2.6
```

### 6. Create Pull Request

Create a PR from `release-2.2.6` to `master` branch on GitHub.

### 7. After PR Merge

Once the PR is merged to master:

```bash
# Checkout master and pull latest
git checkout master
git pull

# Create and push tag
git tag -a v2.2.6 -m "Release v2.2.6"
git push --tags
```

The tag will trigger the GitHub Actions release workflow to:
- Build applications for all platforms (Windows, macOS, Linux)
- Create a GitHub release with artifacts
- Upload built binaries

## Version Management

### Version Sources

The primary version source is `projects/vault-v2/package.json`. All other version files are synchronized from this source.

### Files Updated During Release

- `VERSION` - Root version file
- `projects/vault-v2/package.json` - Main application package
- `projects/vault-v2/src-tauri/tauri.conf.json` - Tauri configuration
- `projects/vault-v2/src-tauri/Cargo.toml` - Rust backend
- `projects/keepkey-rust/Cargo.toml` - KeepKey Rust library
- `projects/vault-v2/VERSION` - Project version file
- `projects/keepkey-rust/VERSION` - Library version file
- Lock files (`package-lock.json`, `Cargo.lock`)

### Makefile Targets

#### Release Branch Creation
- `make release-branch VERSION=x.y.z` - Create branch and set specific version
- `make release-patch-branch` - Auto-increment patch version and create branch
- `make release-minor-branch` - Auto-increment minor version and create branch
- `make release-major-branch` - Auto-increment major version and create branch

#### Version Update Only
- `make release VERSION=x.y.z` - Update version without creating branch
- `make release-patch` - Increment patch version (2.2.5 -> 2.2.6)
- `make release-minor` - Increment minor version (2.2.5 -> 2.3.0)
- `make release-major` - Increment major version (2.2.5 -> 3.0.0)

## GitHub Actions

The project includes automated workflows:

### Build Workflow (`.github/workflows/build.yml`)
**Triggered by:**
- Push to `main`, `master`, or `release-*` branches
- Pull requests to these branches
- Manual workflow dispatch

**Purpose:** CI/CD builds and testing for all platforms

### Release Workflow (`.github/workflows/release.yml`)
**Triggered by:**
- Tags starting with `v*` (e.g., `v2.2.6`)
- Manual workflow dispatch

**Purpose:** Creates official GitHub releases with artifacts for:
- Windows (MSI installer)
- macOS (DMG installer)
- Linux (AppImage, DEB packages)

## Best Practices

1. **Always create a release branch** for version updates
2. **Test thoroughly** before pushing the release branch
3. **Use semantic versioning**:
   - Patch (x.y.Z): Bug fixes, minor updates
   - Minor (x.Y.z): New features, backward compatible
   - Major (X.y.z): Breaking changes
4. **Document changes** in commit messages and PR descriptions
5. **Tag only after** PR is merged to master

## Troubleshooting

### Version Mismatch
If version files get out of sync:
```bash
make release VERSION=<correct-version>
```

### Failed GitHub Actions
Check the Actions tab in GitHub for build logs and errors.

### Local Build Issues
```bash
# Clean and rebuild
make clean
make rebuild
```