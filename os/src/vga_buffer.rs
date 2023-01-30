// https://os.phil-opp.com/zh-CN/vga-text-mode/

/**
 * volatile提供一个名为Volatile的包装类型（wrapping type）和它的 read、write 方法；
 * 这些方法包装了 core::ptr 内的 read_volatile 和 write_volatile 函数，从而保证读操作或写操作不会被编译器优化。
 */
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}
impl core::ops::Deref for ScreenChar {
    type Target = ScreenChar;
    fn deref(&self) -> &Self::Target { self }
}
impl core::ops::DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target { self }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // Volatile 类型是一个泛型（generic），可以包装几乎所有的类型——这确保了我们不会通过普通的写入操作，意外地向它写入数据；我们转而使用提供的 write 方法。
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}
impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // 调用Volatile内的write方法 这能确保编译器不再优化这个写入操作
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
     pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe),
            }

        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| { // 可以让 Mutex 变量在锁定期间的执行逻辑不会被中断信号打断
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_println_simple() { // 验证 println是否正常运行而不会panic
        println!("test_println_simple output");
    }
    #[test_case]
    fn test_println_many() { // 打印很多行且有些行超出屏幕的情况下也没有panic发生
        for _ in 0..200 {
            println!("test_println_many output");
        }
    }
    #[test_case]
    fn test_println_output() { // 验证打印的几行字符是否真的出现在了屏幕上
        use core::fmt::Write;
        use x86_64::instructions::interrupts;
        let s = "Some test string that fits on a single line";
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }
    // #[test_case]
    // fn it_works() {}
}