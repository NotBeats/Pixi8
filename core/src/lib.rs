mod font;
use std::usize;

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
    program_counter: u16,
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
            program_counter: START_ADDRESS,
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
        self.program_counter = START_ADDRESS;
        self.stack = [0; STACK_SIZE];
        self.stack_pointer = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keypad = [false; NUM_KEYS];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.load_font();
    }

    fn load_font(&mut self) {
        self.memory[..FONT_SIZE].copy_from_slice(&FONTSET);
    }

    fn push_to_stack(&mut self, address: u16) {
        self.stack[self.stack_pointer as usize] = address;
        self.stack_pointer += 1;
    }

    fn pop_from_stack(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    fn timer_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            // TODO: play sound
        }
    }

    fn fetch_opcode(&mut self) -> u16 {
        let opcode = self.memory[self.program_counter as usize] as u16;
        let next_opcode = self.memory[(self.program_counter + 1) as usize] as u16;
        self.program_counter += 2;
        opcode << 8 | next_opcode
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let nibble1 = (opcode & 0xF000) >> 12;
        let nibble2 = (opcode & 0x0F00) >> 8;
        let nibble3 = (opcode & 0x00F0) >> 4;
        let nibble4 = opcode & 0x000F;

        match (nibble1, nibble2, nibble3, nibble4) {
            // returns because this is for certain computers for the original chip8 computers
            (0, 0, 0, 0) => return,
            // 00E0 - clears the screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // 00EE - returning from a subroutine
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop_from_stack();
                self.program_counter = return_address;
            }
            // 1nnn - jumps to nnn
            (1, _, _, _) => {
                let nnn = opcode & 0x0FFF;
                self.program_counter = nnn;
            }
            // 2nnn - calls subroutine at nnn
            (2, _, _, _) => {
                let nnn = opcode & 0x0FFF;
                self.push_to_stack(self.program_counter);
                self.program_counter = nnn;
            }
            // 3nnn - skip if vx == nn
            (3, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v_reg[x] == nn {
                    self.program_counter += 2;
                }
            }
            // 4nnn - skip if vx != nn
            (4, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (opcode & 0x00FF) as u8;
                if self.v_reg[x] != nn {
                    self.program_counter += 2;
                }
            }
            // 5xy0 - skips if vx == vy
            (5, _, _, _) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.program_counter += 2;
                }
            }
            // 6xnn - set register vx to nn
            (6, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v_reg[x] = nn;
            }
            // 7xnn - add nn to register vx
            (7, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            // 8xy0 - vx is set to vy
            (8, _, _, 0) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            // 8xy1 - vx is set to the binary or of vx and vy
            (8, _, _, 1) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // 8xy2 - vx is set to the binary and of vx and vy
            (8, _, _, 2) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            // 8xy3 - vx is set to the binary xor of vx and vy
            (8, _, _, 3) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }
            // 8xy4 - vx is set to vx plus vy
            (8, _, _, 4) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] += self.v_reg[y];
            }
            // 8xy5 - vx is set to vx - vy
            (8, _, _, 5) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // 8xy6 - vy is put into vx then shifts vx one bit to the right
            (8, _, _, 6) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // 8xy7 - vx is set to vy - vx
            (8, _, _, 7) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // 8xyE - vy is put into vx then shifts vx one bit to the left
            (8, _, _, 0xE) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // 9xy0 - skips if vx != vy
            (9, _, _, _) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.program_counter += 2;
                }
            }
            // annn - set index register i to nnn
            (0xA, _, _, _) => {
                let nnn = opcode & 0x00FFF;
                self.i_reg = nnn;
            }
            // dxyn - display/draw
            (0xD, _, _, _) => {
                let x; // TODO: get x from v_reg[0]
                let y; // TODO: get y from v_reg[1]
                // TODO: draw n pixel tall at x, y
            }
        }
    }
}
