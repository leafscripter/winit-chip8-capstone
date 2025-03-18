mod cpu;

use std::fs;
use std::io::{self, Write};
use std::process::Command;
use cpu::CPU;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, ElementState};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::keyboard::{PhysicalKey, KeyCode};

use pixels::{SurfaceTexture, Pixels, wgpu::PresentMode};


const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 30;
const CPU_CYCLES_PER_FRAME: usize = 8;

const CHIP8_SCANCODES: [u8; 16] = [
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

struct Emulator<'a> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
    cpu: CPU,
    interval: Duration,
    next_frame_time: Instant,
    key_interval: Duration,
    next_key_rest_time: Instant,
}

impl<'a> Emulator<'a> {
    fn new(window: Arc<Window>, pixels: Pixels<'a>,cpu: CPU, next_frame_time: Instant, next_key_rest_time: Instant, key_interval: Duration) -> Self {
        Emulator {
            window: Some(window),
            pixels: Some(pixels),
            interval: Duration::from_secs_f64(0.017),
            next_frame_time: next_frame_time,
            key_interval: key_interval,
            next_key_rest_time: next_key_rest_time,

            cpu,
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        let mut cpu = CPU::new();
        cpu.load_font(&FONT_SPRITES);
        cpu.map_keypad();

        cpu
    }
}

impl Default for Emulator<'_> {
    fn default() -> Self {
        Self {
            window: None,
            pixels: None,
            interval: Duration::from_millis(17),
            next_frame_time: Instant::now(),
            cpu: CPU::default(),
            next_key_rest_time: Instant::now(),
            key_interval: Duration::from_secs(4),
        }
    }
}

impl ApplicationHandler for Emulator<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
            .with_title("CHIP8-emulator")
            .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH as f64 * SCALE as f64, HEIGHT as f64 * SCALE as f64))
            .with_resizable(false);

            let window = event_loop.create_window(window_attributes).unwrap();
            let arc_window = Arc::new(window);

            self.window = Some(arc_window.clone());

            let surface_texture = SurfaceTexture::new(
                WIDTH as u32 * SCALE as u32,
                HEIGHT as u32 * SCALE as u32,
                arc_window
            );

            let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap();

            self.pixels = Some(pixels);
            self.next_frame_time = Instant::now() + self.interval; // for timing logic

            self.window.as_ref().unwrap().request_redraw();
        }

    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event:WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                
                // Limit CPU instructions per cycle
                for _ in 0..CPU_CYCLES_PER_FRAME {
                    self.cpu.step();
                }

                // rendering logic
                if self.cpu.draw {
                    if let Some(pixels) = &mut self.pixels {
                        let frame = pixels.frame_mut(); // getting a slice of the frame
                        let chunks = frame.chunks_exact_mut(4); // getting 4-bit pixels from frame
                        let buf = self.cpu.buf;

                        for (buf_index, pixel) in chunks.enumerate() {

                            let color = if buf_index < buf.len() && buf[buf_index] != 0 {
                                0xff // make it black
                            } else {
                                0x00 // make it white
                            };

                            // completely fill the pixel with either white or black
                            pixel.fill(color);
                        }

                        pixels.render().expect("Failed to render");
                    }
                    self.cpu.draw = false;
                }

                // Refresh every 60Hz
                sleep(self.next_frame_time - Instant::now());
                self.next_frame_time += self.interval;

                self.cpu.update_dt();
                self.cpu.update_st();

                self.window.as_ref().unwrap().request_redraw();

            },
            WindowEvent::KeyboardInput { 
                device_id, 
                event, 
                is_synthetic 
            } => {

                //TODO
                // check for keys pressed
                // when a key is down, update the corresponding index in cpu.keys (set to true)
                // when a key is released, update the corresponding index in cpu.keys (set to false)


                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Digit1) => self.cpu.keypad[0].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::Digit2) => self.cpu.keypad[1].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::Digit3) => self.cpu.keypad[2].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::Digit4) => self.cpu.keypad[3].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyQ) => self.cpu.keypad[4].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyW) => self.cpu.keypad[5].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyE) => self.cpu.keypad[6].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyR) => self.cpu.keypad[7].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.cpu.keypad[8].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.cpu.keypad[9].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.cpu.keypad[10].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyF) => self.cpu.keypad[11].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyZ) => self.cpu.keypad[12].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyX) => self.cpu.keypad[13].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyC) => self.cpu.keypad[14].pressed = event.state == ElementState::Pressed,
                    PhysicalKey::Code(KeyCode::KeyV) => self.cpu.keypad[15].pressed = event.state == ElementState::Pressed,
                    _ => {},
                }                
            },
            _ => (),
        }
    }
}

fn main() {

    let rom_dir = "ROMS/";

    // List available ROMs
    match fs::read_dir(rom_dir) {
        Ok(entries) => {
            println!("Available games:");
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(filename) = entry.file_name().to_str() {
                        println!("{}", filename);
                    }
                }
            }
        }
        Err(_) => {
            println!("Failed to read ROM directory. Make sure 'ROMS/' exists.");
            return;
        }
    }

    print!("Enter filename: ");
    io::stdout().flush().unwrap();
    let mut filename = String::new();
    io::stdin().read_line(&mut filename).unwrap();
    let filename = format!("{}{}", rom_dir, filename.trim()); // Prefix "ROMS/"

    print!("Enter mode (1 for Classic, 2 for Modern): ");
    io::stdout().flush().unwrap();
    let mut mode_input = String::new();
    io::stdin().read_line(&mut mode_input).unwrap();
    let mode_input = mode_input.trim();

    // Handle window logic
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut emu = Emulator::default();
    
    emu.cpu.load_rom(&filename);

    match mode_input {
        "1" => emu.cpu.set_mode(cpu::Mode::Classic),
        "2" => emu.cpu.set_mode(cpu::Mode::Modern),
        _ => (),
    }

    event_loop.run_app(&mut emu);


}