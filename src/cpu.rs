use std::fs;

const MAX_REGISTER_LEN: usize = 16;
const MAX_RAM_LEN: usize = 4096;
const MAX_STACK_LEN: usize = 16;
const START_ADDR: usize = 512;
const MAX_SCREEN_LEN: usize = 64 * 32;

const FONT_SPRITES: [u8; 80]  = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0 
    0x20, 0x60, 0x20, 0x20, 0x70, //1 
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3 
    0x90, 0x90, 0xF0, 0x10, 0x10, //4 
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5 
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6 
    0xF0, 0x10, 0x20, 0x10, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B 
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C 
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

pub enum StepResult {
    Success,
    Fail,
    Skip,
}

pub struct CPU {
    pc: usize,
    index: u16,
    display: [u8; MAX_SCREEN_LEN],
    ram: [u8; MAX_RAM_LEN],
    reg: [u8; MAX_REGISTER_LEN],
    stack: [u16; MAX_STACK_LEN],
    draw: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            index: 0,
            display: [0; MAX_SCREEN_LEN],
            ram: [0; MAX_RAM_LEN],
            reg: [0; MAX_REGISTER_LEN],
            stack: [0; MAX_STACK_LEN],
            draw: false,
        }
    }

    pub fn load_font(&mut self, font: &[u8]) -> &mut CPU {
        for i in 0..font.len() {
            self.ram[i] = font[i];
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
        let low: u16 = (self.ram[self.pc+1]) as u16;

        self.pc += 2;

        (high << 8) | low
    }

    pub fn step(&mut self) -> StepResult {
        let op = self.fetch();

        // Extracting individual bits of information
        let x: u8 = ((op & 0x0f00) >> 8) as u8;
        let y: u8 = ((op & 0x00f0) >> 4) as u8;
        let n: u8 = ((op & 0x000f)) as u8;
        let nn: u8 = (op & 0x00ff) as u8;
        let addr: u16 = (op & 0x0fff) as u16;
        let id: u8 = ((op & 0xf000) >> 12) as u8;

        match id {
            0x0 => match op {
                0x00e0 => println!("Implement function later!"),
                _ => println!("Incorrect opcode!")
            },
            0x1 => self.op_jump(addr),
            0x6 => self.op_set_reg_vx(x, nn),
            0x7 => self.op_reg_vx_add(x, nn),
            0xA => self.op_set_index(addr),
            0xD => self.op_draw(),
            _ => {
                println!("Instruction not found!");
                return StepResult::Fail;
            },
        }

        StepResult::Success

    }
}

// All the CHIP8 instructions
impl CPU {
    fn op_jump(&mut self, addr: u16) {
        self.pc = addr as usize;
    }

    fn op_set_reg_vx(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] = nn;
    }

    fn op_reg_vx_add(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] += nn;
    }

    fn op_set_index(&mut self, addr: u16) {
        self.index = addr;
    }

    fn op_draw(&mut self) {
        self.draw = true;
    }

}