use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::default());
}

const VGA_BUFFER_START: usize = 0xb8000;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fore: Color, back: Color) -> Self {
        let back_sh = (back as u8) << 4;
        Self(back_sh | fore as u8)
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScreenChar {
    ascii_ch: u8,
    color: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    col: usize,
    color: ColorCode,
    buf: &'static mut Buffer,
}

impl core::default::Default for Writer {
    fn default() -> Self {
        Self {
            col: 0,
            color: ColorCode::new(Color::Yellow, Color::Black),
            buf: unsafe { &mut *(VGA_BUFFER_START as *mut Buffer) },
        }
    }
}

impl Writer {
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn write_byte(&mut self, b: u8) {
        match b {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.col;
                let color = self.color;
                self.buf.chars[row][col].write(ScreenChar {
                    ascii_ch: byte,
                    color,
                });
                self.col += 1;
            }
        }
    }
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buf.chars[row][col].read();
                self.buf.chars[row - 1][col].write(c);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.col = 0;
    }

    pub(crate) fn clear(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let b = ScreenChar {
            ascii_ch: b' ',
            color: self.color,
        };
        for c in 0..BUFFER_WIDTH {
            self.buf.chars[row][c].write(b);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use kern_test::kern_test;

    #[kern_test]
    fn test_println_simple() {
        println!("test_println_simple output");
    }

    #[kern_test]
    fn test_println_many() {
        WRITER.lock().clear();
        for i in 0..200 {
            println!("test_println_many output: {}", i);
        }
    }
    #[kern_test]
    fn test_println_output() {
        WRITER.lock().clear();
        let s = "Some test string that fits on a single line";
        println!("{}", s);
        check_writer_line(BUFFER_HEIGHT - 2, s);
    }
    #[kern_test]
    fn test_print_wrap() {
        WRITER.lock().clear();
        let s = "Some text that doesn't fit on a single line, it needs to actually wrap around to the next line";
        print!("{}", s);
        check_writer_line(BUFFER_HEIGHT - 2, &s[..80]);
        check_writer_line(BUFFER_HEIGHT - 1, &s[80..]);
    }

    #[kern_test]
    fn test_println_overflow_output() {
        WRITER.lock().clear();
        let lines = [
            "line 0", "line 1", "line 2", "line 3", "line 4", "line 5", "line 6", "line 7", "line 8",
            "line 9", "line 10", "line 11", "line 12", "line 13", "line 14", "line 15", "line 16",
            "line 17", "line 18", "line 19", "line 20", "line 21", "line 22", "line 23", "line 24",
            "line 25", "line 26", "line 27", "line 28", "line 29", "line 30", "line 31", "line 32",
            "line 33", "line 34", "line 35", "line 36", "line 37", "line 38", "line 39", "line 40",
            "line 41", "line 42", "line 43", "line 44", "line 45", "line 46", "line 47", "line 48",
            "line 49", "line 50", "line 51", "line 52", "line 53", "line 54", "line 55", "line 56",
            "line 57", "line 58", "line 59", "line 60", "line 61", "line 62", "line 63", "line 64",
            "line 65", "line 66", "line 67", "line 68", "line 69", "line 70", "line 71", "line 72",
            "line 73", "line 74", "line 75", "line 76", "line 77", "line 78", "line 79", "line 80",
            "line 81", "line 82", "line 83", "line 84", "line 85", "line 86", "line 87", "line 88",
            "line 89", "line 90", "line 91", "line 92", "line 93", "line 94", "line 95", "line 96",
            "line 97", "line 98", "line 99",
        ];
        const H: usize = BUFFER_HEIGHT - 1;
        for i in 0..100 {
            if i >= H {
                check_writer_line(0, lines[i - H]);
            }
            println!("{}", lines[i]);
        }
        check_writer_line(0, lines[lines.len() - H]);
        serial::SERIAL.lock().
    }

    fn check_writer_line(line: usize, against: &str) {
        for (i, c) in against.chars().enumerate() {
            let sc = WRITER.lock().buf.chars[line][i].read();
            assert_eq!(char::from(sc.ascii_ch), c);
        }
    }

}
