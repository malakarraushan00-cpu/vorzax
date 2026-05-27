/// Task Scheduler for ARM64
///
/// Implements preemptive multitasking with a fixed-size circular ready queue.

use spin::Mutex;

use crate::kernel::process::{self, ProcessState, MAX_PROCESSES};

/// Scheduling policy
#[derive(Debug, Clone, Copy)]
pub enum SchedulePolicy {
    RoundRobin,
    PriorityBased,
    Fifo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerError {
    QueueFull,
}

/// Scheduler state
pub struct Scheduler {
    current_process: Option<u64>,
    ready_queue: [Option<u64>; MAX_PROCESSES],
    queue_head: usize,
    queue_tail: usize,
    queue_len: usize,
    time_ticks: u64,
    policy: SchedulePolicy,
}

impl Scheduler {
    pub const fn new(policy: SchedulePolicy) -> Self {
        Scheduler {
            current_process: None,
            ready_queue: [None; MAX_PROCESSES],
            queue_head: 0,
            queue_tail: 0,
            queue_len: 0,
            time_ticks: 0,
            policy,
        }
    }

    pub fn reset(&mut self) {
        self.current_process = None;
        self.ready_queue = [None; MAX_PROCESSES];
        self.queue_head = 0;
        self.queue_tail = 0;
        self.queue_len = 0;
        self.time_ticks = 0;
        self.policy = SchedulePolicy::RoundRobin;
    }

    pub fn enqueue(&mut self, pid: u64) -> Result<(), SchedulerError> {
        if self.queue_len >= MAX_PROCESSES || self.contains(pid) {
            return Err(SchedulerError::QueueFull);
        }

        self.ready_queue[self.queue_tail] = Some(pid);
        self.queue_tail = (self.queue_tail + 1) % MAX_PROCESSES;
        self.queue_len += 1;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<u64> {
        if self.queue_len == 0 {
            return None;
        }

        let pid = self.ready_queue[self.queue_head];
        self.ready_queue[self.queue_head] = None;
        self.queue_head = (self.queue_head + 1) % MAX_PROCESSES;
        self.queue_len -= 1;
        pid
    }

    pub fn is_empty(&self) -> bool {
        self.queue_len == 0
    }

    pub fn len(&self) -> usize {
        self.queue_len
    }

    /// Select next process to run based on policy.
    pub fn select_next(&mut self) -> Option<u64> {
        match self.policy {
            SchedulePolicy::RoundRobin | SchedulePolicy::Fifo => self.dequeue(),
            SchedulePolicy::PriorityBased => self.dequeue(),
        }
    }

    pub fn tick(&mut self) {
        self.time_ticks += 1;
    }

    fn contains(&self, pid: u64) -> bool {
        let mut index = self.queue_head;
        for _ in 0..self.queue_len {
            if self.ready_queue[index] == Some(pid) {
                return true;
            }
            index = (index + 1) % MAX_PROCESSES;
        }
        false
    }
}

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new(SchedulePolicy::RoundRobin));

pub fn init() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.reset();
}

/// Add process to ready queue.
pub fn add_ready_process(pid: u64) -> Result<(), SchedulerError> {
    let mut scheduler = SCHEDULER.lock();
    scheduler.enqueue(pid)
}

/// Get currently running process ID.
pub fn current_process() -> Option<u64> {
    SCHEDULER.lock().current_process
}

pub fn ticks() -> u64 {
    SCHEDULER.lock().time_ticks
}

/// Switch to next process.
pub fn switch_context() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.tick();

    let current_expired = scheduler.time_ticks % 10 == 0;
    let current_alive = scheduler
        .current_process
        .and_then(process::get_process)
        .map(|process| process.state != ProcessState::Terminated)
        .unwrap_or(false);

    if current_alive && !current_expired {
        return;
    }

    if current_alive {
        if let Some(pid) = scheduler.current_process {
            let _ = process::set_process_state(pid, ProcessState::Ready);
            let _ = scheduler.enqueue(pid);
        }
    }

    if let Some(next_pid) = scheduler.select_next() {
        scheduler.current_process = Some(next_pid);
        let _ = process::set_process_state(next_pid, ProcessState::Running);
    } else {
        scheduler.current_process = None;
    }
}

/// Yield current process.
pub fn yield_process() {
    let mut scheduler = SCHEDULER.lock();

    if let Some(pid) = scheduler.current_process {
        scheduler.current_process = None;
        let _ = process::set_process_state(pid, ProcessState::Ready);
        let _ = scheduler.enqueue(pid);
    }

    if let Some(next_pid) = scheduler.select_next() {
        scheduler.current_process = Some(next_pid);
        let _ = process::set_process_state(next_pid, ProcessState::Running);
    }
}

/// Block current process.
pub fn block_process() {
    let mut scheduler = SCHEDULER.lock();
    if let Some(pid) = scheduler.current_process {
        let _ = process::set_process_state(pid, ProcessState::Waiting);
    }
    scheduler.current_process = None;
}

/// Wake process.
pub fn wake_process(pid: u64) -> Result<(), SchedulerError> {
    let _ = process::set_process_state(pid, ProcessState::Ready);
    let mut scheduler = SCHEDULER.lock();
    scheduler.enqueue(pid)
}

/// Get scheduler statistics.
pub struct SchedulerStats {
    pub total_ticks: u64,
    pub processes_ready: usize,
    pub current_process: Option<u64>,
}

pub fn get_stats() -> SchedulerStats {
    let scheduler = SCHEDULER.lock();
    SchedulerStats {
        total_ticks: scheduler.time_ticks,
        processes_ready: scheduler.len(),
        current_process: scheduler.current_process,
    }
}
