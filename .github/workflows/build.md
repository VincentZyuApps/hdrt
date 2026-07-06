# Build & Release Workflow

> **[English](build.md)**
> **[简体中文](build.zh-cn.md)**

## Overview

The `hdrt` CI/CD workflow is driven by commit message keywords. Push to `main` with one of the supported keywords and GitHub Actions will build, release, or publish accordingly.

## Keywords

Only these four workflow keywords are enabled for now.

| Keyword in commit message | Build (8 platforms) | GitHub Release | Scoop | AUR / npm | crates.io |
|---------------------------|:---:|:---:|:---:|:---:|:---:|
| `build action` | ✅ | ❌ | ❌ | ❌ | ❌ |
| `build release` | ✅ | ✅ | ❌ | ❌ | ❌ |
| `build publish` | ✅ | ✅ | ✅ | ❌ | ❌ |
| `publish from release` | ❌ | ❌ | ✅ | ❌ | ❌ |

Notes:

- Pull requests always build, but never publish.
- `build publish` builds binaries, creates a GitHub Release, then updates the Scoop bucket.
- `publish from release` skips building and reuses the existing GitHub Release assets for the current `Cargo.toml` version.
- AUR, npm, and crates.io jobs are reserved for later.

## Usage Examples

```bash
# Build all enabled targets.
git commit --allow-empty -m "ci: test hdrt cross build (build action)"

# Build and create GitHub Release.
git commit -m "release: v0.1.0 (build release)"

# Build, create GitHub Release, and publish Scoop manifest.
git commit -m "release: v0.1.0 (build publish)"

# Re-publish Scoop manifest from an existing Release.
git commit --allow-empty -m "ci: update scoop manifest (publish from release)"
```

## Build Targets

| Platform | Architecture | Target | Asset |
|----------|:---:|--------|-------|
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `hdrt-windows-x86_64-vX.Y.Z.exe` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `hdrt-windows-aarch64-vX.Y.Z.exe` |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `hdrt-linux-x86_64-vX.Y.Z` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `hdrt-linux-aarch64-vX.Y.Z` |
| macOS | x86_64 | `x86_64-apple-darwin` | `hdrt-macos-x86_64-vX.Y.Z` |
| macOS | ARM64 | `aarch64-apple-darwin` | `hdrt-macos-aarch64-vX.Y.Z` |
| Android / Termux | ARM64 | `aarch64-linux-android` | `hdrt-android-aarch64-vX.Y.Z` |
| Android / Termux | x86_64 | `x86_64-linux-android` | `hdrt-android-x86_64-vX.Y.Z` |

## Pipeline

```text
check
  ├─ parse commit message
  └─ extract version from Cargo.toml

build
  ├─ build 8 release binaries
  └─ upload artifacts

release
  ├─ download artifacts
  ├─ generate release notes
  └─ create GitHub Release

publish-scoop
  ├─ download Windows binaries from GitHub Release
  ├─ compute SHA256 hashes
  ├─ generate hdrt.json
  └─ push bucket/hdrt.json to VincentZyuApps/scoop-bucket
```

## Scoop Publish

The Scoop job publishes a manifest named `hdrt.json` to:

```text
VincentZyuApps/scoop-bucket
```

The manifest supports:

- Windows x86_64
- Windows ARM64

Required secret:

| Secret | Purpose |
|--------|---------|
| `SCOOP_BUCKET_TOKEN` | GitHub PAT with permission to push to `VincentZyuApps/scoop-bucket` |

## Getting `SCOOP_BUCKET_TOKEN`

Prefer a fine-grained personal access token.

1. Open `https://github.com/settings/personal-access-tokens/new`.
2. Set `Token name` to something like `hdrt-scoop-bucket`.
3. Select an expiration date, such as 90 or 180 days.
4. Set `Resource owner` to the owner that contains `scoop-bucket`.
5. Set `Repository access` to `Only select repositories`.
6. Select `VincentZyuApps/scoop-bucket`.
7. Under `Repository permissions`, set `Contents` to `Read and write`.
8. Click `Generate token`, then copy the token once.
9. Open `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`.
10. Click `New repository secret`, name it `SCOOP_BUCKET_TOKEN`, paste the token, then save.

Required access:

| Item | Value |
|------|-------|
| Token type | Fine-grained PAT preferred |
| Repository | `VincentZyuApps/scoop-bucket` |
| Repository permission | `Contents: Read and write` |
| Secret location | `VincentZyuApps/hdrt` repository Actions secrets |

Classic PAT fallback:

- Use only when fine-grained PAT cannot access the bucket.
- Grant the `repo` scope, then store it as the same `SCOOP_BUCKET_TOKEN` secret.

Notes:

- The token owner must already have push permission to `VincentZyuApps/scoop-bucket`.
- Replace the secret before the token expires.

## Version

The version is extracted from root `Cargo.toml`:

```toml
version = "0.1.0"
```

The workflow turns it into a Release tag like:

```text
v0.1.0
```

Artifact filenames include the same tag.
