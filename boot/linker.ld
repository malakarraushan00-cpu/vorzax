/* ARM64 Linker Script for Vorzax OS */

/* Memory layout for ARM64 */
MEMORY {
    BOOT : ORIGIN = 0x40000000, LENGTH = 64K     /* Early boot code */
    KERNEL : ORIGIN = 0xFFFF000000000000, LENGTH = 512M  /* Kernel space */
}

SECTIONS {
    /* Boot section (lower memory, identity mapped) */
    .boot 0x40000000 : {
        KEEP(*(.boot.header))
        KEEP(*(.boot.entry))
        *(.boot.text)
        *(.boot.rodata)
        . = ALIGN(4K);
    } > BOOT

    /* Kernel text section */
    .text 0xFFFF000000000000 : {
        KEEP(*(.text.start))
        *(.text .text.*)
        *(.gnu.linkonce.t.*)
        . = ALIGN(16);
        __text_end = .;
    } > KERNEL

    /* Read-only data section */
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.gnu.linkonce.r.*)
        . = ALIGN(16);
        __rodata_end = .;
    } > KERNEL

    /* Exception vector table (must be 2KB aligned) */
    .vectors : ALIGN(0x800) {
        KEEP(*(.vectors))
        . = ALIGN(0x800);
    } > KERNEL

    /* Data section */
    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.gnu.linkonce.d.*)
        . = ALIGN(16);
        __data_end = .;
    } > KERNEL

    /* BSS section (uninitialized data) */
    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(.gnu.linkonce.b.*)
        *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    } > KERNEL

    /* Heap section (will be managed by allocator) */
    .heap : ALIGN(4K) {
        __heap_start = .;
        . += 256M;
        __heap_end = .;
    } > KERNEL

    /* Stack section */
    .stack : ALIGN(4K) {
        __stack_start = .;
        . += 4M;
        __stack_end = .;
    } > KERNEL

    /* Debugging sections */
    .debug_info 0 : { *(.debug_info) }
    .debug_abbrev 0 : { *(.debug_abbrev) }
    .debug_line 0 : { *(.debug_line) }
    .debug_str 0 : { *(.debug_str) }

    /DISCARD/ : {
        *(.note.GNU-stack)
        *(.gnu_debuglink)
        *(.gnu_debugaltlink)
    }
}

/* Symbols for entry code */
__kernel_start = 0xFFFF000000000000;
__kernel_end = __bss_end;
