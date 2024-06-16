use noto_sans_mono_bitmap::{FontWeight, RasterHeight};
use limine::framebuffer::Framebuffer;

use core::fmt;


pub struct Cursor {
    row: usize,
    col: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            row: 0,
            col: 0,
        }
    }

    pub fn reset(&mut self) {
        self.row = 0;
        self.col = 0;
    }

    pub fn advance_col(&mut self, max: usize) {
        if self.col < max {
            self.col += 1;
        }
    }

    pub fn advance_row(&mut self, max: usize) {
        if self.row < max {
            self.row += 1;
        }
    }
}

pub struct Frame<'a> {
    framebuffer: Framebuffer<'a>,
}

impl<'a> Frame<'a> {
    fn plot_pixel(&self, x: usize, y: usize, pixel: u32) {
        let offset = x * self.framebuffer.bpp() as usize / 8 + y * self.framebuffer.pitch() as usize;

        unsafe {
            *(self.framebuffer.addr().add(offset) as *mut u32) = pixel;
        }
    }

    fn draw_char(&self, character: &char, row: usize, col: usize) {
        if let Some(raster) = noto_sans_mono_bitmap::get_raster(*character, FontWeight::Regular, RasterHeight::Size16) {
            for (y, line) in raster.raster().iter().enumerate() {
                for (x, pixel) in line.iter().enumerate() {
                    let color = *pixel as u32;
                    let pixel = color << 16 | color << 8 | color;

                    self.plot_pixel(x + (col * raster.width()), y + (row * raster.height()), pixel);
                }
            }
        }
    }

    fn draw_rect(&self, x: usize, y: usize, width: usize, height: usize, pixel: u32) {
        for y in y..y + height {
            for x in x..x + width {
                self.plot_pixel(x, y, pixel);
            }
        }
    }
}

struct Block {
    height: usize,
    width: usize,
}

pub struct TTY<'a> {
    buffer: [[char; 80]; 24],
    cursor: Cursor,
    frame: Frame<'a>,
    block: Block,
}

impl<'a> fmt::Write for TTY<'a> {
    fn write_str(&mut self, content: &str) -> fmt::Result {
        self.write(content);

        Ok(())
    }
}

impl<'a> TTY<'a> {
    pub fn new(framebuffer: Framebuffer) -> TTY {
        TTY {
            buffer: [[' '; 80]; 24],
            cursor: Cursor::new(),
            frame: Frame {
                framebuffer,
            },
            block: Block {
                height: RasterHeight::Size16.val(),
                width: noto_sans_mono_bitmap::get_raster_width(FontWeight::Regular, RasterHeight::Size16),
            },
        }
    }

    pub fn clear(&mut self) {
        self.cursor.reset();

        self.frame.draw_rect(0, 0, self.frame.framebuffer.width() as usize, self.frame.framebuffer.height() as usize, 0x00000000);
    }

    pub fn write(&mut self, content: &str) {
        for character in content.chars() {
            match character {
                '\n' => {
                    self.cursor.col = 0;
                    self.cursor.advance_row(self.buffer.len() - 1);
                },
                _ => {
                    self.buffer[self.cursor.row][self.cursor.col] = character;
                    self.cursor.advance_col(self.buffer[0].len() - 1);
                },
            }
        }
    }

    pub fn render(&self) {
        for (row, line) in self.buffer.iter().enumerate() {
            for (col, character) in line.iter().enumerate() {
                self.frame.draw_char(character, row, col);
            }
        }

        self.frame.draw_rect(self.cursor.col * self.block.width, self.cursor.row * self.block.height, 6, 12, 0xffffffff);
    }
}


