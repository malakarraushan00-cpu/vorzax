/// UART Serial Driver for ARM64
/// 
/// Supports standard ARM UART (PL011) interface

use spin::Mutex;
use volatile::Volatile;

/// UART hardware registers (PL011)
#[repr(C)]
pub struct UartRegisters {
    pub data: Volatile<u32>,        // 0x00
    pub rsrecr: Volatile<u32>,      // 0x04
    _reserved1: [u32; 4],
    pub fr: Volatile<u32>,          // 0x18 - Flag register
    _reserved2: [u32; 2],
    pub ibrd: Volatile<u32>,        // 0x24 - Integer baud rate divisor
    pub fbrd: Volatile<u32>,        // 0x28 - Fractional baud rate divisor
    pub lcr_h: Volatile<u32>,       // 0x2C - Line control register
    pub cr: Volatile<u32>,          // 0x30 - Control register
    pub ifls: Volatile<u32>,        // 0x34 - Interrupt FIFO level select
    pub imsc: Volatile<u32>,        // 0x38 - Interrupt mask set/clear
    pub ris: Volatile<u32>,         // 0x3C - Raw interrupt status
    pub mis: Volatile<u32>,         // 0x40 - Masked interrupt status
    pub icr: Volatile<u32>,         // 0x44 - Interrupt clear register
}

/// UART Driver
pub struct UartDriver {
    base_addr: u64,
    baud_rate: u32,
}

impl UartDriver {
    pub fn new(base_addr: u64, baud_rate: u32) -> Self {
        UartDriver {
            base_addr,
            baud_rate,
        }
    }
    
    fn registers(&self) -> &'static mut UartRegisters {
        unsafe { &mut *(self.base_addr as *mut UartRegisters) }
    }
    
    pub fn init(&self) {
        let regs = self.registers();
        
        // Disable UART
        regs.cr.write(0);
        
        // Set baud rate
        // Assuming 24MHz clock and 115200 baud
        // IBRD = UART_CLK / (16 * BAUD)
        // FBRD = (FRAC * 64 + 0.5) / 16
        regs.ibrd.write(13);  // 24MHz / (16 * 115200) = 13
        regs.fbrd.write(1);
        
        // Set line control (8 bits, 1 stop, no parity)
        regs.lcr_h.write(0x70);  // 8-bit, FIFO enabled
        
        // Enable UART (TXE, RXE)
        regs.cr.write(0x0301);
    }
    
    pub fn putc(&self, c: u8) {
        let regs = self.registers();
        
        // Wait for TXFF flag to clear (transmit FIFO full)
        while (regs.fr.read() & 0x20) != 0 {
            unsafe { core::arch::asm!("nop") };
        }
        
        // Write character to data register
        regs.data.write(c as u32);
    }
    
    pub fn getc(&self) -> u8 {
        let regs = self.registers();
        
        // Wait for RXFE flag to clear (receive FIFO empty)
        while (regs.fr.read() & 0x10) != 0 {
            unsafe { core::arch::asm!("nop") };
        }
        
        // Read character
        (regs.data.read() & 0xFF) as u8
    }
    
    pub fn is_rx_ready(&self) -> bool {
        let regs = self.registers();
        (regs.fr.read() & 0x10) == 0
    }
    
    pub fn is_tx_ready(&self) -> bool {
        let regs = self.registers();
        (regs.fr.read() & 0x20) == 0
    }
}

// Default UART at 0x9000000 (QEMU ARM64 virt machine)
static UART: Mutex<Option<UartDriver>> = Mutex::new(None);

pub fn init(baud_rate: u32) {
    let uart = UartDriver::new(0x9000_000, baud_rate);
    uart.init();
    
    let mut uart_guard = UART.lock();
    *uart_guard = Some(uart);
}

/// Write a single character
pub fn putc(c: u8) {
    let uart_guard = UART.lock();
    if let Some(ref uart) = *uart_guard {
        uart.putc(c);
    }
}

/// Write string
pub fn puts(s: &str) {
    for c in s.bytes() {
        putc(c);
    }
}

/// Read a single character
pub fn getc() -> u8 {
    let uart_guard = UART.lock();
    if let Some(ref uart) = *uart_guard {
        uart.getc()
    } else {
        0
    }
}

/// Check if data is available
pub fn is_readable() -> bool {
    let uart_guard = UART.lock();
    if let Some(ref uart) = *uart_guard {
        uart.is_rx_ready()
    } else {
        false
    }
}

/// Write line (with newline)
pub fn println(s: &str) {
    puts(s);
    puts("\r\n");
}
