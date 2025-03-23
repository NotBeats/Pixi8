mod font;
use font::{FONT_SIZE, FONTSET};

const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const NUM_REGS: usize = 16;
const START_ADDRESS: u16 = 0x200;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    i_reg: u16,
    v_reg: [u8; NUM_REGS],
    pc: u16,
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; NUM_KEYS],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE],
            i_reg: 0,
            v_reg: [0; NUM_REGS],
            pc: START_ADDRESS,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; NUM_KEYS],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        };

        chip8.load_font();
        chip8
    }

    pub fn reset(&mut self) {
        self.memory = [0; MEMORY_SIZE];
        self.i_reg = 0;
        self.v_reg = [0; NUM_REGS];
        self.pc = START_ADDRESS;
        self.stack = [0; STACK_SIZE];
        self.stack_pointer = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keypad = [false; NUM_KEYS];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn load_font(&mut self) {
        self.memory[..FONT_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, address: u16) {
        self.stack[self.stack_pointer as usize] = address;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}
