#![no_std]
#![no_main]
// #![feature(naked_functions)]

mod lang_items;
mod sbi;

use core::arch::{asm, global_asm};
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
fn rust_main() -> () {
    UART.init();
    // Print "Hello, world!"
    UART.write(0x48); // H
    UART.write(0x65); // e
    UART.write(0x6c); // l
    UART.write(0x6c); // l
    UART.write(0x6f); // o
    UART.write(0x2c); // ,
    UART.write(0x20); // space
    UART.write(0x77); // w
    UART.write(0x6f); // o
    UART.write(0x72); // r
    UART.write(0x6c); // l
    UART.write(0x64); // d
    UART.write(0x21); // !
    UART.write(0x0a); // \n

    // Test read
    let c: u8 = UART.read();
    UART.write(c);
    UART.write(0x0a); // \n
    UART.shutdown(true);
}
