> **[📖English](build.md)** | **[📖简体中文](build.zh-cn.md)**
<br>
> **[📖Readme](../../README.md)**

# 🚀 Build & Release Workflow

## 🧭 Overview

The `hdrt` CI/CD workflow is driven by commit message keywords. Push to `main` with one of the supported keywords and GitHub Actions will build, release, or publish accordingly.

## 🏷️ Version

The version is extracted from root `Cargo.toml`:

```toml
version = "0.1.0"
```

The workflow turns it into a Release tag like:

```text
v0.1.0
```

Artifact filenames include the same tag.

## 🔑 Keywords

Only these four workflow keywords are enabled for now.

| Keyword in commit message | Build (8 platforms) | GitHub Release | Gitee Release | GitHub/Gitee Scoop | AUR / npm | crates.io |
|---------------------------|:---:|:---:|:---:|:---:|:---:|:---:|
| `build action` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| `build release` | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| `build publish` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| `publish from release` | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |

Notes:

- Pull requests always build, but never publish.
- Each push mirrors the code repository to Gitee.
- `build release` builds binaries, creates a GitHub Release, then syncs the Release files to Gitee.
- `build publish` builds binaries, creates GitHub/Gitee Releases, then updates both GitHub and Gitee Scoop buckets.
- `publish from release` skips building, reuses the existing GitHub Release assets for the current `Cargo.toml` version, syncs them to Gitee, then updates Scoop manifests.
- AUR, npm, and crates.io jobs are reserved for later.

## 🧪 Usage Examples

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

## 🎯 Build Targets

| Platform | Architecture | Target | Asset |
|----------|:---:|--------|-------|
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `hdrt-windows-x86_64-vX.Y.Z.exe` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `hdrt-windows-aarch64-vX.Y.Z.exe` |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `hdrt-linux-x86_64-vX.Y.Z`, `.deb`, `.rpm` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `hdrt-linux-aarch64-vX.Y.Z`, `.deb`, `.rpm` |
| macOS | x86_64 | `x86_64-apple-darwin` | `hdrt-macos-x86_64-vX.Y.Z` |
| macOS | ARM64 | `aarch64-apple-darwin` | `hdrt-macos-aarch64-vX.Y.Z` |
| Android / Termux | ARM64 | `aarch64-linux-android` | `hdrt-android-aarch64-vX.Y.Z` |
| Android / Termux | x86_64 | `x86_64-linux-android` | `hdrt-android-x86_64-vX.Y.Z` |

## 🔁 Pipeline

```text
check
  ├─ parse commit message
  └─ extract version from Cargo.toml

build
  ├─ build 8 release binaries
  ├─ build .deb and .rpm packages for Linux targets
  └─ upload artifacts

release
  ├─ download artifacts
  ├─ generate release notes from .github/release_template.md
  └─ create GitHub Release

sync-gitee-code
  └─ mirror repository commits to gitee.com/vincent-zyu/hdrt

sync-gitee-release
  ├─ read the GitHub Release body and assets
  ├─ create or find the Gitee Release for the same tag
  └─ upload missing Release assets to Gitee

publish-scoop-github
  ├─ download Windows binaries from GitHub Release
  ├─ compute SHA256 hashes
  ├─ generate hdrt.json
  └─ push bucket/hdrt.json to VincentZyuApps/scoop-bucket

publish-scoop-gitee
  ├─ download Windows binaries from GitHub Release for hashing
  ├─ generate hdrt.json with Gitee Release URLs
  └─ push bucket/hdrt.json to gitee.com/vincent-zyu/scoop-bucket
```

## 📝 Release Notes Template

Release notes are generated from:

```text
.github/release_template.md
```

The workflow replaces these placeholders:

| Placeholder | Value |
|-------------|-------|
| `__REPO__` | GitHub repository, such as `VincentZyuApps/hdrt` |
| `__VERSION__` | Release tag, such as `v0.1.5-alpha.8` |
| `__PLAIN_VER__` | Version without the leading `v` |
| `__BASE_URL__` | GitHub Release asset base URL |

## 📦 Linux Packages And Install Scripts

Linux release builds also publish package files:

- `hdrt-linux-x86_64-vX.Y.Z.deb`
- `hdrt-linux-x86_64-vX.Y.Z.rpm`
- `hdrt-linux-aarch64-vX.Y.Z.deb`
- `hdrt-linux-aarch64-vX.Y.Z.rpm`

The install scripts live in:

```text
docs/scripts/install/install.sh
docs/scripts/install/install_gitee.sh
```

They support apt, dnf, and Android / Termux. Set `HDRT_VERSION=vX.Y.Z` to install a specific release.

Gitee release upload logs include per-file index, file size, curl status, HTTP status, elapsed upload time, and the final success summary.

## 🍨 Scoop Publish

The GitHub Scoop job publishes a manifest named `hdrt.json` to:

```text
VincentZyuApps/scoop-bucket
```

The Gitee Scoop job publishes a mirror manifest named `hdrt.json` to:

```text
gitee.com/vincent-zyu/scoop-bucket
```

The manifest supports:

- Windows x86_64
- Windows ARM64

## 🔐 Secrets And Tokens

The workflow needs these repository secrets. Use the setup links to jump to the matching guide.

| Secret | Purpose | Setup guide |
|--------|---------|-------------|
| `SCOOP_BUCKET_TOKEN` | GitHub PAT with permission to push to `VincentZyuApps/scoop-bucket` | [Getting `SCOOP_BUCKET_TOKEN`](#secret-scoop-bucket-token) |
| `GITEE_TOKEN` | Gitee token with permission to mirror code, create releases, upload release assets, and push `gitee.com/vincent-zyu/scoop-bucket` | [Getting `GITEE_TOKEN`](#secret-gitee-token) |
| `GITEE_PRIVATE_KEY` | SSH private key used by `Yikun/hub-mirror-action` for GitHub -> Gitee repository mirroring | [Getting `GITEE_PRIVATE_KEY`](#secret-gitee-private-key) |

<a id="secret-scoop-bucket-token"></a>

### Getting `SCOOP_BUCKET_TOKEN`

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

<a id="secret-gitee-token"></a>

### Getting `GITEE_TOKEN`

Use a dedicated Gitee personal access token for GitHub Actions.

1. Sign in to Gitee.
2. Open `https://gitee.com/profile/personal_access_tokens`.
3. Create a new personal access token with a name such as `hdrt-github-actions`.
4. Grant repository/project write access that allows repository mirroring, tag/release creation, release asset upload, and pushing to `vincent-zyu/scoop-bucket`.
5. Generate the token, then copy it once.
6. Open `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`.
7. Click `New repository secret`, name it `GITEE_TOKEN`, paste the token, then save.

Required access:

| Item | Value |
|------|-------|
| Token type | Gitee personal access token |
| Gitee repositories | `vincent-zyu/hdrt`, `vincent-zyu/scoop-bucket` |
| Required permission | Repository/project read and write access |
| Secret location | `VincentZyuApps/hdrt` repository Actions secrets |

Notes:

- The token owner must have write permission to both Gitee repositories.
- Keep the token separate from personal daily-use credentials.
- Replace the secret before the token expires or is rotated.

Reference links:

- Gitee personal access tokens: `https://gitee.com/profile/personal_access_tokens`
- GitHub repository secrets: `https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets`

<a id="secret-gitee-private-key"></a>

### Getting `GITEE_PRIVATE_KEY`

`GITEE_PRIVATE_KEY` is the SSH private key used by `Yikun/hub-mirror-action` to push mirrored commits to Gitee. Use a dedicated key pair for this workflow.

1. Generate a dedicated SSH key pair on your machine.

   Bash / Git Bash / Linux / macOS:

   ```bash
   mkdir -p ~/.ssh
   ssh-keygen -t ed25519 -C "hdrt-gitee-mirror" -f ~/.ssh/hdrt_gitee_mirror -N ""
   ```

   Windows PowerShell:

   ```powershell
   New-Item -ItemType Directory -Force "$HOME\.ssh"
   ssh-keygen -t ed25519 -C "hdrt-gitee-mirror" -f "$HOME\.ssh\hdrt_gitee_mirror"
   ```

   When PowerShell prompts for `Enter passphrase` and `Enter same passphrase again`, press Enter twice to leave the passphrase empty.

   PowerShell note: avoid passing `~/.ssh/hdrt_gitee_mirror` directly to `ssh-keygen`; depending on the shell/program boundary, it may be treated as a literal path and fail with `No such file or directory`. Use `"$HOME\.ssh\hdrt_gitee_mirror"` instead.

2. Open the public key file and copy its content.

   Bash / Git Bash / Linux / macOS:

   ```bash
   cat ~/.ssh/hdrt_gitee_mirror.pub
   ```

   Windows PowerShell:

   ```powershell
   Get-Content "$HOME\.ssh\hdrt_gitee_mirror.pub"
   ```

3. Sign in to Gitee and open `https://gitee.com/profile/sshkeys`.
4. Add the public key to the Gitee account that can push to `vincent-zyu/hdrt`.
5. Open the private key file and copy the full content, including the `BEGIN` and `END` lines.

   Bash / Git Bash / Linux / macOS:

   ```bash
   cat ~/.ssh/hdrt_gitee_mirror
   ```

   Windows PowerShell:

   ```powershell
   Get-Content "$HOME\.ssh\hdrt_gitee_mirror" -Raw
   ```

6. Open `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`.
7. Click `New repository secret`, name it `GITEE_PRIVATE_KEY`, paste the private key, then save.

Optional SSH check:

```bash
ssh -i ~/.ssh/hdrt_gitee_mirror -T git@gitee.com
```

Windows PowerShell:

```powershell
ssh -i "$HOME\.ssh\hdrt_gitee_mirror" -T git@gitee.com
```

Required access:

| Item | Value |
|------|-------|
| Key type | Dedicated SSH key pair |
| Public key location | Gitee account SSH keys |
| Private key location | GitHub Actions secret `GITEE_PRIVATE_KEY` |
| Required permission | The Gitee account must be able to push to `vincent-zyu/hdrt` |

Notes:

- Prefer a dedicated key without a passphrase for this GitHub Actions job.
- Do not reuse your personal daily-use SSH private key.
- If you pasted the private key into a chat, issue, log, or any other shared place, remove the corresponding Gitee public key and GitHub secret, then generate a fresh key pair before using it.
- Rotate the key if it is ever exposed in logs or copied to an untrusted machine.

Reference links:

- Gitee SSH keys: `https://gitee.com/profile/sshkeys`
- GitHub repository secrets: `https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets`

### Secrets Summary

The secrets table is repeated here so it is easy to find from the end of the document.

| Secret | Purpose | Setup guide |
|--------|---------|-------------|
| `SCOOP_BUCKET_TOKEN` | GitHub PAT with permission to push to `VincentZyuApps/scoop-bucket` | [Getting `SCOOP_BUCKET_TOKEN`](#secret-scoop-bucket-token) |
| `GITEE_TOKEN` | Gitee token with permission to mirror code, create releases, upload release assets, and push `gitee.com/vincent-zyu/scoop-bucket` | [Getting `GITEE_TOKEN`](#secret-gitee-token) |
| `GITEE_PRIVATE_KEY` | SSH private key used by `Yikun/hub-mirror-action` for GitHub -> Gitee repository mirroring | [Getting `GITEE_PRIVATE_KEY`](#secret-gitee-private-key) |
