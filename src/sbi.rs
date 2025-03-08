#![allow(dead_code)]
#![allow(unused)]

use core::sync::atomic::{AtomicU8, Ordering};
use core::hint::spin_loop;
use volatile::access::{ReadOnly, ReadWrite};
use volatile::{VolatileFieldAccess, VolatileRef};
use core::fmt::{self, Write};

// Hardcoded UART base address for QEMU virt
const UART_BASE: usize = 0x10000000;
const SHUTDOWN_BASE: usize = 0x100000;

const BS: u8 = 0x8;  // Backspace
const DEL: u8 = 0x7F; // Delete
const SHUTDOWN_PASS: u16 = 0x5555;
const SHUTDOWN_FAIL: u16 = 0x3333;

/// Read port when DLAB = 0.
#[repr(C)]
#[derive(VolatileFieldAccess, Default)]
struct ReadPort {
    /// receive buffer
    rbr: AtomicU8,
    /// interrupt enable
    ier: u8,
    /// interrupt identification
    #[access(ReadOnly)]
    iir: u8,
    /// line control
    lcr: u8,
    /// modem control
    mcr: u8,
    /// line status
    lsr: AtomicU8,
    /// modem status
    msr: u8,
    // scratch
    scr: u8,
}

/// Write port when DLAB = 0.
#[repr(C)]
#[derive(VolatileFieldAccess, Default)]
struct WritePort {
    /// transmitter holding
    thr: AtomicU8,
    /// interrupt enable
    ier: u8,
    /// FIFO control
    fcr: u8,
    /// line control
    lcr: u8,
    /// modem control
    mcr: u8,
    /// line status
    lsr: AtomicU8,
    /// not used
    #[access(ReadOnly)]
    _padding: u8,
    // scratch
    #[access(ReadOnly)]
    scr: u8,
}

// Constants for register bit flags
mod flags {
    // Interrupt Enable register flags
    pub const IER_RX_AVAILABLE: u8 = 1 << 0;
    pub const IER_TX_EMPTY: u8 = 1 << 1;

    // FIFO Control register flags
    pub const FCR_ENABLE: u8 = 1 << 0;
    pub const FCR_CLEAR_RX_FIFO: u8 = 1 << 1;
    pub const FCR_CLEAR_TX_FIFO: u8 = 1 << 2;
    pub const FCR_TRIGGER_14: u8 = 0b11 << 6;

    // Line Control register flags
    pub const LCR_DATA_8: u8 = 0b11;
    pub const LCR_DLAB_ENABLE: u8 = 1 << 7;

    // Modem Control register flags
    pub const MCR_DATA_TERMINAL_READY: u8 = 1 << 0;
    pub const MCR_AUXILIARY_OUTPUT_2: u8 = 1 << 3;

    // Line Status register flags
    pub const LSR_INPUT_AVAILABLE: u8 = 1 << 0;
    pub const LSR_OUTPUT_EMPTY: u8 = 1 << 5;
}

/// Simple UART driver
pub struct Uart;

impl Uart {
    /// Get a reference to the read port
    fn read_port(&self) -> &'static mut ReadPort {
        unsafe { &mut *(UART_BASE as *mut ReadPort) }
    }

    /// Get a reference to the write port
    fn write_port(&self) -> &'static mut WritePort { unsafe { &mut *(UART_BASE as *mut WritePort) } }

    /// Initialize the UART with standard settings
    pub fn init(&self) {
        let read_port = self.read_port();
        let mut read_port = VolatileRef::from_mut_ref(read_port);
        let read_port = read_port.as_mut_ptr();
        let write_port = self.write_port();
        let mut write_port = VolatileRef::from_mut_ref(write_port);
        let write_port = write_port.as_mut_ptr();

        // disable interrupts
        read_port.ier().write(0);

        // enable DLAB
        read_port.lcr().write(flags::LCR_DLAB_ENABLE);

        // set maximum speed of 38.4K for LSB
        unsafe {
            (*(read_port.as_raw_ptr().as_ptr())).rbr.store(0x03, Ordering::Release);
        }

        // set maximum speed of 38.4K for MSB
        read_port.ier().write(0);

        // disable DLAB and set data word length to 8 bits
        read_port.lcr().write(flags::LCR_DATA_8);

        // enable FIFO, clear TX/RX queues and set interrupt watermark at 14 bytes
        // write_port.fcr = flags::FCR_ENABLE | flags::FCR_CLEAR_RX_FIFO |
        // flags::FCR_CLEAR_TX_FIFO | flags::FCR_TRIGGER_14;
        write_port.fcr().write(flags::FCR_ENABLE | flags::FCR_CLEAR_RX_FIFO |
            flags::FCR_CLEAR_TX_FIFO | flags::FCR_TRIGGER_14);

        // mark data terminal ready, signal request to send and enable auxiliary output
        read_port.mcr().write(flags::MCR_DATA_TERMINAL_READY | flags::MCR_AUXILIARY_OUTPUT_2);

        // enable receive interrupts (we'll poll in read() function)
        // read_port.ier = flags::IER_RX_AVAILABLE;
        read_port.ier().write(flags::IER_RX_AVAILABLE);
    }

    /// Read a byte from the UART (blocking)
    pub fn read(&self) -> u8 {
        let read_port = self.read_port();
        let lsr = &read_port.lsr;
        let rbr = &read_port.rbr;

        // Wait until input is available
        while lsr.load(Ordering::Acquire) & flags::LSR_INPUT_AVAILABLE == 0 {
            spin_loop();
        }

        // Read the byte
        rbr.load(Ordering::Acquire)
    }

    /// Write a byte to the UART
    pub fn write(&self, data: u8) {
        let write_port = self.write_port();
        let lsr = &write_port.lsr;
        let thr = &mut write_port.thr;

        match data {
            BS | DEL => {
                // Wait until output buffer is empty
                while (lsr.load(Ordering::Acquire) & flags::LSR_OUTPUT_EMPTY) == 0 {
                    spin_loop();
                }
                thr.store(BS, Ordering::Release);

                // Send a space to overwrite the previous character
                while (lsr.load(Ordering::Acquire) & flags::LSR_OUTPUT_EMPTY) == 0 {
                    spin_loop();
                }
                thr.store(b' ', Ordering::Release);

                // Send another backspace to move cursor back
                while (lsr.load(Ordering::Acquire) & flags::LSR_OUTPUT_EMPTY) == 0 {
                    spin_loop();
                }
                thr.store(BS, Ordering::Release);
            }
            _ => {
                // Wait until output buffer is empty
                while (lsr.load(Ordering::Acquire) & flags::LSR_OUTPUT_EMPTY) == 0 {
                    spin_loop();
                }
                thr.store(data, Ordering::Release);
            }
        }
    }

    // Shutdown the machine
    pub fn shutdown(&self, success: bool) -> ! {
        let shutdown = SHUTDOWN_BASE as *mut u16;
        unsafe {
            shutdown.write_volatile(if success { SHUTDOWN_PASS } else { SHUTDOWN_FAIL });
        }
        loop {}
    }
}

// Create a global instance
pub static UART: Uart = Uart;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            UART.write(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::sbi::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::sbi::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}