use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use crate::interrupts::without_interrupts;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ForegroundColor {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BackgroundColor {
    Black = 0x00,
    Blue = 0x10,
    Green = 0x20,
    Cyan = 0x30,
    Red = 0x40,
    Magenta = 0x50,
    Brown = 0x60,
    LightGray = 0x70,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: ForegroundColor, background: BackgroundColor) -> ColorCode {
        ColorCode(foreground as u8 | background as u8)
    }

    pub fn blinking(foreground: ForegroundColor, background: BackgroundColor) -> ColorCode {
        ColorCode(foreground as u8 | background as u8 | 0x80)
    }

    fn background(&self) -> BackgroundColor {
        let background = self.0 & 0b0111_0000;
        unsafe { core::mem::transmute(background) }
    }

    fn foreground(&self) -> ForegroundColor {
        let foreground = self.0 & 0b0000_1111;
        unsafe { core::mem::transmute(foreground) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    buffer: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    const fn vga() -> &'static mut Buffer {
        unsafe { &mut *(0xb8000 as *mut Buffer) }
    }

    fn write(&mut self, row: usize, col: usize, character: ScreenChar) {
        self.buffer[row][col].write(character);
    }

    fn read(&self, row: usize, col: usize) -> ScreenChar {
        self.buffer[row][col].read()
    }

    fn clear_row(&mut self, row: usize, clear_color: BackgroundColor) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(ForegroundColor::White, clear_color),
        };
        for col in 0..BUFFER_WIDTH {
            self.write(row, col, blank);
        }
    }

    fn clear(&mut self, clear_color: BackgroundColor) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row, clear_color);
        }
    }
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::vga());
}

impl Writer {
    pub const fn vga() -> Writer {
        Writer {
            column_position: 0,
            color_code: ColorCode::new(ForegroundColor::White, BackgroundColor::Black),
            buffer: Buffer::vga(),
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.read(row, col);
                self.buffer.write(row - 1, col, character);
            }
        }
        self.buffer
            .clear_row(BUFFER_HEIGHT - 1, self.color_code.background());
        self.column_position = 0;
    }

    pub fn clear(&mut self) {
        self.buffer.clear(self.color_code.background());
        self.column_position = 0;
    }

    pub fn fill(&mut self, color: BackgroundColor) {
        self.color_code = ColorCode::new(ForegroundColor::White, color);
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.write(
                    row,
                    col,
                    ScreenChar {
                        ascii_character: b' ',
                        color_code: ColorCode::new(ForegroundColor::White, color),
                    },
                );
            }
        }
        self.column_position = 0;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.write(
                    row,
                    col,
                    ScreenChar {
                        ascii_character: byte,
                        color_code: self.color_code,
                    },
                );
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn erase(&mut self) {
        if self.column_position == 0 {
            return;
        }

        self.column_position -= 1;

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        self.buffer.write(
            row,
            col,
            ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            },
        );
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! fill_screen {
    ($color:ident) => {
        $crate::interrupts::without_interrupts(|| {
            $crate::vga::WRITER
                .lock()
                .fill($crate::vga::BackgroundColor::$color);
        });
    };
}

#[macro_export]
macro_rules! erase {
    () => {
        $crate::interrupts::without_interrupts(|| {
            $crate::out::vga::WRITER.lock().erase();
        })
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::out::vga::_print(format_args!($($arg)*), $crate::out::vga::ForegroundColor::White));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::out::vga::_print(format_args!($($arg)*), $crate::out::vga::ForegroundColor::Red));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments, foreground: ForegroundColor) {
    use core::fmt::Write;
    without_interrupts(|| {
        let mut writer = WRITER.lock();
        let old_color = writer.color_code.foreground();
        writer.color_code = ColorCode::new(foreground, writer.color_code.background());
        writer.write_fmt(args).unwrap();
        writer.color_code = ColorCode::new(old_color, writer.color_code.background());
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    without_interrupts(|| {
        println!("\n{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.read(BUFFER_HEIGHT - 2, i);
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
