#![no_std]
#![no_main]

#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]
#![feature(ptr_as_ref_unchecked)]
#![feature(const_refs_to_cell)]
#![feature(slice_internals)]
#![test_runner(_test)]

mod allocator;
mod scheduler;
mod process;
mod debug;
mod interrupt;
mod scancodes;
mod tty;

use tty::TTY;

use limine::request::{FramebufferRequest, HhdmRequest, MemoryMapRequest, StackSizeRequest};
use limine::BaseRevision;
use spin::Mutex;

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;

static mut KERNEL_TTY: Mutex<Option<TTY>> = Mutex::new(None);
static mut KERNEL_TICKS: Mutex<usize> = Mutex::new(0);

#[used]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[used]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
static STACK_SIZE: StackSizeRequest = StackSizeRequest::new().with_size(0x32000);

// TODO: i think we just need to continue and develop it as a single process os

#[no_mangle]
pub extern "C" fn print_int(int: u64) {
    debug::write(format_args!("[debug] int: 0x{:x?}\n", int));
}

#[no_mangle]
pub unsafe extern "C" fn test() {
    asm!(
        "add rax, 1",
        "sub rax, 1",
    );

    /*
    if rax >= 1 {
        asm!(
            "sub rax, 1",
        );
    } else {
        asm!(
            "add rax, 1",
        );
    }
    */
}

#[no_mangle]
pub unsafe extern "C" fn proc1() {
    asm!(
        "mov rax, 1",
        "mov rbx, 2",
        "mov rcx, 3",
        "mov rdx, 4",
        "mov rsi, 5",
        "mov rdi, 6",
        "mov r8, 8",
        "mov r9, 9",
        "mov r10, 10",
        "mov r11, 11",
    );

    loop {
        test();

        /*
        asm!(
            "lea rax, [rip]",
            "mov rdi, rax",
            // "call print_int",

            "mov rdi, rsp",
            // "call print_int",
        );
        */

        // debug::write(format_args!("[debug] proc1 is running\n"));
    }
}

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    debug::write(format_args!("[debug] starting\n"));

    interrupt::init();

    if let Some(response) = FRAMEBUFFER.get_response() {
        if let Some(framebuffer) = response.framebuffers().next() {
            let mut lock = KERNEL_TTY.lock();

            *lock = Some(TTY::new(framebuffer));
        }
    }

    if let Some(tty) = KERNEL_TTY.lock().as_mut() {
        tty.write("Welcome to lios!\n");

        tty.render();
    }

    let hhdm = HHDM_REQUEST.get_response().expect("failed to get hhdm");
    let memory_map = MEMORY_MAP_REQUEST
        .get_response()
        .expect("failed to get memory map");

    allocator::init(&memory_map, hhdm);

    let addr = allocator::ALLOC.alloc(Layout::new::<[u64; 20]>().align_to(128).unwrap());
    debug::write(format_args!("[debug] allocated [u64; 20]: {:x?}\n", addr));

    let addr2 = allocator::ALLOC.alloc(Layout::new::<[u64; 12]>().align_to(128).unwrap());
    debug::write(format_args!("[debug] allocated [u64; 12]: {:x?}\n", addr2));

    allocator::ALLOC.dealloc(addr, Layout::new::<[u64; 20]>().align_to(128).unwrap());
    debug::write(format_args!("[debug] deallocated: {:x?}\n", addr));

    allocator::ALLOC.dealloc(addr2, Layout::new::<[u64; 12]>().align_to(128).unwrap());
    debug::write(format_args!("[debug] deallocated: {:x?}\n", addr2));

    process::spawn(proc1 as i64);
    // process::spawn(proc1 as i64);

    process::READY = true;

    loop {}

    halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(tty) = unsafe { KERNEL_TTY.lock().as_mut() } {
        tty.clear();

        if let (Some(message), Some(location)) = (info.message(), info.location()) {
            if write!(
                tty,
                "kernel panicked at `{}`, {}:{}:{}\n",
                message.as_str().unwrap_or_default(),
                location.file(),
                location.line(),
                location.column()
            )
            .is_err()
            {
                tty.write("kernel panicked: failed to format message");
            }
        }

        tty.render();
    }

    halt();
}

fn halt() -> ! {
    unsafe {
        asm!("cli");

        loop {
            asm!("hlt");
        }
    }
}

fn _test(_: &[&i32]) {}


