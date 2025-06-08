#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

mod lang_items;
mod sbi;
mod mem;
mod config;
mod sync;

use core::arch::global_asm;
use crate::sbi::UART;

global_asm!(include_str!("entry.asm"));

#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hart_id: usize, dtb_pa: usize) -> ! {
    clear_bss();
    UART.init();
    assert_eq!(hart_id, 0, "Only hart 0 is supported, but got {}", hart_id);
    mem::heap_allocator::init_heap();
    test_io();
    // mem::heap_allocator::heap_test();
    test_mtime();
    UART.shutdown(true)
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        (sbss as *mut u8).write_bytes(0, (ebss as usize - sbss as usize) / core::mem::size_of::<u8>());
    }
}

#[allow(unused)]
fn test_io() {
    // Print "Hello, world!"
    println!("Hello, world!");
    println!("中文");

    // Test read
    println!("Please input a character:");
    let c: u8 = UART.read();
    println!("Read: {}", c as char);
    #[allow(unreachable_code)]
    if c == 0x61 {
        !panic!("You input '{}' (panic test)", c as char);
        // !panic!();
    }
    // test_large_output();
}

#[allow(unused)]
fn test_large_output() {
    println!("Test large output:");
    for i in 0..1000 {
        print!("Hello, world {}!", i);
    }
}

use core::ptr::{read_volatile, write_volatile};
use core::arch::asm;

#[allow(unused)]
fn test_mtime() {
    let mtime_addr: *const u64 = 0x0200bff8 as *const u64;

    // Get the current hartid (CPU ID)
    let hartid: usize = 0;
    // unsafe {
    //     asm!("csrr {}, mhartid", out(reg) hartid);
    // }

    println!("Got hartid: {}", hartid);

    let mtimecmp_addr: *mut u64 = (0x02004000 + 8 * hartid) as *mut u64;

    println!("Attempting to access timer registers from S-mode...");

    // Try to read mtime
    let mtime_value = unsafe {
        read_volatile(mtime_addr)
    };
    println!("Current mtime: {}", mtime_value);

    // Try to read mtimecmp
    let mtimecmp_value = unsafe {
        read_volatile(mtimecmp_addr)
    };
    println!("Current mtimecmp: {}", mtimecmp_value);

    // Calculate new mtimecmp value
    let new_mtimecmp = mtime_value + 1000000; // 0.1s interval

    println!("Attempting to set mtimecmp to: {}", new_mtimecmp);

    // Try to write to mtimecmp
    unsafe {
        write_volatile(mtimecmp_addr, new_mtimecmp);
    }

    // Verify the write by reading back
    let mtimecmp_after = unsafe {
        read_volatile(mtimecmp_addr)
    };
    println!("mtimecmp after write: {}", mtimecmp_after);

    // Check if our write was successful
    if mtimecmp_after == new_mtimecmp {
        println!("Successfully modified mtimecmp!");
    } else {
        println!("Failed to modify mtimecmp. This is expected if running in S-mode.");
    }
}
