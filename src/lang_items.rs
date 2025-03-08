use core::panic::PanicInfo;
use crate::sbi::UART;
use crate::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[1;31mPanicked at {}:{}\n{}\x1b[0m",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("\x1b[1;31mPanicked:\n{:?}\x1b[0m",
                 info.message());
    }
    UART.shutdown(false)
}
