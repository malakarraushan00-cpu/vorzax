/// Utility functions and macros for the Vorzax kernel

use core::fmt;

/// Simple logging macro for kernel debug output
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = write!($crate::util::LOGGER, $($arg)*);
        }
    };
}

pub struct Logger;

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            crate::driver::uart::putc(byte);
        }
        Ok(())
    }
}

pub static mut LOGGER: Logger = Logger;

/// Spin-wait delay in microseconds
pub fn delay_us(microseconds: u64) {
    let cpu_freq = 2_400_000_000u64; // 2.4 GHz typical ARM64
    let cycles = (cpu_freq / 1_000_000) * microseconds;
    let mut c = 0u64;
    while c < cycles {
        unsafe { core::arch::asm!("nop") };
        c += 1;
    }
}

/// Sleep in milliseconds using delay_us
pub fn sleep_ms(milliseconds: u64) {
    delay_us(milliseconds * 1000);
}

/// Align value up to alignment boundary
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Align value down to alignment boundary
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Check if address is aligned
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    addr % align == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0x1000, 0x1000), 0x1000);
        assert_eq!(align_up(0x1001, 0x1000), 0x2000);
        assert_eq!(align_up(0x2999, 0x1000), 0x3000);
    }

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(0x1000, 0x1000), 0x1000);
        assert_eq!(align_down(0x1001, 0x1000), 0x1000);
        assert_eq!(align_down(0x2999, 0x1000), 0x2000);
    }
}
