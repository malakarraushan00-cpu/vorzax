/// ARM generic timer driver.
///
/// Uses the architectural counter when running on aarch64 and falls back to a
/// software tick counter for non-aarch64 builds.

use spin::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct TimerInfo {
    pub frequency_hz: u64,
    pub boot_counter: u64,
    pub ticks: u64,
}

pub struct TimerDriver {
    frequency_hz: u64,
    boot_counter: u64,
    ticks: u64,
}

impl TimerDriver {
    pub const fn new() -> Self {
        TimerDriver {
            frequency_hz: 0,
            boot_counter: 0,
            ticks: 0,
        }
    }

    pub fn init(&mut self) {
        self.frequency_hz = read_counter_frequency();
        self.boot_counter = read_counter();
        self.ticks = 0;
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    pub fn uptime_millis(&self) -> u64 {
        if self.frequency_hz == 0 {
            return self.ticks;
        }

        let elapsed = read_counter().saturating_sub(self.boot_counter);
        elapsed.saturating_mul(1000) / self.frequency_hz
    }

    pub fn info(&self) -> TimerInfo {
        TimerInfo {
            frequency_hz: self.frequency_hz,
            boot_counter: self.boot_counter,
            ticks: self.ticks,
        }
    }
}

static TIMER: Mutex<TimerDriver> = Mutex::new(TimerDriver::new());

pub fn init() {
    TIMER.lock().init();
}

pub fn tick() {
    TIMER.lock().tick();
}

pub fn uptime_millis() -> u64 {
    TIMER.lock().uptime_millis()
}

pub fn get_info() -> TimerInfo {
    TIMER.lock().info()
}

#[cfg(target_arch = "aarch64")]
fn read_counter_frequency() -> u64 {
    let value: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) value);
    }
    value
}

#[cfg(not(target_arch = "aarch64"))]
fn read_counter_frequency() -> u64 {
    1_000
}

#[cfg(target_arch = "aarch64")]
fn read_counter() -> u64 {
    let value: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) value);
    }
    value
}

#[cfg(not(target_arch = "aarch64"))]
fn read_counter() -> u64 {
    0
}
