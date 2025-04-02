#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod lang_items;
mod sbi;
mod mem;
mod config;

use core::arch::global_asm;
use crate::sbi::UART;

global_asm!(include_str!("entry.asm"));


#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hart_id: usize, dtb_pa: usize) -> ! {
    clear_bss();
    UART.init();
    assert_eq!(hart_id, 0, "Only hart 0 is supported, but got {}", hart_id);
    mem::heap_allocator::init_heap();
    // test_io();
    mem::heap_allocator::heap_test();
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