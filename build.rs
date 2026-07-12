use std::env;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=HDRT_BUILD_TARGET={target}");
    emit_git_build_info();
}

fn emit_git_build_info() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/packed-refs");
    println!("cargo:rerun-if-env-changed=HDRT_GIT_COMMIT_HASH");
    println!("cargo:rerun-if-env-changed=HDRT_GIT_COMMIT_TIME");

    if let Some(head_ref) = git_output(&["symbolic-ref", "-q", "HEAD"]) {
        if let Some(ref_path) = git_output(&["rev-parse", "--git-path", &head_ref]) {
            println!("cargo:rerun-if-changed={ref_path}");
        }
    }

    let commit_hash = env_value("HDRT_GIT_COMMIT_HASH")
        .or_else(|| git_output(&["rev-parse", "--short=7", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());
    let commit_time = env_value("HDRT_GIT_COMMIT_TIME")
        .or_else(|| git_output(&["show", "-s", "--format=%cI", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=HDRT_GIT_COMMIT_HASH={commit_hash}");
    println!("cargo:rustc-env=HDRT_GIT_COMMIT_TIME={commit_time}");
}

fn env_value(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
