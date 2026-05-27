/// ARM64 Interrupt and Exception Handling
/// 
/// Sets up exception vectors and interrupt handlers

use core::ptr;

/// Exception vector table base address (must be 2KB aligned)
const EXCEPTION_VECTOR_BASE: u64 = 0xFFFF_0000_0001_0000;

/// ARM64 Exception Class
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum ExceptionClass {
    Synchronous = 0b000000,
    IrqInterrupt = 0b100001,
    FiqInterrupt = 0b100010,
    SerrorInterrupt = 0b100011,
}

/// Exception type
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum ExceptionType {
    Unknown,
    WfiWfe,
    McrrMrrc,
    LdcStc,
    LdmStm,
    Ldr,
    FloatingPoint,
    ImproperUseOfSveLdr,
    SveAccessTrap,
    PacFailure,
    BranchTarget,
    IllegalExecution,
    Smc,
    HvcTrap,
    SmeAccess,
    RmeTrap,
    GranuleProtectionCheck,
}

/// Exception info from ESR_EL1
#[derive(Debug, Clone, Copy)]
pub struct ExceptionInfo {
    pub exception_class: u64,
    pub instruction_length: u64,
    pub fault_address: u64,
}

impl ExceptionInfo {
    pub fn from_esr(esr: u64, far: u64) -> Self {
        ExceptionInfo {
            exception_class: (esr >> 26) & 0x3F,
            instruction_length: (esr >> 25) & 0x01,
            fault_address: far,
        }
    }
}

pub fn init() {
    setup_exception_vectors();
}

/// Set up ARM64 exception vectors
pub unsafe fn setup_exception_vectors() {
    // Set VBAR_EL1 to exception vector base
    asm!("msr VBAR_EL1, {0}", in(reg) EXCEPTION_VECTOR_BASE);
    
    // ISB to ensure register update
    asm!("isb");
}

/// Synchronous exception handler (called from asm vector)
#[no_mangle]
pub extern "C" fn handle_sync_exception(esr: u64, far: u64, lr: u64) {
    let exc_info = ExceptionInfo::from_esr(esr, far);
    
    match exc_info.exception_class {
        0b000000 => handle_unknown(esr),
        0b010001 => handle_svc(esr),  // SVC call
        0b011001 => handle_data_abort(esr, far),
        0b011000 => handle_instruction_abort(esr, far),
        _ => handle_unknown(esr),
    }
}

/// Synchronous exception for unknown fault
fn handle_unknown(esr: u64) {
    let exc_class = (esr >> 26) & 0x3F;
    crate::log!("Unknown Exception: ESR={:x}, Class={:b}\n", esr, exc_class);
}

/// SVC (Supervisor Call) handler
fn handle_svc(esr: u64) {
    let immediate = esr & 0xFFFF;
    crate::log!("SVC Call: IMM={}\n", immediate);
}

/// Data Abort handler
fn handle_data_abort(esr: u64, far: u64) {
    crate::log!("Data Abort at {:x}, ESR={:x}\n", far, esr);
}

/// Instruction Abort handler
fn handle_instruction_abort(esr: u64, far: u64) {
    crate::log!("Instruction Abort at {:x}, ESR={:x}\n", far, esr);
}

/// IRQ handler
#[no_mangle]
pub extern "C" fn handle_irq() {
    // Get interrupt controller status
    // For now, acknowledge any pending interrupt
    unsafe {
        asm!("msr daifclr, #2"); // Clear IRQ mask
    }
}

/// FIQ handler
#[no_mangle]
pub extern "C" fn handle_fiq() {
    crate::log!("FIQ Received\n");
}

/// System Error handler
#[no_mangle]
pub extern "C" fn handle_serror() {
    crate::log!("System Error!\n");
    loop {
        unsafe { asm!("wfe"); }
    }
}

/// Enable interrupts (set DAIF.I to 0)
pub unsafe fn enable_irq() {
    asm!("msr daifclr, #2");
}

/// Disable interrupts (set DAIF.I to 1)
pub unsafe fn disable_irq() {
    asm!("msr daifset, #2");
}

/// Check if interrupts are enabled
pub fn irq_enabled() -> bool {
    let daif: u64;
    unsafe {
        asm!("mrs {0}, DAIF", out(reg) daif);
    }
    (daif & 0x80) == 0
}
