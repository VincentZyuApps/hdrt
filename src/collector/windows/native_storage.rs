use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};

use crate::hardware::unknown;

const FILE_SHARE_READ: u32 = 0x00000001;
const FILE_SHARE_WRITE: u32 = 0x00000002;
const OPEN_EXISTING: u32 = 3;
const INVALID_HANDLE_VALUE: isize = -1isize;
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400;
const STORAGE_DEVICE_PROPERTY: u32 = 0;
const PROPERTY_STANDARD_QUERY: u32 = 0;

type Handle = isize;

#[repr(C)]
struct StoragePropertyQuery {
    property_id: u32,
    query_type: u32,
    additional_parameters: [u8; 1],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *const c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: Handle,
    ) -> Handle;

    fn DeviceIoControl(
        hDevice: Handle,
        dwIoControlCode: u32,
        lpInBuffer: *mut c_void,
        nInBufferSize: u32,
        lpOutBuffer: *mut c_void,
        nOutBufferSize: u32,
        lpBytesReturned: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> i32;

    fn CloseHandle(hObject: Handle) -> i32;

    fn GetLastError() -> u32;
}

#[derive(Debug, Clone)]
pub struct StorageDescriptor {
    pub index: u32,
    pub device: String,
    pub vendor: String,
    pub product: String,
    pub revision: String,
    pub serial: String,
    pub bus: String,
    pub raw_size: u32,
}

pub fn query_physical_drive(index: u32) -> Result<StorageDescriptor, String> {
    let device = format!("PhysicalDrive{index}");
    let path = format!(r"\\.\{device}");
    let wide_path = wide_null(&path);

    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null(),
            OPEN_EXISTING,
            0,
            0,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(format!("CreateFileW failed: {}", last_error()));
    }

    let result = query_descriptor(handle, index, device);
    unsafe {
        CloseHandle(handle);
    }
    result
}

fn query_descriptor(
    handle: Handle,
    index: u32,
    device: String,
) -> Result<StorageDescriptor, String> {
    let mut query = StoragePropertyQuery {
        property_id: STORAGE_DEVICE_PROPERTY,
        query_type: PROPERTY_STANDARD_QUERY,
        additional_parameters: [0],
    };
    let mut output = vec![0_u8; 4096];
    let mut bytes_returned = 0_u32;

    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            &mut query as *mut _ as *mut c_void,
            std::mem::size_of::<StoragePropertyQuery>() as u32,
            output.as_mut_ptr() as *mut c_void,
            output.len() as u32,
            &mut bytes_returned,
            null_mut(),
        )
    };

    if ok == 0 {
        return Err(format!("DeviceIoControl failed: {}", last_error()));
    }

    if bytes_returned < 36 {
        return Err(format!(
            "descriptor too short: {bytes_returned} bytes returned"
        ));
    }

    let raw_size = read_u32(&output, 4).unwrap_or(bytes_returned);
    let vendor_offset = read_u32(&output, 12).unwrap_or(0);
    let product_offset = read_u32(&output, 16).unwrap_or(0);
    let revision_offset = read_u32(&output, 20).unwrap_or(0);
    let serial_offset = read_u32(&output, 24).unwrap_or(0);
    let bus_type = read_u32(&output, 28).unwrap_or(0);

    Ok(StorageDescriptor {
        index,
        device,
        vendor: read_offset_string(&output, vendor_offset),
        product: read_offset_string(&output, product_offset),
        revision: read_offset_string(&output, revision_offset),
        serial: read_offset_string(&output, serial_offset),
        bus: storage_bus_type(bus_type),
        raw_size,
    })
}

fn read_u32(buffer: &[u8], offset: usize) -> Option<u32> {
    let bytes = buffer.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_offset_string(buffer: &[u8], offset: u32) -> String {
    let offset = offset as usize;
    if offset == 0 || offset >= buffer.len() {
        return unknown();
    }

    let end = buffer[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|end| offset + end)
        .unwrap_or(buffer.len());

    String::from_utf8_lossy(&buffer[offset..end])
        .trim()
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
        .pipe_unknown()
}

fn storage_bus_type(value: u32) -> String {
    match value {
        1 => "SCSI",
        2 => "ATAPI",
        3 => "ATA",
        4 => "IEEE 1394",
        5 => "SSA",
        6 => "Fibre Channel",
        7 => "USB",
        8 => "RAID",
        9 => "iSCSI",
        10 => "SAS",
        11 => "SATA",
        12 => "SD",
        13 => "MMC",
        14 => "Virtual",
        15 => "FileBackedVirtual",
        16 => "Storage Spaces",
        17 => "NVMe",
        18 => "SCM",
        19 => "UFS",
        0 => "Unknown",
        _ => return format!("BusType({value})"),
    }
    .to_string()
}

fn wide_null(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn last_error() -> u32 {
    unsafe { GetLastError() }
}

trait UnknownIfEmpty {
    fn pipe_unknown(self) -> String;
}

impl UnknownIfEmpty for String {
    fn pipe_unknown(self) -> String {
        if self.is_empty() {
            unknown()
        } else {
            self
        }
    }
}
