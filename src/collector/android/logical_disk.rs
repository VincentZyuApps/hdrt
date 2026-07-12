use std::collections::HashMap;
use std::fs;

use crate::hardware::{is_unknown, DebugRecord, HdrtWarning, LogicalDiskInfo};

use super::command::run_command;

const DF_ARGUMENTS: &[&[&str]] = &[&["-kPT"], &["-kP"], &["-kT"], &["-k"], &[]];

pub(super) struct Collection {
    pub(super) disks: Vec<LogicalDiskInfo>,
    pub(super) warnings: Vec<HdrtWarning>,
    pub(super) debug: Vec<DebugRecord>,
}

pub(super) fn collect(debug_enabled: bool) -> Collection {
    let mount_filesystems = fs::read_to_string("/proc/mounts")
        .map(|text| parse_mount_filesystems(&text))
        .unwrap_or_default();
    let mut debug = Vec::new();
    let mut errors = Vec::new();
    let mut any_success = false;

    for args in DF_ARGUMENTS {
        match run_command("df", args) {
            Ok(output) => {
                any_success = true;
                let mut parsed = crate::collector::df::parse(&output, debug_enabled);
                enrich_filesystems(&mut parsed.disks, &mount_filesystems);
                let selected = !parsed.disks.is_empty();

                if debug_enabled {
                    debug.push(
                        DebugRecord::new("df", "android-df")
                            .field("arguments", argument_label(args))
                            .field("rows", parsed.disks.len().to_string())
                            .field("selected", selected.to_string()),
                    );
                    debug.extend(parsed.debug);
                }

                if selected {
                    return Collection {
                        disks: parsed.disks,
                        warnings: Vec::new(),
                        debug,
                    };
                }
            }
            Err(err) => {
                if debug_enabled {
                    debug.push(
                        DebugRecord::new("df", "android-df")
                            .field("arguments", argument_label(args))
                            .field("error", err.clone())
                            .field("selected", "false"),
                    );
                }
                errors.push(format!("df {}: {err}", argument_label(args)));
            }
        }
    }

    if any_success {
        Collection {
            disks: Vec::new(),
            warnings: vec![HdrtWarning::with_hint(
                "android-storage-inventory-empty",
                "df returned no user-facing Android storage rows.",
                "Run hdrt --debug in Termux to inspect each df variant and filtered mount.",
            )],
            debug,
        }
    } else {
        Collection {
            disks: Vec::new(),
            warnings: vec![HdrtWarning::with_hint(
                "android-df-unavailable",
                format!(
                    "Could not run df to collect Android logical storage: {}",
                    errors.join("; ")
                ),
                "Install coreutils in Termux or make sure /system/bin/df is accessible.",
            )],
            debug,
        }
    }
}

fn enrich_filesystems(disks: &mut [LogicalDiskInfo], mount_filesystems: &HashMap<String, String>) {
    for disk in disks {
        if is_unknown(&disk.file_system) {
            if let Some(file_system) = mount_filesystems.get(&disk.mount_point) {
                disk.file_system = file_system.clone();
            }
        }
    }
}

fn parse_mount_filesystems(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let mount_point = decode_mount_field(fields.get(1)?);
            let file_system = fields.get(2)?.to_string();
            Some((mount_point, file_system))
        })
        .collect()
}

fn decode_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

fn argument_label(args: &[&str]) -> String {
    if args.is_empty() {
        "(none)".to_string()
    } else {
        args.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mount_filesystems_and_decodes_spaces() {
        let mounts = parse_mount_filesystems("/dev/fuse /storage/My\\040Disk fuse rw,nosuid 0 0\n");

        assert_eq!(
            mounts.get("/storage/My Disk").map(String::as_str),
            Some("fuse")
        );
    }

    #[test]
    fn fills_unknown_df_filesystem_from_proc_mounts() {
        let mut disks = vec![LogicalDiskInfo {
            device: "/dev/fuse".to_string(),
            mount_point: "/storage/emulated".to_string(),
            ..LogicalDiskInfo::default()
        }];
        let mounts = HashMap::from([("/storage/emulated".to_string(), "fuse".to_string())]);

        enrich_filesystems(&mut disks, &mounts);

        assert_eq!(disks[0].file_system, "fuse");
    }

    #[test]
    fn formats_empty_df_argument_list_for_debug_output() {
        assert_eq!(argument_label(&[]), "(none)");
        assert_eq!(argument_label(&["-kP"]), "-kP");
    }
}
