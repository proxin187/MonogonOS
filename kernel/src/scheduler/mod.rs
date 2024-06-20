use crate::process::{self, *};
use crate::debug;

use spin::Mutex;

use core::arch::asm;

pub static mut SCHEDUELER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
pub static mut NEXT_PROCESS: Process = Process::new();


pub struct Scheduler {
    current_pid: usize,
    initialized: bool,
}

impl Scheduler {
    pub const fn new() -> Scheduler {
        Scheduler {
            current_pid: 0,
            initialized: false,
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
                if !self.initialized {
                    self.initialized = true;
                } else {
                    let mut lock = PROCESS.lock();

                    // TODO: the rip is still not set correctly

                    // we dont want to save context on first context switch
                    // if lock.table[self.current_pid].context.rip != lock.table[self.current_pid].base {
                        lock.table[self.current_pid].context = context;
                        lock.table[self.current_pid].state = State::Waiting;
                    // }
                }

                /*
                process::map(self.current_pid, |proc| {
                    debug::write(format_args!("[debug] huh: {:x?}\n", context));

                    proc.context = context;
                    proc.state = State::Waiting;

                    debug::write(format_args!("[debug] wtf: {:x?}\n", proc.context));
                });
                */

                /*
                process::for_each(|context| {
                    debug::write(format_args!("[debug] context: {:x?}\n", context));
                });
                */

                self.next_pid();

                while process::get(self.current_pid).is_empty() {
                    self.next_pid();
                }

                /*
                process::map(self.current_pid, |proc| {
                    proc.state = State::Running;
                });
                */

                NEXT_PROCESS = process::get(self.current_pid);
            }

            debug::write(format_args!("[debug] returning from next\n"));
        }
    }
}

#[no_mangle]
//                         rdi       rsi       rdx       rcx       r8        r9        rsp     rsp + 8  rsp + 16  sp + 24  rsp + 32
// pub extern "C" fn schedule(rdi: i64, rsi: i64, rdx: i64, rcx: i64, rbp: i64, rsp: i64, rbx: i64, rax: i64, rip: i64, r8: i64, r9: i64, r10: i64, r11: i64) {
pub extern "C" fn schedule(context: Context) {
    unsafe {
        // debug::write(format_args!("[debug] rsp: {:x?}\n", context.rsp));

        let mut lock = SCHEDUELER.lock();

        /*
        process::for_each(|context| {
            debug::write(format_args!("[debug] context: {:x?}\n", context));
        });
        */

        lock.next(context);

        /*
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
            r8,
            r9,
            r10,
            r11,
        });
        */

        /*
        process::for_each(|context| {
            debug::write(format_args!("[debug] context: {:x?}\n", context));
        });
        */

        debug::write(format_args!("[debug] returning from scheduele: {:x?}\n", NEXT_PROCESS.context));
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


        // NOTE: this simply NOT wrong

        asm!(
            "mov rax, {rax}",
            "mov rbx, {rbx}",
            "mov rcx, {rcx}",
            "mov rdx, {rdx}",
            "mov rsi, {rsi}",
            "mov rdi, {rdi}",
            "mov rsp, {rsp}",
            "mov r8, {r8}",
            "mov r9, {r9}",
            "mov r10, {r10}",
            "mov r11, {r11}",
            "sti",
            "jmp {rip}",
            rax = in(reg) NEXT_PROCESS.context.rax,
            rbx = in(reg) NEXT_PROCESS.context.rbx,
            rcx = in(reg) NEXT_PROCESS.context.rcx,
            rdx = in(reg) NEXT_PROCESS.context.rdx,
            rsi = in(reg) NEXT_PROCESS.context.rsi,
            rdi = in(reg) NEXT_PROCESS.context.rdi,
            rsp = in(reg) NEXT_PROCESS.context.rsp,
            r8 = in(reg) NEXT_PROCESS.context.r8,
            r9 = in(reg) NEXT_PROCESS.context.r9,
            r10 = in(reg) NEXT_PROCESS.context.r10,
            r11 = in(reg) NEXT_PROCESS.context.r11,
            rip = in(reg) NEXT_PROCESS.context.rip,
        );

        // x86_64::instructions::interrupts::enable();

        /*
        asm!(
            "sti",
            "jmp {rip}",
            rip = in(reg) NEXT_PROCESS.context.rip,
        );
        */
    }
}


