# Vorzax - ARM64 Operating System

A complete, from-scratch ARM64 operating system written in Rust with a full GUI desktop environment.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    GUI Desktop                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Windows    │  │  Applications│  │   Widgets    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                 Graphics & Rendering                    │
│              Framebuffer Driver (1920x1080)             │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                   Kernel Services                       │
│  ┌──────────┐ ┌────────────┐ ┌──────────┐ ┌─────────┐  │
│  │Scheduler │ │   Memory   │ │Interrupts│ │Process  │  │
│  │          │ │ Management │ │ Handler  │ │Manager  │  │
│  └──────────┘ └────────────┘ └──────────┘ └─────────┘  │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                    Device Drivers                       │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ ┌──────────┐  │
│  │   UART   │ │Framebuffer│ │ Storage │ │  Input   │  │
│  └──────────┘ └──────────┘ └─────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                   ARM64 Hardware (aarch64)              │
│              UEFI Firmware Interface                    │
└─────────────────────────────────────────────────────────┘
```

## Features

- **ARM64 Architecture**: Full aarch64 support with UEFI boot
- **Kernel Subsystems**:
  - Multi-process management with context switching
  - Virtual memory management with paging
  - Interrupt and exception handling
  - Preemptive multitasking scheduler (priority-based, Round-Robin)
  - User registration, login sessions, and account management
  
- **Device Drivers**:
  - Framebuffer (1920x1080 @ 32-bit RGBA)
  - UART serial communication
  - 64 MiB block storage interface
  - Timer, keyboard, mouse, and unified input drivers
  
- **GUI Framework**:
  - Desktop with taskbar
  - Windowing system with draw buffers
  - Event system (keyboard, mouse, window events)
  - Application framework
  - Widget library (buttons, text fields, panels)

## System Requirements

- Rust 1.70+ with `aarch64-unknown-none` target
- Cross-compiler toolchain for ARM64
- QEMU or ARM64 hardware for execution
- 2GB+ RAM, 1920x1080 display

## Building

### Prerequisites

```bash
# Install Rust and ARM64 target
rustup update
rustup target add aarch64-unknown-none

# Install ARM64 cross-compiler (Ubuntu/Debian)
sudo apt-get install gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu

# For macOS:
# brew install aarch64-elf-gcc

# For Windows (native):
# Download MSVC ARM64 build tools or use mingw-w64 with aarch64 support
```

### Build Steps

```bash
cd vorzax

# Debug build
cargo build --target aarch64-unknown-none

# Release build (optimized)
cargo build --release --target aarch64-unknown-none

# Build with UEFI feature
cargo build --features uefi --target aarch64-unknown-none
```

### Output

The compiled kernel is located at:
- Debug: `target/aarch64-unknown-none/debug/vorzax`
- Release: `target/aarch64-unknown-none/release/vorzax`

## Running

### With QEMU

```bash
# Requires QEMU with ARM64 support
qemu-system-aarch64 -machine virt -cpu cortex-a72 \
  -m 2G \
  -bios QEMU_EFI.fd \
  -kernel target/aarch64-unknown-none/release/vorzax \
  -display gtk \
  -smp 4
```

### On ARM64 Hardware

1. Prepare SD card with UEFI bootloader
2. Copy compiled kernel to EFI System Partition
3. Add boot configuration
4. Power on and boot

## Project Structure

```
vorzax/
├── Cargo.toml                 # Project manifest
├── build.rs                   # Build script
├── README.md                  # This file
├── boot/
│   ├── linker.ld             # Linker script for ARM64
│   └── entry.asm             # Assembly bootloader entry
├── src/
│   ├── main.rs               # Kernel entry point
│   ├── lib.rs                # Library root
│   ├── boot/
│   │   ├── mod.rs            # Boot module
│   │   └── entry.rs          # Rust boot initialization
│   ├── kernel/
│   │   ├── mod.rs            # Kernel module
│   │   ├── process.rs        # Process management
│   │   ├── memory.rs         # Memory management
│   │   ├── interrupt.rs      # Interrupt handling
│   │   └── scheduler.rs      # Task scheduler
│   ├── driver/
│   │   ├── mod.rs            # Driver module
│   │   ├── framebuffer.rs    # Graphics driver
│   │   ├── uart.rs           # Serial driver
│   │   ├── storage.rs        # Storage driver
│   │   ├── timer.rs          # ARM generic timer driver
│   │   ├── keyboard.rs       # Keyboard input driver
│   │   ├── mouse.rs          # Mouse input driver
│   │   └── input.rs          # Unified input facade
│   ├── gui/
│   │   ├── mod.rs            # GUI module
│   │   ├── desktop.rs        # Desktop environment
│   │   ├── window.rs         # Window manager
│   │   ├── app.rs            # Application framework
│   │   ├── widget.rs         # UI widgets
│   │   └── event.rs          # Event system
│   └── util/
│       └── mod.rs            # Utilities
└── target/                   # Build output
```

## Key Modules

### Boot (src/boot/)
- UEFI firmware interface initialization
- Memory mapping setup
- Jump to kernel entry point
- Stack and heap initialization

### Kernel (src/kernel/)

**process.rs**: Process control block, creation, termination
```rust
pub struct Process {
    pid: u64,
    state: ProcessState,
    registers: RegisterContext,
    name: [u8; 32],
}

pub fn spawn_process(entry: u64, priority: u8, name: &str) -> Result<u64, ProcessError>;
pub fn open_100_tabs(entry: u64) -> SpawnReport;
```

**memory.rs**: Virtual memory, page tables, heap allocator
```rust
pub struct VirtualMemory {
    page_table: *mut PageTable,
    heap: BumpAllocator,
}
```

**interrupt.rs**: Exception and interrupt vector setup
```rust
pub unsafe fn setup_exception_vectors() { ... }
```

**scheduler.rs**: Context switching, priority queue, Round-Robin
```rust
pub struct Scheduler {
    ready_queue: PriorityQueue<Process>,
    current: Option<Process>,
}
```

**user.rs**: User accounts, login sessions, and management
```rust
pub fn register_user(username: &str, password: &str) -> Result<u32, UserError>;
pub fn login(username: &str, password: &str) -> Result<Session, LoginError>;
pub fn set_user_status(user_id: u32, status: UserStatus) -> Result<(), UserError>;
pub fn reset_password_as_admin(user_id: u32, new_password: &str) -> Result<(), UserError>;
```

Default bootstrap login:
- Username: `admin`
- Password: `admin123`

### Drivers (src/driver/)

**framebuffer.rs**: Linear framebuffer, pixel drawing
- Resolution: 1920x1080
- Color format: 32-bit RGBA
- Double-buffering support

**uart.rs**: Serial port communication
- Baud rate configuration
- TX/RX buffers
- Interrupt-driven I/O

**storage.rs**: Block device abstraction
- Read/write operations
- 64 MiB virtual device (16,384 blocks)
- 16-entry write-through cache
- Geometry and statistics APIs

**timer.rs**: ARM generic timer
- Counter frequency detection
- Uptime in milliseconds
- Software tick counter

**keyboard.rs / mouse.rs / input.rs**: Separated input drivers
- Keyboard and mouse event queues
- Unified input polling facade
- Ready for PS/2 or USB HID backends

### GUI (src/gui/)

**desktop.rs**: Main desktop environment
- Taskbar rendering
- Background wallpaper
- Window management

**window.rs**: Window system
- Window creation and destruction
- Event routing
- Compositing and layering

**app.rs**: Application launcher and manager
- Application registry
- Lifecycle management
- Inter-process communication hooks

**widget.rs**: Reusable UI components
- Button, Label, TextBox
- Panel, Scrollbar
- Event handling

**event.rs**: Event dispatch
- Keyboard events
- Mouse events  
- Window events
- Event queue and handlers

## Memory Layout (ARM64)

```
0xFFFF_FFFF_FFFF_FFFF ┌──────────────────┐
                      │   Kernel Space   │
                      │  (Higher Half)   │
                      │                  │
0xFFFF_0000_0000_0000 ├──────────────────┤
                      │   User Space     │
                      │   (Mappable)     │
                      │                  │
0x0000_0000_0000_0000 └──────────────────┘
```

## Development Guidelines

1. **No_std Only**: Kernel code uses `#![no_std]`
2. **ARM64 Calling Convention**: Follow ARM64 AAPCS
3. **Memory Safety**: Use Rust's ownership for safety
4. **Real Code**: No placeholder implementations

## Compilation Details

- **Target**: `aarch64-unknown-none`
- **CPU Features**: ARMv8-A base instruction set
- **Linking**: Custom linker script (`boot/linker.ld`)
- **Panic Handler**: Kernel panic with UART output
- **Entry Point**: `_start` in `boot/entry.asm`

## Testing

```bash
# Build test binary
cargo build --target aarch64-unknown-none

# Run with QEMU (requires modifications for test mode)
# See testing documentation for setup
```

## License

Vorzax OS is provided as-is for educational and development purposes.

## References

- ARM Architecture Reference Manual for ARMv8-A
- UEFI Specification
- Linux Kernel Architecture
- Rust Embedded Handbook
