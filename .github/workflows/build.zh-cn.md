> **[📖English](build.md)** | **[📖简体中文](build.zh-cn.md)**

<br>

> **[📖Readme](../../README.zh-cn.md)**

# 🚀 构建与发布工作流

## 🧭 概述

`hdrt` 的 CI/CD 工作流由 commit message 中的关键词驱动。推送到 `main` 分支时，只要包含支持的关键词，GitHub Actions 就会执行对应的构建、发布或 Scoop 更新流程。

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
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `hdrt-linux-x86_64-vX.Y.Z`、`.deb`、`.rpm` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `hdrt-linux-aarch64-vX.Y.Z`、`.deb`、`.rpm` |
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
  ├─ 为 Linux target 构建 .deb 和 .rpm 包
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
| `__VERSION__` | Release tag，例如 `v0.1.5-alpha.8` |
| `__PLAIN_VER__` | 去掉开头 `v` 的版本号 |
| `__BASE_URL__` | GitHub Release 产物基础 URL |

## 📦 Linux 包与安装脚本

Linux Release 构建会额外发布这些包：

- `hdrt-linux-x86_64-vX.Y.Z.deb`
- `hdrt-linux-x86_64-vX.Y.Z.rpm`
- `hdrt-linux-aarch64-vX.Y.Z.deb`
- `hdrt-linux-aarch64-vX.Y.Z.rpm`

安装脚本放在：

```text
docs/scripts/install/install.sh
docs/scripts/install/install_gitee.sh
```

脚本支持 apt、dnf 和 Android / Termux。需要安装指定版本时，可以设置 `HDRT_VERSION=vX.Y.Z`。

Gitee Release 上传日志会显示每个文件的序号、文件大小、curl 状态、HTTP 状态、上传耗时，以及最终成功数量汇总。

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

## 🔐 Secrets 与密钥

工作流需要配置下面这些 repository secrets。可以通过表格里的配置教程链接跳转到对应段落。

| Secret | 用途 | 配置教程 |
|--------|------|----------|
| `SCOOP_BUCKET_TOKEN` | 可推送到 `VincentZyuApps/scoop-bucket` 的 GitHub PAT | [获取 `SCOOP_BUCKET_TOKEN`](#secret-scoop-bucket-token) |
| `GITEE_TOKEN` | 可同步代码、创建 Release、上传 Release 产物并推送 `gitee.com/vincent-zyu/scoop-bucket` 的 Gitee token | [获取 `GITEE_TOKEN`](#secret-gitee-token) |
| `GITEE_PRIVATE_KEY` | `Yikun/hub-mirror-action` 用于 GitHub -> Gitee 仓库镜像的 SSH 私钥 | [获取 `GITEE_PRIVATE_KEY`](#secret-gitee-private-key) |

<a id="secret-scoop-bucket-token"></a>

### 获取 `SCOOP_BUCKET_TOKEN`

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

<a id="secret-gitee-token"></a>

### 获取 `GITEE_TOKEN`

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

<a id="secret-gitee-private-key"></a>

### 获取 `GITEE_PRIVATE_KEY`

`GITEE_PRIVATE_KEY` 是 `Yikun/hub-mirror-action` 用来向 Gitee 推送镜像提交的 SSH 私钥。建议为这个 workflow 单独生成一对 SSH key。

1. 在本机生成专用 SSH key。

   Bash / Git Bash / Linux / macOS：

   ```bash
   mkdir -p ~/.ssh
   ssh-keygen -t ed25519 -C "hdrt-gitee-mirror" -f ~/.ssh/hdrt_gitee_mirror -N ""
   ```

   Windows PowerShell：

   ```powershell
   New-Item -ItemType Directory -Force "$HOME\.ssh"
   ssh-keygen -t ed25519 -C "hdrt-gitee-mirror" -f "$HOME\.ssh\hdrt_gitee_mirror"
   ```

   PowerShell 提示 `Enter passphrase` 和 `Enter same passphrase again` 时，连续按两次回车即可留空 passphrase。

   PowerShell 注意：不要把 `~/.ssh/hdrt_gitee_mirror` 直接传给 `ssh-keygen`；在 shell 和外部程序边界上，它可能会被当成字面路径，最后报 `No such file or directory`。稳妥写法是使用 `"$HOME\.ssh\hdrt_gitee_mirror"`。

2. 打开公钥文件并复制内容。

   Bash / Git Bash / Linux / macOS：

   ```bash
   cat ~/.ssh/hdrt_gitee_mirror.pub
   ```

   Windows PowerShell：

   ```powershell
   Get-Content "$HOME\.ssh\hdrt_gitee_mirror.pub"
   ```

3. 登录 Gitee，打开 `https://gitee.com/profile/sshkeys`。
4. 将公钥添加到有权限推送 `vincent-zyu/hdrt` 的 Gitee 账号。
5. 打开私钥文件并复制完整内容，包括 `BEGIN` 和 `END` 行。

   Bash / Git Bash / Linux / macOS：

   ```bash
   cat ~/.ssh/hdrt_gitee_mirror
   ```

   Windows PowerShell：

   ```powershell
   Get-Content "$HOME\.ssh\hdrt_gitee_mirror" -Raw
   ```

6. 打开 `VincentZyuApps/hdrt` -> `Settings` -> `Secrets and variables` -> `Actions`。
7. 点击 `New repository secret`，名称填写 `GITEE_PRIVATE_KEY`，粘贴私钥后保存。

可选 SSH 连通性检查：

```bash
ssh -i ~/.ssh/hdrt_gitee_mirror -T git@gitee.com
```

Windows PowerShell：

```powershell
ssh -i "$HOME\.ssh\hdrt_gitee_mirror" -T git@gitee.com
```

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
- 如果私钥已经被粘贴到聊天、issue、日志或其他共享位置，应删除对应的 Gitee 公钥和 GitHub secret，然后重新生成一对 key 再使用。
- 如果私钥出现在日志或被复制到不可信机器上，应立即轮换。

参考链接：

- Gitee SSH 公钥：`https://gitee.com/profile/sshkeys`
- GitHub 仓库 Secrets：`https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets`

### Secrets 速查

为了方便从文档末尾查找，这里重复一次密钥表。

| Secret | 用途 | 配置教程 |
|--------|------|----------|
| `SCOOP_BUCKET_TOKEN` | 可推送到 `VincentZyuApps/scoop-bucket` 的 GitHub PAT | [获取 `SCOOP_BUCKET_TOKEN`](#secret-scoop-bucket-token) |
| `GITEE_TOKEN` | 可同步代码、创建 Release、上传 Release 产物并推送 `gitee.com/vincent-zyu/scoop-bucket` 的 Gitee token | [获取 `GITEE_TOKEN`](#secret-gitee-token) |
| `GITEE_PRIVATE_KEY` | `Yikun/hub-mirror-action` 用于 GitHub -> Gitee 仓库镜像的 SSH 私钥 | [获取 `GITEE_PRIVATE_KEY`](#secret-gitee-private-key) |
