#![no_std]
#![no_main]

mod lang_items;

use core::arch::{asm, global_asm};
global_asm!(include_str!("entry.asm"));

// #[unsafe(no_mangle)]
// #[unsafe(link_section = ".text.entry")]
// #[naked]
// fn _start() -> ! {
//     unsafe {
//         asm! {
//             "la sp, boot_stack_top"
//         }
//     }
//     main();
//     loop {}
// }

#[unsafe(no_mangle)]
fn rust_main() -> () {
    unsafe {
        asm!("li ra, 100");
    }
}
