# 🚀 构建与发布工作流

> **[English](build.md)**
> **[简体中文](build.zh-cn.md)**

## 🧭 概述

`hdrt` 的 CI/CD 工作流由 commit message 中的关键词驱动。推送到 `main` 分支时，只要包含支持的关键词，GitHub Actions 就会执行对应的构建、发布或 Scoop 更新流程。

## 🔑 关键词

当前只启用下面四种工作流关键词。

| Commit 信息中的关键词 | 构建（8 平台） | GitHub Release | Gitee Release | GitHub/Gitee Scoop | AUR / npm | crates.io |
|----------------------|:---:|:---:|:---:|:---:|:---:|:---:|
| `build action` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| `build release` | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| `build publish` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| `publish from release` | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |

说明：

- Pull Request 始终会构建，但不会发布。
- 每次 push 都会同步代码仓库到 Gitee。
- `build release` 会构建二进制、创建 GitHub Release，然后同步 Release 文件到 Gitee。
- `build publish` 会构建二进制、创建 GitHub/Gitee Release，然后同时更新 GitHub 和 Gitee Scoop bucket。
- `publish from release` 不重新构建，会复用当前 `Cargo.toml` 版本对应的已有 GitHub Release 产物，先同步到 Gitee，再更新 Scoop manifest。
- AUR、npm 和 crates.io 任务暂时预留，后续再接。

## 🧪 用法示例

```bash
# 构建所有已启用目标。
git commit --allow-empty -m "ci: test hdrt cross build (build action)"

# 构建并创建 GitHub Release。
git commit -m "release: v0.1.0 (build release)"

# 构建、创建 GitHub Release，并发布 Scoop manifest。
git commit -m "release: v0.1.0 (build publish)"

# 从已有 Release 重新发布 Scoop manifest。
git commit --allow-empty -m "ci: update scoop manifest (publish from release)"
```

## 🎯 构建目标

| 平台 | 架构 | Target | 产物 |
|------|:---:|--------|------|
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `hdrt-windows-x86_64-vX.Y.Z.exe` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `hdrt-windows-aarch64-vX.Y.Z.exe` |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `hdrt-linux-x86_64-vX.Y.Z` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `hdrt-linux-aarch64-vX.Y.Z` |
| macOS | x86_64 | `x86_64-apple-darwin` | `hdrt-macos-x86_64-vX.Y.Z` |
| macOS | ARM64 | `aarch64-apple-darwin` | `hdrt-macos-aarch64-vX.Y.Z` |
| Android / Termux | ARM64 | `aarch64-linux-android` | `hdrt-android-aarch64-vX.Y.Z` |
| Android / Termux | x86_64 | `x86_64-linux-android` | `hdrt-android-x86_64-vX.Y.Z` |

## 🔁 流水线

```text
check
  ├─ 解析 commit message
  └─ 从 Cargo.toml 提取版本

build
  ├─ 构建 8 个 release 二进制
  └─ 上传构建产物

release
  ├─ 下载构建产物
  ├─ 从 .github/release_template.md 生成 release notes
  └─ 创建 GitHub Release

sync-gitee-code
  └─ 镜像代码提交到 gitee.com/vincent-zyu/hdrt

sync-gitee-release
  ├─ 读取 GitHub Release 正文和产物
  ├─ 创建或查找同 tag 的 Gitee Release
  └─ 补齐缺失的 Release 产物到 Gitee

publish-scoop-github
  ├─ 从 GitHub Release 下载 Windows 二进制
  ├─ 计算 SHA256
  ├─ 生成 hdrt.json
  └─ 推送 bucket/hdrt.json 到 VincentZyuApps/scoop-bucket

publish-scoop-gitee
  ├─ 从 GitHub Release 下载 Windows 二进制用于计算 hash
  ├─ 生成指向 Gitee Release URL 的 hdrt.json
  └─ 推送 bucket/hdrt.json 到 gitee.com/vincent-zyu/scoop-bucket
```

## 📝 Release Notes 模板

Release notes 从下面的模板生成：

```text
.github/release_template.md
```

工作流会替换这些占位符：

| 占位符 | 值 |
|--------|----|
| `__REPO__` | GitHub 仓库，例如 `VincentZyuApps/hdrt` |
| `__VERSION__` | Release tag，例如 `v0.1.4-alpha.7` |
| `__PLAIN_VER__` | 去掉开头 `v` 的版本号 |
| `__BASE_URL__` | GitHub Release 产物基础 URL |

## 🍨 Scoop 发布

GitHub Scoop job 会发布名为 `hdrt.json` 的 manifest 到：

```text
VincentZyuApps/scoop-bucket
```

Gitee Scoop job 会发布镜像 manifest 到：

```text
gitee.com/vincent-zyu/scoop-bucket
```

manifest 支持：

- Windows x86_64
- Windows ARM64

需要配置的密钥：

| Secret | 用途 |
|--------|------|
| `SCOOP_BUCKET_TOKEN` | 可推送到 `VincentZyuApps/scoop-bucket` 的 GitHub PAT |
| `GITEE_TOKEN` | 可同步代码、创建 Release、上传 Release 产物并推送 `gitee.com/vincent-zyu/scoop-bucket` 的 Gitee token |
| `GITEE_PRIVATE_KEY` | `Yikun/hub-mirror-action` 用于 GitHub -> Gitee 仓库镜像的 SSH 私钥 |

## 🔐 获取 `SCOOP_BUCKET_TOKEN`

推荐使用 fine-grained personal access token。

1. 打开 `https://github.com/settings/personal-access-tokens/new`。
2. 将 `Token name` 设为类似 `hdrt-scoop-bucket` 的名称。
3. 选择过期时间，例如 90 天或 180 天。
4. 将 `Resource owner` 设为包含 `scoop-bucket` 的所有者。
5. 将 `Repository access` 设为 `Only select repositories`。
6. 选择 `VincentZyuApps/scoop-bucket`。
7. 在 `Repository permissions` 中，将 `Contents` 设为 `Read and write`。
8. 点击 `Generate token`，然后立即复制 token。
9. 打开 `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`。
10. 点击 `New repository secret`，名称填写 `SCOOP_BUCKET_TOKEN`，粘贴 token 后保存。

所需权限：

| 项目 | 值 |
|------|----|
| Token 类型 | 优先使用 fine-grained PAT |
| 仓库 | `VincentZyuApps/scoop-bucket` |
| 仓库权限 | `Contents: Read and write` |
| Secret 位置 | `VincentZyuApps/hdrt` 仓库 Actions secrets |

Classic PAT 兜底：

- 仅在 fine-grained PAT 无法访问 bucket 时使用。
- 授予 `repo` scope，然后同样保存为 `SCOOP_BUCKET_TOKEN` secret。

注意事项：

- Token 所属账号必须已经拥有 `VincentZyuApps/scoop-bucket` 的 push 权限。
- 在 token 过期前替换 secret。

## 🔐 获取 `GITEE_TOKEN`

建议为 GitHub Actions 单独创建一个 Gitee 私人令牌。

1. 登录 Gitee。
2. 打开 `https://gitee.com/profile/personal_access_tokens`。
3. 创建新的私人令牌，名称可以填 `hdrt-github-actions`。
4. 授予仓库 / 项目写入权限，确保它可以执行仓库镜像、创建 tag / release、上传 Release 附件，以及推送 `vincent-zyu/scoop-bucket`。
5. 生成令牌，然后立即复制 token。
6. 打开 `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`。
7. 点击 `New repository secret`，名称填写 `GITEE_TOKEN`，粘贴 token 后保存。

所需权限：

| 项目 | 值 |
|------|----|
| Token 类型 | Gitee 私人令牌 |
| Gitee 仓库 | `vincent-zyu/hdrt`、`vincent-zyu/scoop-bucket` |
| 所需权限 | 仓库 / 项目读写权限 |
| Secret 位置 | `VincentZyuApps/hdrt` 仓库 Actions secrets |

注意事项：

- Token 所属 Gitee 账号必须对两个 Gitee 仓库都有写入权限。
- 这个 token 建议只给 CI 使用，不要和日常个人凭据混用。
- token 过期或轮换后，需要同步替换 GitHub Actions secret。

参考链接：

- Gitee 私人令牌：`https://gitee.com/profile/personal_access_tokens`
- GitHub 仓库 Secrets：`https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets`

## 🔑 获取 `GITEE_PRIVATE_KEY`

`GITEE_PRIVATE_KEY` 是 `Yikun/hub-mirror-action` 用来向 Gitee 推送镜像提交的 SSH 私钥。建议为这个 workflow 单独生成一对 SSH key。

1. 在本机生成专用 SSH key：

   ```bash
   ssh-keygen -t ed25519 -C "hdrt-gitee-mirror" -f ~/.ssh/hdrt_gitee_mirror
   ```

2. 打开公钥文件并复制内容：

   ```bash
   cat ~/.ssh/hdrt_gitee_mirror.pub
   ```

3. 登录 Gitee，打开 `https://gitee.com/profile/sshkeys`。
4. 将公钥添加到有权限推送 `vincent-zyu/hdrt` 的 Gitee 账号。
5. 打开私钥文件并复制完整内容，包括 `BEGIN` 和 `END` 行：

   ```bash
   cat ~/.ssh/hdrt_gitee_mirror
   ```

6. 打开 `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`。
7. 点击 `New repository secret`，名称填写 `GITEE_PRIVATE_KEY`，粘贴私钥后保存。

所需权限：

| 项目 | 值 |
|------|----|
| Key 类型 | 专用 SSH key pair |
| 公钥位置 | Gitee 账号 SSH 公钥 |
| 私钥位置 | GitHub Actions secret `GITEE_PRIVATE_KEY` |
| 所需权限 | Gitee 账号必须可以推送 `vincent-zyu/hdrt` |

注意事项：

- 这个 GitHub Actions job 建议使用不带 passphrase 的专用 key。
- 不要复用日常个人 SSH 私钥。
- 如果私钥出现在日志或被复制到不可信机器上，应立即轮换。

参考链接：

- Gitee SSH 公钥：`https://gitee.com/profile/sshkeys`
- GitHub 仓库 Secrets：`https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets`

## 🏷️ 版本号

版本号从根目录 `Cargo.toml` 提取：

```toml
version = "0.1.0"
```

工作流会转换成 Release tag：

```text
v0.1.0
```

构建产物文件名也会包含同一个 tag。
