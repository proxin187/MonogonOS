use crate::process::{self, *};
use crate::debug;

use spin::Mutex;

use core::arch::asm;

pub static mut SCHEDUELER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
pub static mut NEXT_PROCESS: Process = Process::new();


pub struct Scheduler {
    current_pid: usize,
}

impl Scheduler {
    pub const fn new() -> Scheduler {
        Scheduler {
            current_pid: 0,
        }
    }

    pub fn next_pid(&mut self) {
        if self.current_pid >= PROCESS_LIMIT - 1 {
            self.current_pid = 0;
        } else {
            self.current_pid += 1;
        }
    }

    pub fn next(&mut self, context: Context) {
        debug::write(format_args!("[debug] context: {:x?}\n", context));

        unsafe {
            let is_empty = PROCESS.lock().table.iter().all(|process| process.is_empty());

            if !is_empty {
                if process::get(self.current_pid).state == State::Running {
                    process::map(self.current_pid, |proc| {
                        proc.context = context;
                        proc.state = State::Waiting;
                    });
                }

                self.next_pid();

                while process::get(self.current_pid).is_empty() {
                    self.next_pid();
                }

                process::map(self.current_pid, |proc| {
                    proc.state = State::Running;
                });

                NEXT_PROCESS = process::get(self.current_pid);
            }

            debug::write(format_args!("[debug] returning from next\n"));
        }
    }
}

#[no_mangle]
//                         rdi       rsi       rdx       rcx       r8        r9       [rsp + 16][rsp + 24][rsp + 32]
pub extern "C" fn schedule(rdi: i64, rsi: i64, rdx: i64, rcx: i64, rbp: i64, rsp: i64, rbx: i64, rax: i64, rip: i64) {
    unsafe {
        let mut lock = SCHEDUELER.lock();

        lock.next(Context {
            rax,
            rbx,
            rcx,
            rdx,
            rsp,
            rbp,
            rsi,
            rdi,
            rip,
        });

        debug::write(format_args!("[debug] returning from scheduele: {:?}\n", NEXT_PROCESS.context));
    }
}

#[no_mangle]
pub extern "C" fn next_process() {
    unsafe {
        /*
        debug::write(format_args!("[debug] next process called\n"));

        let current_pid = SCHEDUELER.lock().current_pid;

        let context = process::get(current_pid).context;

        debug::write(format_args!("[debug] context: {:x?}\n", context));
        */

        asm!(
            "mov rax, {rax}",
            "mov rbx, {rbx}",
            "mov rcx, {rcx}",
            "mov rdx, {rdx}",
            "mov rsi, {rsi}",
            "mov rdi, {rdi}",
            rax = in(reg) NEXT_PROCESS.context.rax,
            rbx = in(reg) NEXT_PROCESS.context.rbx,
            rcx = in(reg) NEXT_PROCESS.context.rcx,
            rdx = in(reg) NEXT_PROCESS.context.rdx,
            rsi = in(reg) NEXT_PROCESS.context.rsi,
            rdi = in(reg) NEXT_PROCESS.context.rdi,
        );

        // x86_64::instructions::interrupts::enable();

        asm!(
            "sti",
            "jmp {rip}",
            rip = in(reg) NEXT_PROCESS.context.rip,
        );
    }
}


