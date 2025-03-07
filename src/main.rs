#![no_std]
#![no_main]
// #![feature(naked_functions)]

mod lang_items;

use core::arch::{asm, global_asm};
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
    unsafe {
        asm!("li a0, 100");
    }
}
