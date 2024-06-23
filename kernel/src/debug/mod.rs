use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;
use x86::io;

use core::fmt::{self, Write};


lazy_static! {
    pub static ref SERIAL_PORT: Mutex<SerialPort> = {
        Mutex::new(SerialPort::init(0x3f8))
    };
}

pub fn write(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        if SERIAL_PORT.lock().write_fmt(args).is_err() {
            panic!("failed to write to serial port");
        }
    });
}

pub struct SerialPort {
    port: u16,
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, message: &str) -> fmt::Result {
        self.write(message);

        Ok(())
    }
}

impl SerialPort {
    pub fn init(port: u16) -> SerialPort {
        unsafe {
            io::outb(port + 1, 0x00);
            io::outb(port + 3, 0x80);
            io::outb(port, 0x03);
            io::outb(port + 1, 0x00);
            io::outb(port + 3, 0x03);
            io::outb(port + 2, 0xc7);
            io::outb(port + 4, 0x0b);
            io::outb(port + 4, 0x1e);
            io::outb(port + 4, 0x0f);
        }


        SerialPort {
            port,
        }
    }

    pub fn write(&self, message: &str) {
        unsafe {
            for character in message.chars() {
                while io::inb(self.port + 5) & 0x20 == 0 {}

                io::outb(self.port, character as u8);
            }
        }
    }
}


