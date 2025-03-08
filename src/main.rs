#![no_std]
#![no_main]
// #![feature(naked_functions)]

mod lang_items;
mod sbi;

use core::arch::global_asm;
use crate::sbi::UART;

global_asm!(include_str!("entry.asm"));
// use core::arch::naked_asm;


// #[unsafe(no_mangle)]
// #[unsafe(link_section = ".text.entry")]
// #[naked]
// extern "C" fn _start() {
//     unsafe {
//         naked_asm! {
//             "la sp, boot_stack_top",
//             "call rust_main",
//             "j ."
//         }
//     }
// }

#[unsafe(no_mangle)]
fn rust_main() -> ! {
    UART.init();
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
    UART.shutdown(true)
}
