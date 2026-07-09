$ErrorActionPreference = 'SilentlyContinue'

$physicalDisks = @(
  Get-PhysicalDisk |
    Select-Object DeviceId, FriendlyName, Manufacturer, Model, SerialNumber, Size, MediaType, BusType, FirmwareVersion, HealthStatus
)

$diskDrives = @(
  Get-CimInstance Win32_DiskDrive |
    Select-Object DeviceID, Model, SerialNumber, Size, MediaType, InterfaceType, FirmwareRevision, Manufacturer
)

$memory = @(
  Get-CimInstance Win32_PhysicalMemory |
    Select-Object BankLabel, DeviceLocator, Capacity, Speed, ConfiguredClockSpeed, Manufacturer, PartNumber, SerialNumber
)

$cpu = Get-CimInstance Win32_Processor |
  Select-Object -First 1 Name, Manufacturer, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed

$baseBoard = Get-CimInstance Win32_BaseBoard |
  Select-Object -First 1 Manufacturer, Product, Version, SerialNumber

$bios = Get-CimInstance Win32_BIOS |
  Select-Object -First 1 Manufacturer, SMBIOSBIOSVersion, Version, SerialNumber

[pscustomobject]@{
  PhysicalDisks = $physicalDisks
  DiskDrives = $diskDrives
  Memory = $memory
  Cpu = $cpu
  BaseBoard = $baseBoard
  Bios = $bios
} | ConvertTo-Json -Depth 6 -Compress
