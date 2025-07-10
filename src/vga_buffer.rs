
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] //rust does not have u4, so using u8
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

impl ColorCode{
    fn new(foreground: Color, background: Color) -> ColorCode{
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar{
    ascii_char: u8,
    color_code: ColorCode,
}

//80x25 character buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGAPOINTER: i32 = 0xb8000;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    current_color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer{
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                self.buffer.chars[BUFFER_HEIGHT-1][self.column_position].write(ScreenChar {
                    ascii_char: byte,
                    color_code: self.current_color,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str){
        for byte in s.bytes(){
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self){
        //move all lines up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(char);
            }
        }
        self.clear_row(BUFFER_HEIGHT-1);
        self.column_position = 0;
    }
    fn clear_row(&mut self, row: usize){
        for col in 0..BUFFER_WIDTH {
            let blank = ScreenChar{
                ascii_char: b' ',
                color_code: self.current_color
            };
            self.buffer.chars[row][col].write(blank);
        }
    }
}


use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

//global interface :]
use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
pub static ref WRITER: Mutex<Writer> = Mutex::new(
        Writer {
    column_position: 0,
    current_color: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(VGAPOINTER as *mut Buffer)},
}
    );
}

//MACROS


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}


// TESTS DOWN HERE

#[test_case]
fn test_println_simple(){
    println!("test_println_simple output");
}
#[test_case]
fn test_println_many(){
    for _ in 0..200{
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_char), c);
    }
}

#[test_case]
fn test_println_wraparound(){
    println!("Long string! Gray eel-catfish labyrinth fish x-ray tetra, barbeled houndshark gianttail dorado Mexican golden trout, mudfish ground shark.\" North American freshwater catfish scaleless black dragonfish, \"blacktip reef shark,\" kaluga sea lamprey sixgill shark searobin; bluntnose knifefish, soldierfish. Butterfly ray red velvetfish golden trout humuhumunukunukuapua'a. Goldfish yellow-and-black triplefin mummichog, Pacific hake mackerel shark char banded killifish, \"scat halosaur, snoek weever, garden eel snailfish Pacific cod.\" Ghost fish roosterfish peamouth Australasian salmon jewel tetra pufferfish orbicular batfish convict cichlid stonecat spinefoot, seamoth silverside longjaw mudsucker burma danio shiner eucla cod yellowfin pike Asiatic glassfish. Javelin Pacific saury glowlight danio skipping goby jewelfish, hardhead catfish blackchin sand knifefish rivuline; Old World rivuline Atlantic trout.");
}