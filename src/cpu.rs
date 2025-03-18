use std::fs;
use rand::Rng;

const MAX_REGISTER_LEN: usize = 16;
const MAX_RAM_LEN: usize = 4096;
const START_ADDR: u16 = 0x200; 
const VIDEO_HEIGHT: u32 = 32;
const VIDEO_WIDTH: u32 = 64;
const MAX_SCREEN_LEN: usize = 64 * 32;
const MAX_KEYPAD_LEN: usize = 16;
const MAX_STACK_LEN: usize = 16;

pub enum State {
    Running,
    Paused,
    Drawing,
}

#[derive(PartialEq)]
pub enum Mode {
    Modern,
    Classic,
}

impl State {
    pub fn new(state: &str) -> Self {
        match state {
            "running" => State::Running,
            "paused" => State::Paused,
            "drawing" => State::Drawing,
            &_ => panic!("Invalid state"),
        }
    } 
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Key {
    pub scancode: u8,
    pub pressed: bool,
}

pub struct CPU {
    pub state: State,
    pc: u16,
    index: u16,
    pub buf: [u8; MAX_SCREEN_LEN],
    ram: [u8; MAX_RAM_LEN],
    reg: [u8; MAX_REGISTER_LEN],
    stack: [u16; MAX_STACK_LEN],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    pub keypad: [Key; MAX_KEYPAD_LEN], // setting a keypad with 15 elements
    pub draw: bool,
    mode: Mode,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            state: State::Running,
            draw: false,
            pc: START_ADDR,
            index: 0,
            buf: [0; MAX_SCREEN_LEN],
            ram: [0; MAX_RAM_LEN],
            reg: [0; MAX_REGISTER_LEN],
            stack: [0; 16],
            sp: 0,
            keypad: [Key{scancode: 0, pressed: false}; MAX_KEYPAD_LEN],
            delay_timer: 0,
            sound_timer: 0,
            mode: Mode::Classic,
        }
    }

    pub fn load_font(&mut self, font: &[u8]) -> &mut CPU {
        for i in 0..font.len() {
            self.ram[0x050 + i] = font[i];
        }

        self        
    }

    pub fn load_rom(&mut self, fpath: &str) -> &mut CPU{
        let buf = fs::read(fpath).expect("File could not be read");

        for i in 0..buf.len() {
            self.ram[START_ADDR as usize + i] = buf[i];
        }

        self
    }

    pub fn set_mode(&mut self ,mode: Mode) {
        self.mode = mode;
    }
    
    pub fn map_keypad(&mut self) {

        self.keypad[0].scancode = 0x1;
        self.keypad[1].scancode = 0x2;
        self.keypad[2].scancode = 0x3;
        self.keypad[3].scancode = 0xC;
        self.keypad[4].scancode = 0x4;
        self.keypad[5].scancode = 0x5;
        self.keypad[6].scancode = 0x6;
        self.keypad[7].scancode = 0xD;
        self.keypad[8].scancode = 0x7;
        self.keypad[9].scancode = 0x8;
        self.keypad[10].scancode = 0x9;
        self.keypad[11].scancode = 0xE;
        self.keypad[12].scancode = 0xA;
        self.keypad[13].scancode = 0x0;
        self.keypad[14].scancode = 0xB;
        self.keypad[15].scancode = 0xF;
        
    }

}

impl CPU {
    // Fetching two bytes from memory and merging them into one instruction
    fn fetch(&mut self) -> u16 {
        let high: u16 = (self.ram[self.pc as usize]) as u16;
        let low: u16 = (self.ram[self.pc as usize + 1]) as u16;

        (high << 8) | low
    }

    pub fn update_dt(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn update_st(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn step(&mut self) -> State {        
        let op = self.fetch();

        self.pc += 2;
        
        // Extracting individual bits of information
        let x: u8 = ((op & 0x0f00) >> 8) as u8;
        let y: u8 = ((op & 0x00f0) >> 4) as u8;
        let n: u8 = ((op & 0x000f)) as u8;
        let nn: u8 = (op & 0x00ff) as u8;
        let addr: u16 = (op & 0x0fff) as u16;
        let id: u8 = ((op & 0xf000) >> 12) as u8;

        // println!("Running op 0x{:4x}", op);
   
        match id {
            0x0 => match nn {
                0xe0 => self.op_00e0(),
                0xee => self.op_00ee(),
                _ => return State::Paused,
            },
            0x1 => self.op_1nnn(addr),
            0x2 => self.op_2nnn(addr),
            0x3 => self.op_3xnn(x, nn),
            0x4 => self.op_4xnn(x, nn),
            0x5 => self.op_5xy0(x, y),
            0x6 => self.op_6xnn(x, nn),
            0x7 => self.op_7xnn(x, nn),
            0xA => self.op_annn(addr),
            0xD => self.op_dxyn(x, y, n),
            0xB => self.op_bnnn(x, nn, addr),
            0xc => self.op_cxnn(x, nn),
            0x9 => self.op_9xy0(x, y),
            0xF => match nn {
                0x15 => self.op_fx15(x),
                0x07 => self.op_fx07(x),
                0x18 => self.op_fx18(x),
                0x1e => self.op_fx1e(x),
                0x29 => self.op_fx29(x),
                0x33 => self.op_fx33(x),
                0x55 => self.op_fx55(x),
                0x65 => self.op_fx65(x),
                0x0a => self.op_fx0a(x),
                _ => return State::Paused,
            },
            0x8 => match n {
                0x0 => self.op_8xy0(x,y),
                0x1 => self.op_8xy1(x,y),
                0x2 => self.op_8xy2(x,y),
                0x3 => self.op_8xy3(x,y),
                0x4 => self.op_8xy4(x,y),
                0x5 => self.op_8xy5(x,y),
                0x6 => self.op_8xy6(x,y),
                0x7 => self.op_8xy7(x,y), 
                0xe => self.op_8xye(x,y),
                _ => {return State::Paused},
            },
            0xe => match nn {
                0x9e => self.op_ex9e(x),
                0xa1 => self.op_exa1(x),
                _ => (),
            },
            _ => {
                return State::Paused;
            },
        }

        State::Running
    }
}



// All the CHIP8 instructions
impl CPU {
    fn op_3xnn(&mut self, x:u8, nn: u8) {

        if self.reg[x as usize] == nn {
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self, x:u8, nn:u8) {
        if self.reg[x as usize] != nn {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x:u8, y:u8) {
        if self.reg[x as usize] == self.reg[y as usize] {
            self.pc += 2;
        }
    }

    fn op_9xy0(&mut self, x: u8, y: u8) {
        if self.reg[x as usize] != self.reg[y as usize] {
            self.pc += 2;
        }
    }

    fn op_bnnn(&mut self, x: u8, nn: u8, addr: u16) {
        match self.mode {
            Mode::Classic => self.pc = addr + (self.reg[0] as u16),
            Mode::Modern => {
                let mut xnn = x as u16;
                xnn <<= 8;
                xnn |= nn as u16;
                let vx = self.reg[x as usize];
                self.pc = (xnn as u8 + vx) as u16;
            },

        };
    }

    fn op_cxnn(&mut self, x: u8, nn: u8) {

        let mut rng = rand::rng();  // Correct way to get RNG
        let random_num = rng.random::<u8>();  // Generate random u8
    
        self.reg[x as usize] = random_num & nn;  // Bitwise AND with nn
    }

    fn op_8xy0(&mut self, x:u8, y:u8) {
        self.reg[x as usize] = self.reg[y as usize];
    }

    fn op_8xy1(&mut self, x: u8, y: u8) {
        self.reg[x as usize] |= self.reg[y as usize];
        self.reg[0xf] = 0;
    }

    fn op_8xy2(&mut self, x:u8, y: u8) {
        self.reg[x as usize] &= self.reg[y as usize]; 
        self.reg[0xf] = 0;
    }

    fn op_8xy3(&mut self, x:u8, y:u8) {
        self.reg[x as usize] ^= self.reg[y as usize];
        self.reg[0xf] = 0;
    }

    fn op_8xy4(&mut self, x:u8, y:u8) {
        let vx = self.reg[x as usize] as u16;
        let vy = self.reg[y as usize] as u16;
        let sum = vx + vy;

        self.reg[x as usize] = self.reg[x as usize].wrapping_add(self.reg[y as usize]);

        if sum > 0xff {
            self.reg[0xf] = 1;
        } else {
            self.reg[0xf] = 0;
        }

    }

    fn op_8xy5(&mut self, x: u8, y:u8) {
        let vx= self.reg[x as usize];
        let vy = self.reg[y as usize];
        let (sub, borrow) = vx.overflowing_sub(vy);

        self.reg[x as usize] = sub;
        self.reg[0xF] = if borrow { 0 } else { 1 }; // Correct borrow flag
    }

    fn op_8xy6(&mut self, x:u8, y: u8) {
        // Set VX to VY

        if self.mode == Mode::Classic {
            self.reg[x as usize] = self.reg[y as usize]; //make configurable
        }

        let lsb = self.reg[x as usize] & 1;
        self.reg[x as usize] >>= 1;
        self.reg[0xF] = lsb;  // VF = original LSB of Vx

    }

    fn op_8xye(&mut self, x:u8, y:u8) {
        // Set VX to VY
        if self.mode == Mode::Classic {
            self.reg[x as usize] = self.reg[y as usize]; // make configurable
        }
        let msb = (self.reg[x as usize] & 0x80) >> 7; // get bit thats shifted out
        self.reg[x as usize] <<= 1; 
        self.reg[0xf] = msb;
    }

    // set VX to VY - VX
    fn op_8xy7(&mut self, x:u8, y:u8) {
        let vx= self.reg[x as usize];
        let vy = self.reg[y as usize];
        let (sub, borrow) = vy.overflowing_sub(vx);

        self.reg[x as usize] = sub;
        self.reg[0xF] = if borrow { 0 } else { 1 }; // Correct borrow flag
    }

    //TODO: revise later
    fn op_fx0a(&mut self, x: u8) {
        let mut key_pressed = false;
    
        for (i, key) in self.keypad.iter().enumerate() {
            if key.pressed {
                self.reg[x as usize] = key.scancode; // Use scancode, not index
                key_pressed = true;
                break;
            }
        }
    
        if !key_pressed {
            self.pc -= 2;
            self.update_dt();
            self.update_st();
        }
    }
    
    fn op_ex9e(&mut self, x: u8) {
        let key_value = self.reg[x as usize];
    
        // Find the key with this scancode
        for key in &self.keypad {
            if key.scancode == key_value && key.pressed {
                self.pc += 2;
                break;
            }
        }
    }

    fn op_exa1(&mut self, x:u8) {
        let key_value = self.reg[x as usize];
        let mut key_pressed = false;
    
        // Find the key with this scancode
        for key in &self.keypad {
            if key.scancode == key_value && key.pressed {
                key_pressed = true;
                break;
            }
        }
    
        if !key_pressed {
            self.pc += 2;
        }
    }

    fn op_fx1e(&mut self, x: u8) {
        // TODO: set VF to 1 when index register overflows outside addressing range
        // println!("fx1e is running");
        let vx = self.reg[x as usize];
        let sum = self.index.wrapping_add(vx as u16);
        self.reg[0xf] = match sum > 0xfff {
            true => 1,
            false => 0,
        };
        self.index = sum; 
    }

    fn op_fx07(&mut self, x: u8) {
        self.reg[x as usize] = self.delay_timer;
    }

    fn op_fx15(&mut self, x: u8) {
        self.delay_timer = self.reg[x as usize];
    }

    fn op_fx18(&mut self, x: u8) {
        self.sound_timer = self.reg[x as usize];
    }

    fn op_fx29(&mut self, x:u8) {
        let char = (self.reg[x as usize]) & 0x0f; // getting the font character
        self.index = 0x050 + (char as u16 * 5); 
    }

    fn op_fx33(&mut self, x:u8) {
        let byte = self.reg[x as usize];
        self.ram[self.index as usize] = byte / 100;
        self.ram[self.index as usize + 1] = (byte / 10) % 10;
        self.ram[self.index as usize + 2] = byte % 10;
    }

    fn op_fx55(&mut self, x: u8) {
        // store V0-VX in ram[index + X]
        // Modern behavior
        for i in 0..=x {
            // println!("i = {}", i);
            self.ram[(self.index + i as u16) as usize] = self.reg[i as usize];
        }

        if self.mode == Mode::Classic {
            self.index = self.index + x as u16 + 1;
        }
    }

    fn op_fx65(&mut self, x: u8) {
        //TODO: Implement old behavior (increment index)

        // Modern behavior
        for i in 0..=x as usize {
            self.reg[i] = self.ram[self.index as usize + i];
        }

        if self.mode == Mode::Classic {
            self.index = self.index + x as u16 + 1;
        }
    }

    fn op_00e0(&mut self) {
        self.buf.fill(0); // zero the display
    }

    fn op_1nnn(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn op_2nnn(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc; // push pc onto stack
        self.sp += 1; // increment stack pointer
        self.pc = addr; // update pc
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_6xnn(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] = nn;
    }

    fn op_7xnn(&mut self, x: u8, nn: u8) {
        let vx = self.reg[x as usize];
        let result = vx.wrapping_add(nn);
        self.reg[x as usize] = result;
    }

    fn op_annn(&mut self, addr: u16) {
        self.index = addr;
    }

    fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        let x_pos = self.reg[x as usize] as u32 % VIDEO_WIDTH ;
        let y_pos = self.reg[y as usize] as u32 % VIDEO_HEIGHT;

        // set VF to 0
        self.reg[0xf] = 0;

        for row in 0..n as u32 {
            if y_pos + row >= VIDEO_HEIGHT {
                break;
            }
            let sprite_byte: u8 = self.ram[(self.index as usize) + (row as usize)];

            for col in 0..8 {
                if x_pos + col > VIDEO_WIDTH {
                    break;
                }

                let sprite_pixel = sprite_byte & (0x80 >> col); // get each individual pixel
                let pixel_idx = ((y_pos + row) * VIDEO_WIDTH + (x_pos + col)) as usize % 2048;

                if sprite_pixel != 0 {
                    if self.buf[pixel_idx] == 1 {
                        self.reg[0xf] = 1;
                    }

                    self.buf[pixel_idx]^= 1;
                }
            }
        }
        self.draw = true;
    }

}