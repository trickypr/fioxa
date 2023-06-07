#![no_std]
#![no_main]

use alloc::string::String;
use input::keyboard::{
    us_keyboard::USKeymap,
    virtual_code::{Control, Modifier, VirtualKeyCode},
    KeyboardEvent,
};
use kernel_userspace::syscall::{self, exit, stream_pop};

extern crate alloc;
#[macro_use]
extern crate userspace;
extern crate userspace_bumpalloc;

#[export_name = "_start"]
pub extern "C" fn main() {
    let mut line = String::new();
    let mut decoder = KBInputDecoder::new();

    loop {
        if let Some(msg) = syscall::stream_pop() {
            let event = unsafe { &*(&msg.data as *const [u8] as *const KeyboardEvent) };

            if let Some(char) = decoder.parse_char(event) {
                line.push(char);
                print!("{}", char);
            } else if decoder.ret(event) {
                // TODO: parsing
                print!("{}\n> ", line);
                line = String::new();
            }
        }

        syscall::yield_now();
    }
}

#[panic_handler]
fn panic(i: &core::panic::PanicInfo) -> ! {
    println!("{}", i);
    syscall::exit()
}

pub struct KBInputDecoder {
    lshift: bool,
    rshift: bool,
    caps_lock: bool,
    num_lock: bool,
}

impl KBInputDecoder {
    pub fn new() -> Self {
        Self {
            lshift: false,
            rshift: false,
            caps_lock: false,
            num_lock: false,
        }
    }

    pub fn shift(&self) -> bool {
        self.lshift && self.rshift
    }

    /// Checks if the event is the return key being pressed. If the shift key
    /// is being held, it will prevent this from calling
    pub fn ret(&self, event: &KeyboardEvent) -> bool {
        match event {
            KeyboardEvent::Up(VirtualKeyCode::Control(key)) => match key {
                Control::Enter => !self.shift(),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn parse_char(&mut self, event: &KeyboardEvent) -> Option<char> {
        match event {
            KeyboardEvent::Up(VirtualKeyCode::Modifier(key)) => match key {
                Modifier::LeftShift => {
                    self.lshift = false;
                    None
                }
                Modifier::RightShift => {
                    self.rshift = false;
                    None
                }
                _ => None,
            },
            KeyboardEvent::Up(_) => None,
            KeyboardEvent::Down(VirtualKeyCode::Modifier(key)) => match key {
                Modifier::LeftShift => {
                    self.lshift = true;
                    None
                }
                Modifier::RightShift => {
                    self.rshift = true;
                    None
                }
                Modifier::CapsLock => {
                    self.caps_lock = !self.caps_lock;
                    None
                }
                Modifier::NumLock => {
                    self.num_lock = !self.num_lock;
                    None
                }
                _ => None,
            },
            KeyboardEvent::Down(letter) => Some(USKeymap::get_unicode(
                letter.clone(),
                self.lshift,
                self.rshift,
                self.caps_lock,
                self.num_lock,
            )),
        }
    }
}
