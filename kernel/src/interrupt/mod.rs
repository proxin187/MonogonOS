use crate::{KERNEL_TTY, process, debug, halt, scheduler, syscall::Syscall, scancodes::Scancodes, process::Context};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

use core::fmt::Write;
use core::arch::asm;

// TODO: disable interrupts while handling other interrupts


static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(32, 40) });
static SCANCODES: Mutex<Scancodes> = Mutex::new(Scancodes::new());

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint);
        idt.double_fault.set_handler_fn(double_fault);
        idt[32].set_handler_fn(timer_interrupt);
        idt[33].set_handler_fn(keyboard_interrupt);
        idt[128].set_handler_fn(syscall_interrupt);

        return idt;
    };
}

pub fn init() {
    IDT.load();

    unsafe {
        PICS.lock().initialize();

        PICS.lock().write_masks(0, 0);
    }

    x86_64::instructions::interrupts::enable();

    debug::write(format_args!("[debug] initialized\n"));
}

#[no_mangle]
extern "x86-interrupt" fn syscall_interrupt(_stack_frame: InterruptStackFrame) {
    unsafe {
        x86_64::instructions::interrupts::disable();

        let mut syscall = Syscall::new();

        asm!(
            "mov {rax}, [rsp + 112]",
            "mov {rdi}, [rsp + 112 + 32]",
            "mov {rsi}, [rsp + 112 + 24]",
            "mov {rdx}, [rsp + 112 + 16]",
            rax = out(reg) syscall.args[0],
            rdi = out(reg) syscall.args[1],
            rsi = out(reg) syscall.args[2],
            rdx = out(reg) syscall.args[3],
        );

        syscall.perform();

        PICS.lock().notify_end_of_interrupt(128);

        x86_64::instructions::interrupts::enable();
    }
}

#[no_mangle]
extern "x86-interrupt" fn timer_interrupt(stack_frame: InterruptStackFrame) {
    unsafe {
        x86_64::instructions::interrupts::disable();

        // TODO: all the registers are already pushed on the stack for us before the rust compiler
        // has had any time to mess with them, we need to figure a way to read these registers
        //
        // sp = rsp + 0xb0
        // rax = [sp]
        // rcx = [sp + 8]
        // rdx = [sp + 16]
        // rsi = [sp + 24]
        // rdi = [sp + 32]
        // r8  = [sp + 40]
        // r9  = [sp + 48]
        // r10 = [sp + 56]
        // r11 = [sp + 64]

        if process::READY {
            let mut context = Context::new();

            // TODO: some registers still seem to be saved wrong

            // 272
            asm!(
                "mov {rdi}, [rsp + 512 + 32]",
                "mov {rsi}, [rsp + 512 + 24]",
                "mov {rdx}, [rsp + 512 + 16]",
                "mov {rcx}, [rsp + 512 + 8]",
                "mov {rax}, [rsp + 512]",
                "mov {rbp}, rbp",
                "mov {rbx}, rbx",
                rdi = out(reg) context.rdi,
                rsi = out(reg) context.rsi,
                rdx = out(reg) context.rdx,
                rcx = out(reg) context.rcx,
                rax = out(reg) context.rax,
                rbp = out(reg) context.rbp,
                rbx = out(reg) context.rbx,
            );

            asm!(
                "mov {r8}, [rsp + 512 + 40]",
                "mov {r9}, [rsp + 512 + 48]",
                "mov {r10}, [rsp + 512 + 56]",
                "mov {r11}, [rsp + 512 + 64]",
                r8 = out(reg) context.r8,
                r9 = out(reg) context.r9,
                r10 = out(reg) context.r10,
                r11 = out(reg) context.r11,
            );

            /*
            asm!(
                "mov {rsp}, {sp}",
                "mov {rip}, {ip}",
                rsp = out(reg) context.rsp,
                rip = out(reg) context.rip,
                sp = in(reg) stack_frame.stack_pointer.as_u64(),
                ip = in(reg) stack_frame.instruction_pointer.as_u64(),
            );
            */

            context.rsp = stack_frame.stack_pointer.as_u64() as i64;
            context.rip = stack_frame.instruction_pointer.as_u64() as i64;

            scheduler::schedule(context);

            /*
            asm!(
                // call convention
                // rdi, rsi, rdx, rcx, r8, r9, [rsp], [rsp + 8], [rsp + 16]

//                            rdi       rsi       rdx       rcx       r8        r9        rsp       rsp + 8   rsp + 16  sp + 24  rsp + 32 rsp + 40  rsp + 48
// pub extern "C" fn schedule(rdi: i64, rsi: i64, rdx: i64, rcx: i64, rbp: i64, rsp: i64, rbx: i64, rax: i64, rip: i64, r8: i64, r9: i64, r10: i64, r11: i64) {
                // "mov [rsp - 8], {ip}",
                "mov [rsp - 40], {ip}",

                "mov rdi, [rsp + 176 + 40]",
                "mov rsi, [rsp + 176 + 48]",
                "mov rdx, [rsp + 176 + 56]",
                "mov rcx, [rsp + 176 + 64]",

                "mov [rsp - 32], rdi",
                "mov [rsp - 24], rsi",
                "mov [rsp - 16], rdx",
                "mov [rsp - 8], rcx",

                "mov rdi, [rsp + 176 + 32]",
                "mov rsi, [rsp + 176 + 24]",
                "mov rdx, [rsp + 176 + 16]",
                "mov rcx, [rsp + 176 + 8]",
                "mov r8, rbp",
                "mov r9, {sp}",

                "mov rax, [rsp + 176]",

                // "sub rsp, 24",
                "sub rsp, 56",

                "mov [rsp + 8], rax",

                "mov [rsp], rbx",

                // we may need this for debugging later
                // "mov [rsp + 16], {ip}",

                "call schedule",

                // "add rsp, 24",
                "add rsp, 56",

                sp = in(reg) stack_frame.stack_pointer.as_u64(),
                ip = in(reg) stack_frame.instruction_pointer.as_u64(),
            );
            */

            PICS.lock().notify_end_of_interrupt(32);

            asm!(
                // remove registers from stack
                // "add rsp, 0xb0",

                // jump to next_process
                // "jmp next_process"

                // MonogonOS
                // TODO: maybe we need to use iret in order to call the function? that will maybe
                // preserve the stack pointer and stuff
                //
                // we want to make iretq return to the next_process function.
                // TODO: MAYBE WE NEED TO POP?, OR MAYBE THE STRUCT IS CORRUPTED IN ANY OTHER WAY?
                "push {stack_segment:r}",
                "push {new_stack_pointer}",
                "push {rflags}",
                "push {code_segment:r}",
                "push {instruction_pointer}",
                "iretq",
                rflags = in(reg) stack_frame.cpu_flags.bits(),
                // for some reason the stack pointer is always decremented by 8 bytes so we need to
                // increment it by 8 just to stop it from overflowing
                // new_stack_pointer = in(reg) stack_frame.stack_pointer.as_u64(),
                // TODO: this also crashes because rsp is set to 0 when we first start
                new_stack_pointer = in(reg) scheduler::NEXT_PROCESS.context.rsp,
                code_segment = in(reg) stack_frame.code_segment.0,
                stack_segment = in(reg) stack_frame.stack_segment.0,
                instruction_pointer = in(reg) scheduler::next_process as u64,
            );
        } else {
            PICS.lock().notify_end_of_interrupt(32);

            x86_64::instructions::interrupts::enable();
        }
    }
}

extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Some(character) = SCANCODES.lock().advance(scancode) {
        debug::write(format_args!("character: {:?}\n", character));
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(33);
    }
}

#[no_mangle]
extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    debug::write(format_args!("[debug] double fault: {:#?}\nerror_code: {}\n", stack_frame, error_code));

    if let Some(tty) = unsafe { KERNEL_TTY.lock().as_mut() } {
        tty.clear();

        if write!(tty, "unrecovarable double fault: {:#?}\nerror_code: {}", stack_frame, error_code).is_err() {
            tty.write("unrecovarable double fault: failed to format");
        }

        tty.render();
    }

    halt();
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    debug::write(format_args!("[debug] breakpoint\n"));

    if let Some(tty) = unsafe { KERNEL_TTY.lock().as_mut() } {
        tty.clear();

        if write!(tty, "kernel interrupt: {:#?}", stack_frame).is_err() {
            tty.write("kernel interrupt: failed to format");
        }

        tty.render();
    }
}


