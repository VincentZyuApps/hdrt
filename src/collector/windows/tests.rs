use serde_json::json;

use super::{cpu, disk, memory, motherboard};

#[test]
fn maps_cpu_inventory_fields() {
    let root = json!({
        "Cpu": {
            "Name": "Fixture CPU",
            "Manufacturer": "Fixture Vendor",
            "NumberOfCores": 8,
            "NumberOfLogicalProcessors": 16,
            "MaxClockSpeed": 4200
        }
    });

    let cpu = cpu::collect(&root).unwrap();
    assert_eq!(cpu.model, "Fixture CPU");
    assert_eq!(cpu.physical_cores, Some(8));
    assert_eq!(cpu.logical_threads, Some(16));
    assert_eq!(cpu.frequency, "4200 MHz");
}

#[test]
fn maps_memory_and_falls_back_to_reported_speed() {
    let root = json!({
        "Memory": [{
            "DeviceLocator": "DIMM0",
            "Capacity": 17179869184_u64,
            "ConfiguredClockSpeed": null,
            "Speed": "3200",
            "Manufacturer": "Fixture Memory",
            "PartNumber": "PART-1",
            "SerialNumber": "SERIAL-1"
        }]
    });

    let memory = memory::collect(&root);
    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0].size, "16.00 GiB");
    assert_eq!(memory[0].speed, "3200 MT/s");
    assert_eq!(memory[0].slot, "DIMM0");
}

#[test]
fn combines_baseboard_and_bios_inventory() {
    let root = json!({
        "BaseBoard": {
            "Manufacturer": "Fixture Board",
            "Product": "Board 1",
            "Version": "1.0",
            "SerialNumber": "BOARD-SERIAL"
        },
        "Bios": {
            "Manufacturer": "Fixture BIOS",
            "SMBIOSBIOSVersion": null,
            "Version": "2.0"
        }
    });

    let board = motherboard::collect(&root).unwrap();
    assert_eq!(board.product, "Board 1");
    assert_eq!(board.serial, "BOARD-SERIAL");
    assert_eq!(board.bios_version, "2.0");
}

#[test]
fn falls_back_to_disk_drive_inventory() {
    let root = json!({
        "DiskDrives": [{
            "DeviceID": "PhysicalDrive0",
            "Model": "Fixture HDD",
            "SerialNumber": "DISK-SERIAL",
            "Size": 500107862016_u64,
            "MediaType": "Fixed hard disk media",
            "InterfaceType": "SATA",
            "FirmwareRevision": "1.0"
        }]
    });

    let disks = disk::collect(&root);
    assert_eq!(disks.len(), 1);
    assert_eq!(disks[0].device, "PhysicalDrive0");
    assert_eq!(disks[0].model, "Fixture HDD");
    assert_eq!(disks[0].firmware, "1.0");
    assert_eq!(disks[0].source, "Win32_DiskDrive");
}
