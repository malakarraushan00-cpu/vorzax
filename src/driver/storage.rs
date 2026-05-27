/// Block storage driver for ARM64.
///
/// Provides a larger virtual block device and a small write-through cache.

use spin::Mutex;

/// Block size (typically 4096 bytes)
pub const BLOCK_SIZE: usize = 4096;

/// Increased virtual disk size: 16,384 blocks * 4 KiB = 64 MiB.
pub const DEFAULT_TOTAL_BLOCKS: u64 = 16_384;
pub const DEFAULT_BASE_ADDR: u64 = 0x6000_0000;
pub const CACHE_ENTRIES: usize = 16;

/// Storage device trait
pub trait StorageDevice {
    fn read_block(&self, block_num: u64, buffer: &mut [u8]) -> Result<(), StorageError>;
    fn write_block(&self, block_num: u64, buffer: &[u8]) -> Result<(), StorageError>;
    fn total_blocks(&self) -> u64;
}

/// Storage errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    InvalidBlock,
    BufferTooSmall,
    ReadError,
    WriteError,
    Timeout,
}

#[derive(Debug, Clone, Copy)]
pub struct StorageGeometry {
    pub block_size: usize,
    pub total_blocks: u64,
    pub total_bytes: u64,
    pub base_addr: u64,
}

/// Virtual block device
#[derive(Clone, Copy)]
pub struct VirtualBlockDevice {
    total_blocks: u64,
    base_addr: u64,
}

impl VirtualBlockDevice {
    pub const fn new(total_blocks: u64, base_addr: u64) -> Self {
        VirtualBlockDevice {
            total_blocks,
            base_addr,
        }
    }

    pub fn geometry(&self) -> StorageGeometry {
        StorageGeometry {
            block_size: BLOCK_SIZE,
            total_blocks: self.total_blocks,
            total_bytes: self.total_blocks * BLOCK_SIZE as u64,
            base_addr: self.base_addr,
        }
    }
}

impl StorageDevice for VirtualBlockDevice {
    fn read_block(&self, block_num: u64, buffer: &mut [u8]) -> Result<(), StorageError> {
        if block_num >= self.total_blocks {
            return Err(StorageError::InvalidBlock);
        }

        if buffer.len() < BLOCK_SIZE {
            return Err(StorageError::BufferTooSmall);
        }

        let src = (self.base_addr + block_num * BLOCK_SIZE as u64) as *const u8;
        unsafe {
            for (i, byte) in buffer.iter_mut().take(BLOCK_SIZE).enumerate() {
                *byte = *src.add(i);
            }
        }

        Ok(())
    }

    fn write_block(&self, block_num: u64, buffer: &[u8]) -> Result<(), StorageError> {
        if block_num >= self.total_blocks {
            return Err(StorageError::InvalidBlock);
        }

        if buffer.len() < BLOCK_SIZE {
            return Err(StorageError::BufferTooSmall);
        }

        let dst = (self.base_addr + block_num * BLOCK_SIZE as u64) as *mut u8;
        unsafe {
            for (i, byte) in buffer.iter().take(BLOCK_SIZE).enumerate() {
                *dst.add(i) = *byte;
            }
        }

        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        self.total_blocks
    }
}

/// Storage cache entry
#[derive(Clone, Copy)]
struct CacheEntry {
    block_num: u64,
    data: [u8; BLOCK_SIZE],
    valid: bool,
    last_used: u64,
}

impl CacheEntry {
    pub const fn empty() -> Self {
        CacheEntry {
            block_num: 0,
            data: [0; BLOCK_SIZE],
            valid: false,
            last_used: 0,
        }
    }
}

/// Storage cache (write-through with LRU replacement)
pub struct StorageCache {
    device: Option<VirtualBlockDevice>,
    cache: [CacheEntry; CACHE_ENTRIES],
    clock: u64,
    cache_hits: u64,
    cache_misses: u64,
    reads: u64,
    writes: u64,
}

impl StorageCache {
    pub const fn new() -> Self {
        StorageCache {
            device: None,
            cache: [CacheEntry::empty(); CACHE_ENTRIES],
            clock: 0,
            cache_hits: 0,
            cache_misses: 0,
            reads: 0,
            writes: 0,
        }
    }

    pub fn init_device(&mut self, device: VirtualBlockDevice) {
        self.device = Some(device);
        self.cache = [CacheEntry::empty(); CACHE_ENTRIES];
        self.clock = 0;
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.reads = 0;
        self.writes = 0;
    }

    pub fn read(&mut self, block_num: u64, buffer: &mut [u8]) -> Result<(), StorageError> {
        if buffer.len() < BLOCK_SIZE {
            return Err(StorageError::BufferTooSmall);
        }

        self.reads += 1;
        self.clock += 1;

        if let Some(index) = self.find_cache(block_num) {
            self.cache_hits += 1;
            self.cache[index].last_used = self.clock;
            buffer[..BLOCK_SIZE].copy_from_slice(&self.cache[index].data);
            return Ok(());
        }

        self.cache_misses += 1;
        let device = self.device.ok_or(StorageError::ReadError)?;
        device.read_block(block_num, buffer)?;

        let index = self.evict_index();
        self.cache[index] = CacheEntry {
            block_num,
            data: copy_block(buffer),
            valid: true,
            last_used: self.clock,
        };

        Ok(())
    }

    pub fn write(&mut self, block_num: u64, buffer: &[u8]) -> Result<(), StorageError> {
        if buffer.len() < BLOCK_SIZE {
            return Err(StorageError::BufferTooSmall);
        }

        self.writes += 1;
        self.clock += 1;

        let device = self.device.ok_or(StorageError::WriteError)?;
        device.write_block(block_num, buffer)?;

        let index = self.find_cache(block_num).unwrap_or_else(|| self.evict_index());
        self.cache[index] = CacheEntry {
            block_num,
            data: copy_block(buffer),
            valid: true,
            last_used: self.clock,
        };

        Ok(())
    }

    pub fn geometry(&self) -> StorageGeometry {
        self.device
            .map(|device| device.geometry())
            .unwrap_or(StorageGeometry {
                block_size: BLOCK_SIZE,
                total_blocks: 0,
                total_bytes: 0,
                base_addr: 0,
            })
    }

    fn find_cache(&self, block_num: u64) -> Option<usize> {
        self.cache
            .iter()
            .position(|entry| entry.valid && entry.block_num == block_num)
    }

    fn evict_index(&self) -> usize {
        if let Some(index) = self.cache.iter().position(|entry| !entry.valid) {
            return index;
        }

        self.cache
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

static STORAGE: Mutex<StorageCache> = Mutex::new(StorageCache::new());

pub fn init() {
    let mut storage = STORAGE.lock();
    let device = VirtualBlockDevice::new(DEFAULT_TOTAL_BLOCKS, DEFAULT_BASE_ADDR);
    storage.init_device(device);
}

/// Read block from storage
pub fn read_block(block_num: u64, buffer: &mut [u8]) -> Result<(), StorageError> {
    let mut storage = STORAGE.lock();
    storage.read(block_num, buffer)
}

/// Write block to storage
pub fn write_block(block_num: u64, buffer: &[u8]) -> Result<(), StorageError> {
    let mut storage = STORAGE.lock();
    storage.write(block_num, buffer)
}

pub fn get_geometry() -> StorageGeometry {
    STORAGE.lock().geometry()
}

/// Get statistics
pub struct StorageStats {
    pub total_blocks: u64,
    pub total_bytes: u64,
    pub cache_entries: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub reads: u64,
    pub writes: u64,
}

pub fn get_stats() -> StorageStats {
    let storage = STORAGE.lock();
    let geometry = storage.geometry();
    StorageStats {
        total_blocks: geometry.total_blocks,
        total_bytes: geometry.total_bytes,
        cache_entries: CACHE_ENTRIES,
        cache_hits: storage.cache_hits,
        cache_misses: storage.cache_misses,
        reads: storage.reads,
        writes: storage.writes,
    }
}

fn copy_block(buffer: &[u8]) -> [u8; BLOCK_SIZE] {
    let mut data = [0u8; BLOCK_SIZE];
    data.copy_from_slice(&buffer[..BLOCK_SIZE]);
    data
}
