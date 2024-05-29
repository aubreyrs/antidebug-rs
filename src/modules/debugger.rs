extern crate winapi;

use winapi::um::debugapi::IsDebuggerPresent;
use winapi::um::sysinfoapi::GetTickCount;
use winapi::um::processthreadsapi::{GetThreadContext, GetCurrentThread};
use winapi::um::winnt::CONTEXT;
use winapi::um::memoryapi::VirtualQuery;
use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, PAGE_GUARD};
use std::mem::zeroed;

pub fn check_debugger() -> bool {
    check_is_debugger_present() || check_timing() || check_thread_context() || check_page_guard()
}

fn check_is_debugger_present() -> bool {
    unsafe { IsDebuggerPresent() != 0 }
}

fn check_timing() -> bool {
    let start = unsafe { GetTickCount() };
    for _ in 0..10000 { let _ = 2 + 2; }
    let end = unsafe { GetTickCount() };
    end - start > 10
}

fn check_thread_context() -> bool {
    let mut context: CONTEXT = unsafe { zeroed() };
    context.ContextFlags = winapi::um::winnt::CONTEXT_DEBUG_REGISTERS;

    unsafe {
        if GetThreadContext(GetCurrentThread(), &mut context) == 0 {
            return false;
        }
    }

    context.Dr0 != 0 || context.Dr1 != 0 || context.Dr2 != 0 || context.Dr3 != 0
}

fn check_page_guard() -> bool {
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };
    let addr = check_page_guard as *const () as *const winapi::ctypes::c_void;

    unsafe {
        if VirtualQuery(addr, &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
            return false;
        }
    }

    (mbi.Protect & PAGE_GUARD) != 0
}
