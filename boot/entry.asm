/* ARM64 UEFI Boot Entry Assembly
 * 
 * This is the entry point for the Vorzax kernel.
 * It handles:
 * - Bootloader handoff
 * - Stack setup
 * - Exception vectors initialization
 * - Jump to Rust main
 */

.text
.align 16

/* ARM64 boot header for UEFI compliance */
.section .boot.header
    /* UEFI Image Header */
    .long 0x5A4D              /* MZ signature (DOS header) */
    .long 0x00000040          /* PE offset */
    .zero 0x3C
    .long 0x00000040          /* PE header offset */

/* UEFI PE header */
.section .boot.entry
    .long 0x4550              /* PE signature */
    .short 0x00B1             /* Machine type: ARM64 */
    
.align 0x1000

/* Entry point - called by bootloader */
.global _start
.type _start, @function
_start:
    /* X0-X3 contain: Handle, SystemTable, reserved, reserved */
    /* We are in EL2 or EL1, with MMU disabled */
    
    /* Save UEFI system table pointer */
    ldr x4, =uefi_system_table
    str x1, [x4]
    
    /* Initialize SCTLR_EL1 - disable caches initially */
    mov x5, #0
    msr SCTLR_EL1, x5
    
    /* Set up exception level and stack */
    mrs x6, CurrentEL
    and x6, x6, #0xC
    cmp x6, #0x8              /* Check if in EL2 */
    b.ne setup_el1
    
    /* Drop from EL2 to EL1 */
    mov x6, #0x3c5            /* EL1h mode, IRQ/FIQ disabled */
    msr SPSR_EL2, x6
    adr x6, setup_el1
    msr ELR_EL2, x6
    eret

.align 8
setup_el1:
    /* Set SP_EL1 - kernel stack at 0xFFFF000000400000 */
    mov x7, #0x0000
    movk x7, #0x0004, lsl #16
    movk x7, #0xFFFF, lsl #32
    msr SP_EL1, x7
    
    /* Load VBAR_EL1 - exception vector base */
    ldr x8, =exception_vectors
    msr VBAR_EL1, x8
    
    /* Initialize page tables (identity mapping for boot) */
    bl setup_mmu
    
    /* Enable MMU and caches */
    mrs x9, SCTLR_EL1
    orr x9, x9, #0x01         /* M bit - enable MMU */
    orr x9, x9, #0x04         /* C bit - enable data cache */
    orr x9, x9, #0x1000       /* I bit - enable instruction cache */
    msr SCTLR_EL1, x9
    isb
    
    /* Jump to Rust entry point */
    ldr x10, =_start_rust
    blr x10
    
    /* Halt if we return */
    b halt

/* Setup MMU - simple identity mapping for first 2GB */
setup_mmu:
    /* This is simplified - assumes pre-allocated page tables */
    /* In production, we'd allocate and initialize L0-L3 tables */
    
    /* Set TTBR0_EL1 to kernel page table */
    mov x11, #0x1000000
    msr TTBR0_EL1, x11
    
    /* Set TTBR1_EL1 */
    mov x11, #0x1100000
    msr TTBR1_EL1, x11
    
    /* Configure TCR_EL1 */
    mov x11, #0x00b5_1a59     /* 4KB pages, 39-bit VA */
    msr TCR_EL1, x11
    
    /* Configure MAIR_EL1 (memory attributes) */
    mov x11, #0x00b4_3210     /* Normal, Device, Strongly-ordered */
    msr MAIR_EL1, x11
    
    /* ISB to ensure settings take effect */
    isb
    
    ret

/* Exception vectors - must be 2KB aligned */
.align 0x800
.section .vectors
.global exception_vectors
exception_vectors:
    /* Current EL with SP_EL0 */
    /* Synchronous */
    .align 7
    sub sp, sp, #0x100
    b handle_sync_el1
    
    /* IRQ */
    .align 7
    sub sp, sp, #0x100
    b handle_irq_el1
    
    /* FIQ */
    .align 7
    sub sp, sp, #0x100
    b handle_fiq_el1
    
    /* System Error */
    .align 7
    sub sp, sp, #0x100
    b handle_serror_el1

    /* Current EL with SP_ELx */
    /* Synchronous */
    .align 7
    sub sp, sp, #0x100
    b handle_sync_el1
    
    /* IRQ */
    .align 7
    sub sp, sp, #0x100
    b handle_irq_el1
    
    /* FIQ */
    .align 7
    sub sp, sp, #0x100
    b handle_fiq_el1
    
    /* System Error */
    .align 7
    sub sp, sp, #0x100
    b handle_serror_el1

/* Exception handlers */
handle_sync_el1:
    /* Save context */
    stp x0, x1, [sp]
    mrs x0, ESR_EL1
    mrs x1, FAR_EL1
    /* Call Rust handler */
    ldr x2, =handle_sync_exception
    blr x2
    /* Restore and return */
    ldp x0, x1, [sp]
    add sp, sp, #0x100
    eret

handle_irq_el1:
    /* Save context and call IRQ handler */
    stp x0, x1, [sp]
    ldr x2, =handle_irq
    blr x2
    ldp x0, x1, [sp]
    add sp, sp, #0x100
    eret

handle_fiq_el1:
    ldr x2, =handle_fiq
    blr x2
    eret

handle_serror_el1:
    ldr x2, =handle_serror
    blr x2
    eret

halt:
    wfe
    b halt

/* Data section */
.data
uefi_system_table:
    .quad 0

/* Symbol for Rust entry */
.extern _start_rust
