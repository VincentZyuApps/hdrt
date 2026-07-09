pub fn decorate(enabled: bool, key: &str, text: impl Into<String>) -> String {
    let text = text.into();
    if !enabled {
        return text;
    }

    icon(key)
        .map(|icon| format!("{icon} {text}"))
        .unwrap_or(text)
}

pub fn icon(key: &str) -> Option<&'static str> {
    match key {
        "app.title" => Some("🖥️"),
        "section.all" => Some("🖥️"),
        "section.overview" => Some("🏠"),
        "section.disk" => Some("💽"),
        "section.memory" => Some("🧠"),
        "section.cpu" => Some("🧩"),
        "section.motherboard" => Some("🧱"),
        "section.health" => Some("💚"),
        "warnings" => Some("⚠️"),
        "hint" => Some("💡"),
        "notes" => Some("📝"),
        "platform" => Some("🖥️"),
        "arch" => Some("🧬"),
        "elevated" => Some("🔐"),
        "doctor.title" => Some("🩺"),
        "doctor.name" => Some("🏷️"),
        "doctor.available" => Some("✅"),
        "doctor.path" => Some("📍"),
        "doctor.purpose" => Some("📝"),
        "bench.title" => Some("🧪"),
        "bench.backend" => Some("🧩"),
        "bench.ok" => Some("✅"),
        "bench.elapsed" => Some("⏱️"),
        "bench.disks" => Some("💽"),
        "bench.memory" => Some("🧠"),
        "bench.warnings" => Some("⚠️"),
        "bench.note" => Some("📝"),
        "disk.device" => Some("🔢"),
        "disk.model" => Some("🏷️"),
        "disk.serial" => Some("🔐"),
        "disk.size" => Some("📦"),
        "disk.kind" => Some("💾"),
        "disk.bus" => Some("🔌"),
        "disk.firmware" => Some("🧬"),
        "disk.health" => Some("💚"),
        "memory.slot" => Some("📍"),
        "memory.size" => Some("📦"),
        "memory.speed" => Some("⚡"),
        "memory.manufacturer" => Some("🏭"),
        "memory.part_number" => Some("🏷️"),
        "memory.serial" => Some("🔐"),
        "cpu.model" => Some("🧠"),
        "cpu.vendor" => Some("🏭"),
        "cpu.physical_cores" => Some("🧩"),
        "cpu.logical_threads" => Some("🧵"),
        "cpu.frequency" => Some("⚡"),
        "motherboard.manufacturer" => Some("🏭"),
        "motherboard.product" => Some("🧱"),
        "motherboard.version" => Some("🏷️"),
        "motherboard.serial" => Some("🔐"),
        "motherboard.bios_vendor" => Some("🧬"),
        "motherboard.bios_version" => Some("🏷️"),
        "spinner.collect" => Some("🖥️"),
        "spinner.doctor" => Some("🩺"),
        "spinner.bench" => Some("🧪"),
        "tui.subtitle" => Some("🖥️"),
        "tui.memory_hint" => Some("🧠"),
        "tui.placeholder" => Some("🚧"),
        "tui.help" => Some("⌨️"),
        _ => None,
    }
}
