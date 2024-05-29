use std::ffi::{CString, OsString};
use std::os::windows::ffi::OsStrExt;
use winapi::um::winreg::{RegOpenKeyExW, HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::psapi::{EnumProcesses, GetModuleBaseNameA, GetDeviceDriverBaseNameA, EnumDeviceDrivers};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, KEY_READ};
use winapi::ctypes::c_char;
use std::path::Path;
use std::ptr::null_mut;

pub fn check_vm() -> bool {
    check_vm_by_vendor() || check_vm_by_registry() || check_vm_by_processes() || check_vm_by_drivers() || check_vm_by_directories() || check_vm_by_files()
}

fn check_vm_by_vendor() -> bool {
    use std::arch::x86_64::__cpuid;

    let cpuid = unsafe { __cpuid(0) };

    let vendor_id = [
        cpuid.ebx as u8, (cpuid.ebx >> 8) as u8, (cpuid.ebx >> 16) as u8, (cpuid.ebx >> 24) as u8,
        cpuid.edx as u8, (cpuid.edx >> 8) as u8, (cpuid.edx >> 16) as u8, (cpuid.edx >> 24) as u8,
        cpuid.ecx as u8, (cpuid.ecx >> 8) as u8, (cpuid.ecx >> 16) as u8, (cpuid.ecx >> 24) as u8,
    ];

    let vendor = String::from_utf8_lossy(&vendor_id);

    vendor.contains("VMware") || vendor.contains("VBox") || vendor.contains("KVM")
}

fn check_vm_by_registry() -> bool {
    let reg_keys = vec![
        r#"HKEY_LOCAL_MACHINE\HARDWARE\ACPI\DSDT\VBOX__"#,
        r#"HKEY_LOCAL_MACHINE\HARDWARE\ACPI\FADT\VBOX__"#,
        r#"HKEY_LOCAL_MACHINE\HARDWARE\ACPI\RSDT\VBOX__"#,
        r#"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\VBox*"#,
        r#"HKEY_LOCAL_MACHINE\SOFTWARE\Oracle\VirtualBox Guest Additions\"#,
        r#"HKEY_CURRENT_USER\Software\VMware, Inc.\"#,
        r#"HKEY_LOCAL_MACHINE\SOFTWARE\VMware, Inc.\"#,
        r#"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Enum\SCSI\*VMware*\"#,
        r#"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\*VMware*\"#,
    ];

    for key in reg_keys {
        if check_registry_key(key) {
            return true;
        }
    }

    false
}

fn check_registry_key(path: &str) -> bool {
    let (hkey, sub_key) = if path.starts_with("HKEY_LOCAL_MACHINE") {
        let sub_key = path.trim_start_matches("HKEY_LOCAL_MACHINE\\");
        (HKEY_LOCAL_MACHINE, sub_key)
    } else if path.starts_with("HKEY_CURRENT_USER") {
        let sub_key = path.trim_start_matches("HKEY_CURRENT_USER\\");
        (HKEY_CURRENT_USER, sub_key)
    } else {
        return false;
    };

    let sub_key: Vec<u16> = OsString::from(sub_key).encode_wide().chain(Some(0)).collect();
    let mut hkey_result = null_mut();

    unsafe {
        if RegOpenKeyExW(hkey, sub_key.as_ptr(), 0, KEY_READ, &mut hkey_result) == ERROR_SUCCESS.try_into().unwrap() {
            return true;
        }
    }

    false
}

fn check_vm_by_processes() -> bool {
    let process_list = vec!["VBoxService.exe", "VBoxTray.exe", "vmtoolsd.exe", "vmwaretray.exe"];

    for process in process_list {
        if is_process_running(process) {
            return true;
        }
    }

    false
}

fn is_process_running(process_name: &str) -> bool {
    const MAX_PATH: usize = 260;
    const PROCESS_BUFFER_SIZE: usize = 1024;

    let mut process_ids = [0u32; PROCESS_BUFFER_SIZE];
    let mut cb_needed = 0;

    unsafe {
        if EnumProcesses(process_ids.as_mut_ptr(), (PROCESS_BUFFER_SIZE * std::mem::size_of::<u32>()) as u32, &mut cb_needed) == 0 {
            return false;
        }
    }

    let num_processes = cb_needed as usize / std::mem::size_of::<u32>();

    for i in 0..num_processes {
        let process_id = process_ids[i];
        let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id) };
        if handle.is_null() {
            continue;
        }

        let mut process_name_buf = [0 as c_char; MAX_PATH];
        unsafe {
            if GetModuleBaseNameA(handle, null_mut(), process_name_buf.as_mut_ptr(), MAX_PATH as u32) > 0 {
                let process_name_cstr = CString::from_raw(process_name_buf.as_mut_ptr());
                if process_name_cstr.to_str().unwrap() == process_name {
                    return true;
                }
            }
        }
    }

    false
}

fn check_vm_by_drivers() -> bool {
    let drivers_list = vec!["VBoxMouse.sys", "VBoxGuest.sys", "VBoxSF.sys", "VBoxVideo.sys", "vmmouse.sys", "vmhgfs.sys", "vm3dmp.sys"];

    for driver in drivers_list {
        if check_driver(driver) {
            return true;
        }
    }

    false
}

fn check_driver(driver_name: &str) -> bool {
    use winapi::ctypes::c_void;

    const DRIVER_BUFFER_SIZE: usize = 1024;
    let mut drivers = [0 as *mut c_void; DRIVER_BUFFER_SIZE];
    let mut cb_needed = 0;

    unsafe {
        if EnumDeviceDrivers(drivers.as_mut_ptr(), (DRIVER_BUFFER_SIZE * std::mem::size_of::<*mut c_void>()) as u32, &mut cb_needed) == 0 {
            return false;
        }
    }

    let num_drivers = cb_needed as usize / std::mem::size_of::<*mut c_void>();

    for i in 0..num_drivers {
        let mut driver_name_buf = [0 as c_char; 260];
        unsafe {
            if GetDeviceDriverBaseNameA(drivers[i], driver_name_buf.as_mut_ptr(), 260 as u32) > 0 {
                let driver_name_cstr = CString::from_raw(driver_name_buf.as_mut_ptr());
                if driver_name_cstr.to_str().unwrap().contains(driver_name) {
                    return true;
                }
            }
        }
    }

    false
}

fn check_vm_by_directories() -> bool {
    let paths = vec![
        "C:\\Program Files\\Oracle\\VirtualBox Guest Additions",
        "C:\\Program Files\\VMware\\VMware Tools"
    ];

    for path in paths {
        if Path::new(path).exists() {
            return true;
        }
    }

    false
}

fn check_vm_by_files() -> bool {
    let vmware_files = vec![
        "C:\\Windows\\System32\\drivers\\vmhgfs.sys",
        "C:\\Windows\\System32\\drivers\\vmmemctl.sys",
        "C:\\Windows\\System32\\drivers\\vmmouse.sys",
        "C:\\Windows\\System32\\drivers\\vmrawdsk.sys"
    ];

    let virtualbox_files = vec![
        "C:\\Windows\\System32\\drivers\\VBoxMouse.sys",
        "C:\\Windows\\System32\\drivers\\VBoxGuest.sys",
        "C:\\Windows\\System32\\drivers\\VBoxSF.sys",
        "C:\\Windows\\System32\\drivers\\VBoxVideo.sys"
    ];

    for file in vmware_files.iter().chain(virtualbox_files.iter()) {
        if Path::new(file).exists() {
            return true;
        }
    }

    false
}
