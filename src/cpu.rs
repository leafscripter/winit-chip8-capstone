use std::fs;
use pixels::wgpu::naga::proc::index;
use rand::Rng;

const MAX_REGISTER_LEN: usize = 16;
const MAX_RAM_LEN: usize = 4096;
const START_ADDR: usize = 512;
const VIDEO_HEIGHT: u32 = 32;
const VIDEO_WIDTH: u32 = 64;
const MAX_SCREEN_LEN: usize = 64 * 32;
const MAX_KEYPAD_LEN: usize = 16;

const KEY_HEXCODES: [u8; 16] = [
    0x1, 
    0x2, 
    0x3, 
    0xC,
    0x4, 
    0x5, 
    0x6, 
    0xD,
    0x7, 
    0x8, 
    0x9, 
    0xE,
    0xA, 
    0x0, 
    0xB, 
    0xF,
];

pub enum StepResult {
    Success,
    Fail,
    Skip,
}

pub struct CPU {
    pc: usize,
    index: u16,
    buf: [u32; MAX_SCREEN_LEN],
    ram: [u8; MAX_RAM_LEN],
    reg: [u8; MAX_REGISTER_LEN],
    stack: Vec<u16>,
    pub draw: bool,
    delay_timer: u8,
    sound_timer: u8,
    pub keys: [u8; MAX_KEYPAD_LEN], // setting a keypad with 15 elements
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            index: 0,
            buf: [0; MAX_SCREEN_LEN],
            ram: [0; MAX_RAM_LEN],
            reg: [0; MAX_REGISTER_LEN],
            stack: Vec::with_capacity(10),
            keys: [0; MAX_KEYPAD_LEN],
            draw: false,
            delay_timer: 0,
            sound_timer: 0,
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
            self.ram[START_ADDR + i] = buf[i];
        }

        self
    }

}

impl CPU {
    // Fetching two bytes from memory and merging them into one instruction
    fn fetch(&mut self) -> u16 {
        let high: u16 = (self.ram[self.pc]) as u16;
        let low: u16 = (self.ram[self.pc + 1]) as u16;

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

    pub fn step(&mut self) -> StepResult {
        let op = self.fetch();

        self.pc += 2;

        // Extracting individual bits of information
        let x: u8 = ((op & 0x0f00) >> 8) as u8;
        let y: u8 = ((op & 0x00f0) >> 4) as u8;
        let n: u8 = ((op & 0x000f)) as u8;
        let nn: u8 = (op & 0x00ff) as u8;
        let addr: u16 = (op & 0x0fff) as u16;
        let id: u8 = ((op & 0xf000) >> 12) as u8;

        match id {
            0x0 => match op {
                0x00e0 => self.op_00e0(),
                0x00ee => self.op_00ee(),
                _ => return StepResult::Fail,
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
            0xB => self.op_bnnn(addr),
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
                _ => return StepResult::Fail,
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
                _ => {return StepResult::Fail},
            },
            0xe => match op {
                0xe09e => self.op_ex9e(x),
                0xe0a1 => self.op_exa1(x),
                _ => (),
            },
            _ => {
                return StepResult::Fail;
            },
        }

        StepResult::Success
    }

    pub fn get_pixel_buf(&self) -> Vec<u32> {
        let mut buf = Vec::with_capacity((VIDEO_WIDTH * VIDEO_HEIGHT) as usize);
        let color = 0xFF40E0D0;
        let bg = 0xFFFF7F50;
    
        for y in 0..VIDEO_HEIGHT {
            for x in 0..VIDEO_WIDTH {
                let pixel = self.buf[((y * VIDEO_WIDTH) + x) as usize];
                // Convert pixel value to ARGB format (0xFFFFFcF for white, 0x000000 for black)
                let color = if pixel != 0 { color } else { bg };
                buf.push(color);
            }
        }

        buf
    }

    pub fn get_draw(&self) -> bool {
        return self.draw;
    }
}

// impl CPU {
//     fn evaluate_state(&mut self, ) {
//         match self.state {
//             State::Running => (), // proceed, do nothing
//             State::Debug => {
//                 match op & 0xf000 {

//                 }
//             }, 
//             _ => panic!("Invalid state")
//         }
//     }
// }

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

    fn op_bnnn(&mut self, addr: u16) {
        self.pc = (addr + (self.reg[0] as u16)) as usize;
    }

    fn op_cxnn(&mut self, x: u8, nn: u8) {
        let mut rng = rand::rng();
        let rand = rng.random_range(0..=nn);

        self.reg[x as usize] = rand & nn;
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
        let vx = self.reg[x as usize];
        let vy = self.reg[y as usize];

        match vx.checked_add(vy) {
            Some(sum) => {
                self.reg[x as usize] = sum;
                self.reg[0xf] = 0;
            },
            None => {
                self.reg[x as usize] = vx.wrapping_add(vy);
                self.reg[0xf] = 1;
            }
        }

    }

    fn op_8xy5(&mut self, x: u8, y:u8) {
        let vx = self.reg[x as usize];
        let vy = self.reg[y as usize];

        match vx.checked_sub(vy) {
            Some(sub) => {
                self.reg[x as usize] = sub;
                self.reg[0xf] = 1;
            },
            None => {
                self.reg[x as usize] = vx.wrapping_sub(vy);
                self.reg[0xf] = 0;
                // println!("Overflow occured!");
            }
        }
    }

    fn op_8xy6(&mut self, x:u8, y: u8) {
        // make configurable
        self.reg[x as usize] = self.reg[y as usize];

        let ret = self.reg[x as usize] & 1;
        self.reg[x as usize] >>= 1;

        if ret == 1 { 
            self.reg[0xf] = 1;
        } else {
            self.reg[0xf] = 0;
        }

    }

    fn op_8xye(&mut self, x:u8, y:u8) {
        // Set VX to VY
        self.reg[x as usize] = self.reg[y as usize];
        let msb = (self.reg[x as usize] & 0x80) >> 7; // get bit thats shifted out
        self.reg[x as usize] <<= 1; 
        self.reg[0xf] = msb;
    }

    fn op_8xy7(&mut self, x:u8, y:u8) {
        let vx = self.reg[x as usize];
        let vy = self.reg[y as usize];

        match vy.checked_sub(vx) {
            Some(sub) => {
                self.reg[x as usize] = sub;
                self.reg[0xf] = 1;
            },
            None => {
                self.reg[x as usize] = vy.wrapping_sub(vx);
                self.reg[0xf] = 0;
                // println!("Overflow occured!");
            }
        }
    }

    fn op_fx0a(&mut self, x: u8) {
        loop {
            // println!("entering infinite input loop!");
            // keep it at 60Hz in this loop
            std::thread::sleep(std::time::Duration::from_millis(17));
            //update our timers :3
            self.update_dt();
            self.update_st();

            if let Some(pressed_key) = self.keys.iter().position(|&k| k == 1) {
                // println!("key pressed!");
                self.reg[x as usize] = KEY_HEXCODES[pressed_key];
                break;
            } else {
                // println!("stopping instruction from proceeding");
                self.pc += 2;
                self.pc -= 2; // don't let instruction proceed
            }
        }
    }
    
    fn op_ex9e(&mut self, x: u8) {
        for i in 0..KEY_HEXCODES.len() {
            // check that key is pressed and keycode matches vx
            if self.keys[i] == 1 && KEY_HEXCODES[i] == self.reg[x as usize] {
                self.pc += 2; // skip if VX corresponds to a keycode
            }
        }
    }

    fn op_exa1(&mut self, x:u8) {
        for i in 0..KEY_HEXCODES.len() {
            // check that key is released and keycode matches vx
            if self.keys[i] == 0 && KEY_HEXCODES[i] == self.reg[x as usize] {
                self.pc += 2; // skip if 
            }
        }
    }

    fn op_fx1e(&mut self, x: u8) {
        // TODO: set VF to 1 when index register overflows outside addressing range
        // println!("fx1e is running");
        let index: u16= self.index;
        let vx = self.reg[x as usize] as u16;
        let sum = index + vx;
        let threshold= 1000;

        self.index = sum;

        // if the index register index goes beyond the addressing range
        // set VF to 1
        if sum > threshold {
            self.reg[0xf] = 1;
        }
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
        let char = (self.reg[x as usize]) & 0x000f; // getting the font character
        let start = 0x050; // this is where fonts begin
        let end = 0x200;  // this is where fonts end

        for i in start..end {
            if self.ram[i] == char {
                // let high: u16 = (self.ram[i]) as u16;
                // let low: u16 = (self.ram[i + 1]) as u16;
                // let addr = ((high << 8) | low) & 0x0fff;
                self.index = i as u16;
                break;
            } 
        }
    }

    fn op_fx33(&mut self, x:u8) {
        let byte = self.reg[x as usize] as f32;
        let first_digit = (byte / 100.0).floor() as u8 ; // 255 / 100 = 2.55.floor() = 2.00
        let second_digit = (((byte % 100.0) / 10.0).floor()) as u8; // 255 % 100 = 55 / 10 = 5.5.floor() = 5
        let third_digit = ((byte % 100.0) % 10.0) as u8; // 255 % 100 = 55 % 10
        
        self.ram[self.index as usize] = first_digit;
        self.ram[self.index as usize + 1] = second_digit;
        self.ram[self.index as usize + 2] = third_digit;
    }

    fn op_fx55(&mut self, x: u8) {
        // store V0-VX in ram[index + X]
        // Modern behavior
        // TODO: Implement old behavior (increment index)
        // println!("x = {}", x);
        for i in 0..=x as usize {
            // println!("i = {}", i);
            self.ram[self.index as usize + i] = self.reg[i];
        }
        self.index = self.index + x as u16 + 1;
    }

    fn op_fx65(&mut self, x: u8) {
        //TODO: Implement old behavior (increment index)

        // Modern behavior
        for i in 0..=x as usize {
            self.reg[i] = self.ram[self.index as usize + i];
        }

        self.index = self.index + x as u16 + 1;
    }

    fn op_00e0(&mut self) {
        self.buf.fill(0); // zero the display
    }

    fn op_1nnn(&mut self, addr: u16) {
        self.pc = addr as usize;
    }

    fn op_2nnn(&mut self, addr: u16) {
        self.stack.push(self.pc as u16);
        self.pc = addr as usize;
    }

    fn op_00ee(&mut self) {
        self.pc = (self.stack.pop()).unwrap() as usize;
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
        let x_coord = (self.reg[x as usize] as u32) % VIDEO_WIDTH ;
        let y_coord = (self.reg[y as usize] as u32) % VIDEO_HEIGHT;

        // set VF to 0
        self.reg[0xf] = 0;

        for row in 0..n as u32 {
            if y_coord + row >= VIDEO_HEIGHT {
                break;
            }
            let sprite_byte: u8 = self.ram[(self.index as usize) + (row as usize)];

            for col in 0..8 {
                if x_coord + col >= VIDEO_WIDTH {
                    break;
                }

                let sprite_pixel = sprite_byte & (0x80 >> col); // get each individual pixel
                let pixel_idx = ((y_coord + row) * VIDEO_WIDTH + (x_coord + col)) as usize;

                if sprite_pixel != 0 {
                    if self.buf[pixel_idx] == 0xFFFFFFFF {
                        self.reg[0xf] = 1;
                    }

                    self.buf[pixel_idx]^= 0xFFFFFFFF;
                }

            }

        }

        self.draw = true;
    }

}