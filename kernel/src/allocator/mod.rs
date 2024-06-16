use crate::debug;

use limine::memory_map::{Entry, EntryType};
use limine::response::{HhdmResponse, MemoryMapResponse};
use spin::Mutex;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::cmp::Ordering;
use core::slice;

const CHUNK_LIMIT: usize = 100;

#[global_allocator]
pub static mut ALLOC: Allocator = Allocator::new();


#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    base: u64,
    length: u64,
}

impl Chunk {
    pub const fn new(base: u64, length: u64) -> Chunk {
        Chunk {
            base,
            length,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base == 0 && self.length == 0
    }
}

pub struct Allocator {
    chunks: *mut [Chunk; CHUNK_LIMIT],
    lock: Mutex<()>,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            chunks: ptr::null_mut(),
            lock: Mutex::new(()),
        }
    }

    pub fn map<F>(&self, f: F) where F: Fn(&mut Chunk) -> bool {
        unsafe {
            for chunk in (*self.chunks).iter_mut() {
                if f(chunk) {
                    break;
                }
            }
        }
    }

    pub unsafe fn merge(&self) {
        let mut index = 1;

        while !(*self.chunks)[index].is_empty() && index < CHUNK_LIMIT - 1 {
            if (*self.chunks)[index].base + (*self.chunks)[index].length == (*self.chunks)[index - 1].base {
                (*self.chunks)[index].length += (*self.chunks)[index - 1].length;

                (*self.chunks)[index - 1] = Chunk::new(0, 0);

                (*self.chunks)[index - 1..].rotate_left(1);
            } else {
                index += 1;
            }
        }
    }

    pub fn push(&self, new: Chunk) {
        self.map(|chunk| {
            chunk.is_empty().then(|| { *chunk = new; true }).unwrap_or(false)
        });
    }

    pub fn largest(&self) -> Option<usize> {
        unsafe {
            (*self.chunks).iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.length.partial_cmp(&b.length).unwrap_or(Ordering::Equal))
                .map(|(index, _)| index)
        }
    }

    pub unsafe fn cleanup(&self) {
        slice::sort::quicksort(self.chunks.as_mut_unchecked(), |a, b| a.base > b.base);

        self.merge();
    }
}

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock.lock();

        if let Some(index) = self.largest() {
            if (*self.chunks)[index].length < layout.size() as u64 {
                panic!("not enough memory, buy more ram :)");
            }

            (*self.chunks)[index].length -= layout.size() as u64;

            let offset = ((*self.chunks)[index].base + (*self.chunks)[index].length) % layout.align() as u64;

            if offset != 0 {
                (*self.chunks)[index].length -= offset;

                self.push(Chunk::new((*self.chunks)[index].base + (*self.chunks)[index].length + layout.size() as u64, offset));
            }

            let addr = (*self.chunks)[index].base + (*self.chunks)[index].length;

            self.cleanup();

            /*
            self.map(|chunk| {
                debug::write(format_args!("[debug] {:x?}\n", chunk));

                chunk.is_empty()
            });
            */

            addr as *mut u8
        } else {
            panic!("ran out of memory chunks");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock.lock();

        self.push(Chunk::new(ptr as u64, layout.size() as u64));

        self.cleanup();

        /*
        self.map(|chunk| {
            debug::write(format_args!("[debug] {:x?}\n", chunk));

            chunk.is_empty()
        });
        */
    }
}

pub fn init(memory_map: &MemoryMapResponse, hhdm: &HhdmResponse) {
    let entries = memory_map
        .entries()
        .iter()
        .filter(|entry| entry.entry_type == EntryType::USABLE)
        .map(|entry| Entry {
            base: entry.base + hhdm.offset(),
            length: entry.length,
            entry_type: entry.entry_type,
        });

    unsafe {
        for entry in entries {
            if ALLOC.chunks.is_null() {
                *(entry.base as *mut [Chunk; CHUNK_LIMIT]) = [Chunk::new(0, 0); CHUNK_LIMIT];

                ALLOC.chunks = entry.base as *mut [Chunk; CHUNK_LIMIT];
            } else {
                ALLOC.push(Chunk::new(entry.base, entry.length));
            }
        }

        slice::sort::quicksort(ALLOC.chunks.as_mut_unchecked(), |a, b| a.base > b.base);

        ALLOC.map(|chunk| {
            debug::write(format_args!("[debug] {:x?}\n", chunk));

            chunk.is_empty()
        });
    }
}


