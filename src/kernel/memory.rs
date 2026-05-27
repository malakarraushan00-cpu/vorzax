/// Memory management for ARM64
/// 
/// Implements virtual memory, paging, and heap allocation

use core::ptr;
use spin::Mutex;

/// Page size (4KB)
const PAGE_SIZE: usize = 0x1000;

/// Page table entry
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PageTableEntry {
    descriptor: u64,
}

impl PageTableEntry {
    pub fn new(phys_addr: u64, flags: u64) -> Self {
        PageTableEntry {
            descriptor: (phys_addr & 0xFFFF_F000) | flags,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        (self.descriptor & 0x01) != 0
    }
    
    pub fn get_address(&self) -> u64 {
        self.descriptor & 0xFFFF_F000
    }
}

/// ARM64 Page table (512 entries × 8 bytes)
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry { descriptor: 0 }; 512],
        }
    }
    
    pub fn get_entry(&self, index: usize) -> PageTableEntry {
        self.entries[index]
    }
    
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.entries[index] = entry;
    }
}

/// Virtual address structure for ARM64
#[derive(Clone, Copy, Debug)]
pub struct VirtAddr {
    addr: u64,
}

impl VirtAddr {
    pub fn new(addr: u64) -> Self {
        VirtAddr { addr }
    }
    
    pub fn l0_index(&self) -> usize {
        ((self.addr >> 39) & 0x1FF) as usize
    }
    
    pub fn l1_index(&self) -> usize {
        ((self.addr >> 30) & 0x1FF) as usize
    }
    
    pub fn l2_index(&self) -> usize {
        ((self.addr >> 21) & 0x1FF) as usize
    }
    
    pub fn l3_index(&self) -> usize {
        ((self.addr >> 12) & 0x1FF) as usize
    }
    
    pub fn page_offset(&self) -> usize {
        (self.addr & 0xFFF) as usize
    }
}

/// Simple bump allocator for kernel heap
pub struct BumpAllocator {
    start: u64,
    current: u64,
    end: u64,
}

impl BumpAllocator {
    pub fn new(start: u64, size: u64) -> Self {
        BumpAllocator {
            start,
            current: start,
            end: start + size,
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<u64> {
        let aligned_size = (size + 0xF) & !0xF; // 16-byte align
        if self.current + aligned_size as u64 <= self.end {
            let ptr = self.current;
            self.current += aligned_size as u64;
            Some(ptr)
        } else {
            None
        }
    }
    
    pub fn free(&mut self, _ptr: u64, _size: usize) {
        // Bump allocators don't support freeing
    }
}

static HEAP: Mutex<BumpAllocator> = Mutex::new(BumpAllocator {
    start: 0,
    current: 0,
    end: 0,
});

pub fn init() {
    // Initialize heap at 0xFFFF_0000_0000_0000 (typical kernel space)
    // Size: 256MB for initial setup
    let mut heap = HEAP.lock();
    *heap = BumpAllocator::new(0xFFFF_0000_0000_0000, 256 * 1024 * 1024);
}

/// Allocate kernel memory
pub fn allocate(size: usize) -> Option<u64> {
    let mut heap = HEAP.lock();
    heap.allocate(size)
}

/// Map virtual address to physical address
pub fn map_page(virt: u64, phys: u64, flags: u64) {
    // Get L0 page table from TTBR0_EL1 or TTBR1_EL1
    let mut ttbr: u64;
    unsafe {
        asm!("mrs {0}, TTBR0_EL1", out(reg) ttbr);
    }
    
    let vaddr = VirtAddr::new(virt);
    let l0_idx = vaddr.l0_index();
    
    let l0_table = ttbr as *mut PageTable;
    
    // Walk page tables (simplified for single-level)
    unsafe {
        let entry = PageTableEntry::new(phys, flags | 0x03); // Valid + page descriptor
        (*l0_table).set_entry(l0_idx, entry);
    }
}

/// Unmap virtual address
pub fn unmap_page(virt: u64) {
    let mut ttbr: u64;
    unsafe {
        asm!("mrs {0}, TTBR0_EL1", out(reg) ttbr);
    }
    
    let vaddr = VirtAddr::new(virt);
    let l0_idx = vaddr.l0_index();
    
    let l0_table = ttbr as *mut PageTable;
    
    unsafe {
        let entry = PageTableEntry { descriptor: 0 };
        (*l0_table).set_entry(l0_idx, entry);
    }
}

/// Get physical address for virtual address
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    let mut ttbr: u64;
    unsafe {
        asm!("mrs {0}, TTBR0_EL1", out(reg) ttbr);
    }
    
    let vaddr = VirtAddr::new(virt);
    let l0_idx = vaddr.l0_index();
    
    let l0_table = ttbr as *const PageTable;
    
    unsafe {
        let entry = (*l0_table).get_entry(l0_idx);
        if entry.is_valid() {
            let offset = vaddr.page_offset();
            Some(entry.get_address() + offset as u64)
        } else {
            None
        }
    }
}
