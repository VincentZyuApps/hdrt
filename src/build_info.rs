const PRODUCT_LABEL: &str = "Hardware Device Rust Ratatui";

pub(crate) fn try_print_version() -> bool {
    let args = std::env::args().collect::<Vec<_>>();
    if !version_requested(&args) {
        return false;
    }

    let bold = !args.iter().any(|arg| arg == "--no-bold");
    print!("{}", version_output(bold));
    true
}

fn version_requested(args: &[String]) -> bool {
    args.iter()
        .skip(1)
        .any(|arg| arg == "--version" || arg == "-V")
}

fn version_output(bold: bool) -> String {
    let title = format!("hdrt {} ({PRODUCT_LABEL})", env!("CARGO_PKG_VERSION"));
    let title = if bold {
        format!("\x1b[1m{title}\x1b[0m")
    } else {
        title
    };

    format!(
        "{title}\nCommit Hash: {} | Commit Time: {}\nSystem: {} | Arch: {} | Target: {}\n",
        env!("HDRT_GIT_COMMIT_HASH"),
        env!("HDRT_GIT_COMMIT_TIME"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("HDRT_BUILD_TARGET")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn recognizes_long_and_short_version_flags() {
        assert!(version_requested(&args(&["hdrt", "--version"])));
        assert!(version_requested(&args(&["hdrt", "-V"])));
        assert!(!version_requested(&args(&["hdrt", "--version=value"])));
    }

    #[test]
    fn version_output_contains_build_and_target_metadata() {
        let output = version_output(false);

        assert!(output.starts_with(&format!("hdrt {}", env!("CARGO_PKG_VERSION"))));
        assert!(output.contains("Commit Hash:"));
        assert!(output.contains("Commit Time:"));
        assert!(output.contains("System:"));
        assert!(output.contains("Arch:"));
        assert!(output.contains("Target:"));
        assert!(!output.contains("\x1b[1m"));
    }

    #[test]
    fn bold_version_only_styles_the_title_line() {
        let output = version_output(true);

        assert!(output.starts_with("\x1b[1m"));
        assert_eq!(output.matches("\x1b[1m").count(), 1);
        assert_eq!(output.matches("\x1b[0m").count(), 1);
    }
}
