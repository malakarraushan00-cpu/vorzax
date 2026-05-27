/// Process management for ARM64.
///
/// Supports fixed-table multi-process creation, runnable queue handoff, and a
/// fast "tab" burst helper for stress-opening 100 lightweight processes.

use spin::Mutex;

use crate::kernel::scheduler;

pub const MAX_PROCESSES: usize = 256;
pub const PROCESS_NAME_MAX: usize = 32;
pub const TAB_BURST_PER_SECOND: usize = 100;

/// Process states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Created,
    Ready,
    Running,
    Waiting,
    Terminated,
}

/// Process creation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    InvalidEntry,
    InvalidCount,
    ProcessTableFull,
    SchedulerFull,
}

/// ARM64 register context for context switching
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegisterContext {
    pub x: [u64; 31], // X0-X30 (X31 is SP)
    pub sp: u64,      // Stack pointer (SP_EL0)
    pub pc: u64,      // Program counter
    pub pstate: u64,  // Processor state
}

impl RegisterContext {
    pub fn new(entry: u64, stack: u64) -> Self {
        RegisterContext {
            x: [0; 31],
            sp: stack,
            pc: entry,
            pstate: 0x0000_0000, // EL0t, IRQ/FIQ disabled
        }
    }
}

/// Process control block
#[derive(Clone, Copy)]
pub struct Process {
    pub pid: u64,
    pub parent_pid: Option<u64>,
    pub state: ProcessState,
    pub registers: RegisterContext,
    pub priority: u8,
    pub time_slice: u64,
    pub time_consumed: u64,
    pub created_at_ticks: u64,
    pub name: [u8; PROCESS_NAME_MAX],
    pub name_len: usize,
}

impl Process {
    pub fn new(pid: u64, entry: u64, priority: u8, name: &str) -> Self {
        let stack = 0x8000_0000u64 + (pid << 12); // 4KB stack per process
        Process {
            pid,
            parent_pid: scheduler::current_process(),
            state: ProcessState::Created,
            registers: RegisterContext::new(entry, stack),
            priority,
            time_slice: 10,
            time_consumed: 0,
            created_at_ticks: scheduler::ticks(),
            name: copy_name(name),
            name_len: core::cmp::min(name.len(), PROCESS_NAME_MAX),
        }
    }

    pub fn name_bytes(&self) -> &[u8] {
        &self.name[..self.name_len]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpawnReport {
    pub requested: usize,
    pub spawned: usize,
    pub first_pid: Option<u64>,
    pub last_pid: Option<u64>,
    pub failed: Option<ProcessError>,
}

pub struct ProcessStats {
    pub total_slots: usize,
    pub used_slots: usize,
    pub ready: usize,
    pub running: usize,
    pub waiting: usize,
    pub terminated: usize,
}

static NEXT_PID: Mutex<u64> = Mutex::new(1);
static PROCESSES: Mutex<[Option<Process>; MAX_PROCESSES]> = Mutex::new([None; MAX_PROCESSES]);

pub fn init() {
    let mut next_pid = NEXT_PID.lock();
    let mut processes = PROCESSES.lock();

    *next_pid = 1;
    *processes = [None; MAX_PROCESSES];
}

/// Create a runnable process and enqueue it for scheduling.
pub fn create_process(entry: u64, priority: u8) -> Option<u64> {
    spawn_process(entry, priority, "process").ok()
}

/// Create a runnable process with a name.
pub fn spawn_process(entry: u64, priority: u8, name: &str) -> Result<u64, ProcessError> {
    if entry == 0 {
        return Err(ProcessError::InvalidEntry);
    }

    let pid = allocate_pid();
    let mut process = Process::new(pid, entry, priority, name);
    process.state = ProcessState::Ready;

    {
        let mut processes = PROCESSES.lock();
        let slot = processes
            .iter_mut()
            .find(|process| process.is_none())
            .ok_or(ProcessError::ProcessTableFull)?;
        *slot = Some(process);
    }

    if scheduler::add_ready_process(pid).is_err() {
        remove_process(pid);
        return Err(ProcessError::SchedulerFull);
    }

    Ok(pid)
}

/// Spawn up to 100 process-backed tabs in one burst.
pub fn open_100_tabs(entry: u64) -> SpawnReport {
    spawn_process_tabs(entry, TAB_BURST_PER_SECOND)
}

/// Spawn process-backed tabs. The request is capped at 100 per call.
pub fn spawn_process_tabs(entry: u64, requested_tabs: usize) -> SpawnReport {
    if requested_tabs == 0 {
        return SpawnReport {
            requested: requested_tabs,
            spawned: 0,
            first_pid: None,
            last_pid: None,
            failed: Some(ProcessError::InvalidCount),
        };
    }

    let capped = core::cmp::min(requested_tabs, TAB_BURST_PER_SECOND);
    let mut report = SpawnReport {
        requested: requested_tabs,
        spawned: 0,
        first_pid: None,
        last_pid: None,
        failed: None,
    };

    for _ in 0..capped {
        match spawn_process(entry, 5, "tab") {
            Ok(pid) => {
                if report.first_pid.is_none() {
                    report.first_pid = Some(pid);
                }
                report.last_pid = Some(pid);
                report.spawned += 1;
            }
            Err(err) => {
                report.failed = Some(err);
                break;
            }
        }
    }

    report
}

/// Get process by ID
pub fn get_process(pid: u64) -> Option<Process> {
    let processes = PROCESSES.lock();
    processes.iter().find_map(|slot| {
        if let Some(process) = slot {
            if process.pid == pid {
                return Some(*process);
            }
        }
        None
    })
}

pub fn set_process_state(pid: u64, state: ProcessState) -> bool {
    let mut processes = PROCESSES.lock();
    if let Some(process) = processes.iter_mut().find_map(|slot| {
        if let Some(process) = slot {
            if process.pid == pid {
                return Some(process);
            }
        }
        None
    }) {
        process.state = state;
        return true;
    }
    false
}

/// Terminate process
pub fn terminate_process(pid: u64) -> bool {
    set_process_state(pid, ProcessState::Terminated)
}

/// Remove a process table entry completely.
pub fn remove_process(pid: u64) -> bool {
    let mut processes = PROCESSES.lock();
    if let Some(index) = processes.iter().position(|slot| {
        if let Some(process) = slot {
            return process.pid == pid;
        }
        false
    }) {
        processes[index] = None;
        return true;
    }
    false
}

/// Get all ready processes
pub fn get_ready_processes() -> [Option<Process>; MAX_PROCESSES] {
    let processes = PROCESSES.lock();
    let mut ready = [None; MAX_PROCESSES];
    let mut index = 0;

    for slot in processes.iter() {
        if let Some(process) = slot {
            if process.state == ProcessState::Ready && index < MAX_PROCESSES {
                ready[index] = Some(*process);
                index += 1;
            }
        }
    }

    ready
}

pub fn get_stats() -> ProcessStats {
    let processes = PROCESSES.lock();
    let mut stats = ProcessStats {
        total_slots: MAX_PROCESSES,
        used_slots: 0,
        ready: 0,
        running: 0,
        waiting: 0,
        terminated: 0,
    };

    for slot in processes.iter() {
        if let Some(process) = slot {
            stats.used_slots += 1;
            match process.state {
                ProcessState::Created => {}
                ProcessState::Ready => stats.ready += 1,
                ProcessState::Running => stats.running += 1,
                ProcessState::Waiting => stats.waiting += 1,
                ProcessState::Terminated => stats.terminated += 1,
            }
        }
    }

    stats
}

fn allocate_pid() -> u64 {
    let mut next_pid = NEXT_PID.lock();
    let pid = *next_pid;
    *next_pid += 1;
    pid
}

fn copy_name(name: &str) -> [u8; PROCESS_NAME_MAX] {
    let mut buffer = [0u8; PROCESS_NAME_MAX];
    let bytes = name.as_bytes();
    let len = core::cmp::min(bytes.len(), PROCESS_NAME_MAX);
    for i in 0..len {
        buffer[i] = bytes[i];
    }
    buffer
}
