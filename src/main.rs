mod cpu;

use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};
use cpu::CPU;
use cpu::StepResult;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

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

fn main() {
    // Create a window
    let mut window = Window::new(
        "CHIP-8 Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: Scale::X16,
            none: false,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: false,
            transparency: false,
        },
    ).unwrap();

    window.set_cursor_visibility(false);

    let mut cpu = CPU::new();
    cpu.load_font(&FONT_SPRITES);
    cpu.load_rom("5-quirks.ch8");

    let keys: [Key; 16] = [
        Key::Key1,
        Key::Key2,
        Key::Key3,
        Key::Key4,
        Key::Q,
        Key::W,
        Key::E, 
        Key::R, 
        Key::A,
        Key::S,
        Key::D,
        Key::F, 
        Key::Z,
        Key::X,
        Key::C,
        Key::V,
    ];

    // Main loop
    while window.is_open() {
        // Update the display with our buffer

        for _ in 0..8 {
            let step_result = cpu.step();

            // abort the code if an instruction is wrong
            if let StepResult::Fail = step_result {
                break;
            }

            // updating keys
            for (i, key) in keys.iter().enumerate() {
                if window.is_key_down(*key) {
                    cpu.keys[i] = 1;
                } else {
                    cpu.keys[i] = 0;
                }
            }
        }

        // update every 1/60 seconds which is 60Hz (basic physics)
        std::thread::sleep(std::time::Duration::from_millis(17));

        // when the draw flag is toggled
        // retrieve the current pixel buffer
        // and render it onto the screen
        let draw = cpu.get_draw();
        if draw {
            let window_buffer = cpu.get_pixel_buf();
            window.update_with_buffer(&window_buffer, WIDTH, HEIGHT).unwrap();
            cpu.draw = !draw;
        }

        cpu.update_dt();
        cpu.update_st();
    }
}