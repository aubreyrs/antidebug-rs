use std::process::Command;

pub fn check_network() -> bool {
    check_mac_addresses() || check_network_adapters()
}

fn check_mac_addresses() -> bool {
    let vm_mac_prefixes = vec![
        "00:05:69",
        "00:0C:29",
        "00:1C:14",
        "00:50:56",
        "08:00:27",
        "52:54:00",
    ];

    let output = Command::new("getmac")
        .output()
        .expect("Failed to execute command");

    let mac_addresses = String::from_utf8_lossy(&output.stdout);

    for line in mac_addresses.lines() {
        for prefix in &vm_mac_prefixes {
            if line.trim().starts_with(prefix) {
                return true;
            }
        }
    }

    false
}

fn check_network_adapters() -> bool {
    let vm_adapter_names = vec![
        "VMware Virtual Ethernet Adapter",
        "VirtualBox Host-Only Ethernet Adapter",
        "VirtualBox Bridged Networking Driver",
        "Intel(R) PRO/1000 MT Desktop Adapter",
        "Intel(R) PRO/1000 MT Server Adapter",
    ];

    let output = Command::new("ipconfig")
        .output()
        .expect("Failed to execute command");
    let adapters = String::from_utf8_lossy(&output.stdout);
    for line in adapters.lines() {
        for adapter in &vm_adapter_names {
            if line.contains(adapter) {
                return true;
            }
        }
    }

    false
}
