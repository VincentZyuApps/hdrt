#[derive(Debug, Clone, Default)]
pub(super) struct DiskIoCounter {
    pub(super) name: String,
    pub(super) read_bytes_per_sec: f64,
    pub(super) write_bytes_per_sec: f64,
    pub(super) total_read_bytes: u64,
    pub(super) total_written_bytes: u64,
}

#[cfg(windows)]
mod platform {
    use serde::Deserialize;
    use wmi::WMIConnection;

    use super::DiskIoCounter;

    pub(crate) struct DiskIoSampler {
        conn: Option<WMIConnection>,
    }

    impl DiskIoSampler {
        pub(crate) fn new() -> Self {
            Self {
                conn: WMIConnection::with_namespace_path("ROOT\\CIMV2").ok(),
            }
        }

        pub(crate) fn sample(&mut self) -> Vec<DiskIoCounter> {
            let Some(conn) = &self.conn else {
                return Vec::new();
            };

            let query = "SELECT Name, DiskReadBytesPersec, DiskWriteBytesPersec \
                FROM Win32_PerfFormattedData_PerfDisk_LogicalDisk";
            conn.raw_query::<Win32LogicalDiskPerf>(query)
                .map(|rows| {
                    rows.into_iter()
                        .filter(|row| row.name.as_deref() != Some("_Total"))
                        .map(|row| DiskIoCounter {
                            name: row.name.unwrap_or_default(),
                            read_bytes_per_sec: row.disk_read_bytes_per_sec.unwrap_or_default()
                                as f64,
                            write_bytes_per_sec: row.disk_write_bytes_per_sec.unwrap_or_default()
                                as f64,
                            ..DiskIoCounter::default()
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
    }

    #[derive(Debug, Deserialize)]
    struct Win32LogicalDiskPerf {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "DiskReadBytesPersec")]
        disk_read_bytes_per_sec: Option<u64>,
        #[serde(rename = "DiskWriteBytesPersec")]
        disk_write_bytes_per_sec: Option<u64>,
    }
}

#[cfg(unix)]
mod platform {
    use std::collections::HashMap;
    use std::fs;
    use std::time::Instant;

    use super::DiskIoCounter;

    type DeviceNumber = (u32, u32);

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    struct RawDiskIo {
        read_bytes: u64,
        written_bytes: u64,
    }

    pub(crate) struct DiskIoSampler {
        previous: HashMap<DeviceNumber, RawDiskIo>,
        last_sample: Option<Instant>,
    }

    impl DiskIoSampler {
        pub(crate) fn new() -> Self {
            Self {
                previous: HashMap::new(),
                last_sample: None,
            }
        }

        pub(crate) fn sample(&mut self) -> Vec<DiskIoCounter> {
            let now = Instant::now();
            let current = read_diskstats();
            let mounts = read_mountinfo();
            let elapsed = self
                .last_sample
                .map(|last| now.saturating_duration_since(last).as_secs_f64());

            let counters = mounts
                .into_iter()
                .filter_map(|(mount_point, device)| {
                    let value = current.get(&device)?;
                    let previous = self.previous.get(&device);
                    Some(DiskIoCounter {
                        name: mount_point,
                        read_bytes_per_sec: rate(
                            value.read_bytes,
                            previous.map(|item| item.read_bytes),
                            elapsed,
                        ),
                        write_bytes_per_sec: rate(
                            value.written_bytes,
                            previous.map(|item| item.written_bytes),
                            elapsed,
                        ),
                        total_read_bytes: value.read_bytes,
                        total_written_bytes: value.written_bytes,
                    })
                })
                .collect();

            self.previous = current;
            self.last_sample = Some(now);
            counters
        }
    }

    fn read_diskstats() -> HashMap<DeviceNumber, RawDiskIo> {
        fs::read_to_string("/proc/diskstats")
            .map(|text| parse_diskstats(&text))
            .unwrap_or_default()
    }

    fn parse_diskstats(text: &str) -> HashMap<DeviceNumber, RawDiskIo> {
        text.lines()
            .filter_map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                let major = fields.first()?.parse::<u32>().ok()?;
                let minor = fields.get(1)?.parse::<u32>().ok()?;
                let sectors_read = fields.get(5)?.parse::<u64>().ok()?;
                let sectors_written = fields.get(9)?.parse::<u64>().ok()?;
                Some((
                    (major, minor),
                    RawDiskIo {
                        // Linux diskstats sectors are defined as 512-byte units.
                        read_bytes: sectors_read.saturating_mul(512),
                        written_bytes: sectors_written.saturating_mul(512),
                    },
                ))
            })
            .collect()
    }

    fn read_mountinfo() -> Vec<(String, DeviceNumber)> {
        fs::read_to_string("/proc/self/mountinfo")
            .map(|text| parse_mountinfo(&text))
            .unwrap_or_default()
    }

    fn parse_mountinfo(text: &str) -> Vec<(String, DeviceNumber)> {
        text.lines()
            .filter_map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                let (major, minor) = fields.get(2)?.split_once(':')?;
                let device = (major.parse::<u32>().ok()?, minor.parse::<u32>().ok()?);
                let mount_point = decode_mount_field(fields.get(4)?);
                Some((mount_point, device))
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

    fn rate(current: u64, previous: Option<u64>, elapsed: Option<f64>) -> f64 {
        let (Some(previous), Some(elapsed)) = (previous, elapsed) else {
            return 0.0;
        };
        if elapsed <= f64::EPSILON {
            return 0.0;
        }
        current.saturating_sub(previous) as f64 / elapsed
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parses_linux_diskstats_sector_counters() {
            let parsed = parse_diskstats("253 0 dm-0 10 0 2048 0 20 0 4096 0 0 0 0 0 0 0\n");

            assert_eq!(
                parsed.get(&(253, 0)),
                Some(&RawDiskIo {
                    read_bytes: 1_048_576,
                    written_bytes: 2_097_152,
                })
            );
        }

        #[test]
        fn parses_and_decodes_mountinfo_mount_points() {
            let parsed = parse_mountinfo(
                "42 31 253:0 / /storage/My\\040Disk rw,nosuid - ext4 /dev/block/dm-0 rw\n",
            );

            assert_eq!(parsed, vec![("/storage/My Disk".to_string(), (253, 0))]);
        }

        #[test]
        fn rate_requires_a_previous_sample() {
            assert_eq!(rate(2_048, None, Some(1.0)), 0.0);
            assert_eq!(rate(2_048, Some(1_024), Some(2.0)), 512.0);
        }
    }
}

#[cfg(not(any(windows, unix)))]
mod platform {
    use super::DiskIoCounter;

    pub(crate) struct DiskIoSampler;

    impl DiskIoSampler {
        pub(crate) fn new() -> Self {
            Self
        }

        pub(crate) fn sample(&mut self) -> Vec<DiskIoCounter> {
            Vec::new()
        }
    }
}

pub(super) use platform::DiskIoSampler;
