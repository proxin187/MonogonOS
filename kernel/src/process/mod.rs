use crate::debug;

use spin::Mutex;

pub static mut PROCESS: Mutex<ProcessHandler> = Mutex::new(ProcessHandler::new());
pub static mut READY: bool = false;

pub const PROCESS_LIMIT: usize = 20;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Running,
    Waiting,
}

// TODO: the problem with the crash when the instruction pointer is for some reason set to 0x296 is
// most likely a result of us not saving r8, r9, r10 and r11
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub rax: i64,
    pub rbx: i64,
    pub rcx: i64,
    pub rdx: i64,
    pub rsp: i64,
    pub rbp: i64,
    pub rsi: i64,
    pub rdi: i64,
    pub rip: i64,
}

impl Context {
    #[inline]
    pub const fn new() -> Context {
        Context {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsp: 0,
            rbp: 0,
            rsi: 0,
            rdi: 0,
            rip: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Process {
    pub state: State,
    pub context: Context,
    pub base: i64,
}

impl Process {
    pub const fn new() -> Process {
        Process {
            state: State::Waiting,
            context: Context::new(),
            base: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base == 0
    }
}

pub struct ProcessHandler {
    pub table: [Process; PROCESS_LIMIT],
    pub pid: usize,
}

impl ProcessHandler {
    pub const fn new() -> ProcessHandler {
        ProcessHandler {
            table: [Process::new(); PROCESS_LIMIT],
            pid: 0,
        }
    }

    pub fn spawn(&mut self, addr: i64) {
        self.table[self.pid] = Process::new();

        self.table[self.pid].state = State::Waiting;
        self.table[self.pid].context.rip = addr;
        self.table[self.pid].base = addr;

        self.pid += 1;
    }
}

pub fn map<F>(pid: usize, f: F) where F: Fn(&mut Process) {
    unsafe {
        let mut lock = PROCESS.lock();

        f(&mut lock.table[pid]);
    }
}

pub fn get(pid: usize) -> Process {
    unsafe {
        let lock = PROCESS.lock();

        lock.table[pid]
    }
}

pub fn spawn(addr: i64) {
    unsafe {
        let mut lock = PROCESS.lock();

        lock.spawn(addr);
    }
}



