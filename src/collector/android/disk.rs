use crate::hardware::{DiskInfo, HdrtWarning};

use super::command::{format_bytes, run_command};

pub(super) fn collect() -> Vec<DiskInfo> {
    match run_df() {
        Ok(output) => {
            let disks = parse_df(&output);
            if disks.is_empty() {
                vec![unavailable_disk(
                    "df returned no parseable Android storage rows.",
                )]
            } else {
                disks
            }
        }
        Err(err) => vec![DiskInfo {
            source: "df".to_string(),
            warnings: vec![HdrtWarning::with_hint(
                "android-df-unavailable",
                format!("Could not run df to collect Android storage inventory: {err}"),
                "Install coreutils in Termux or run hdrt in a less restricted Android environment.",
            )],
            ..DiskInfo::default()
        }],
    }
}

fn run_df() -> Result<String, String> {
    run_command("df", &["-kP"])
        .or_else(|_| run_command("df", &["-k"]))
        .or_else(|_| run_command("df", &[]))
}

fn unavailable_disk(message: &str) -> DiskInfo {
    DiskInfo {
        source: "df".to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "android-storage-inventory-empty",
            message,
            "Run hdrt in Termux with coreutils installed for a more standard df output.",
        )],
        ..DiskInfo::default()
    }
}

fn parse_df(output: &str) -> Vec<DiskInfo> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 6 {
                return None;
            }

            let filesystem = fields[0];
            let blocks_kib = fields.get(1)?.parse::<u64>().ok()?;
            let mount = fields[5];

            Some(DiskInfo {
                device: mount.to_string(),
                model: filesystem.to_string(),
                size: format_bytes(blocks_kib * 1024),
                media_type: "Logical".to_string(),
                source: "df".to_string(),
                ..DiskInfo::default()
            })
        })
        .collect()
}
