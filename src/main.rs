mod cpu;

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

const KEY_MAPPING: [KeyCode; 16] = [
    KeyCode::Digit1, // 1
    KeyCode::Digit2, // 2
    KeyCode::Digit3, // 3
    KeyCode::Digit4, // c
    KeyCode::KeyQ,   // 4
    KeyCode::KeyW,   // 5
    KeyCode::KeyE,   // 6
    KeyCode::KeyR,   // D
    KeyCode::KeyA,   // 7
    KeyCode::KeyS,   // 8
    KeyCode::KeyD,   // 9
    KeyCode::KeyF,   // E
    KeyCode::KeyZ,   // A
    KeyCode::KeyX,   // 0
    KeyCode::KeyC,   // B
    KeyCode::KeyV,   // F
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
        cpu.load_rom("6-keypad.ch8");
        // cpu.map_keypad(&CHIP8_SCANCODES);

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

            println!("KEY_MAPPINGS: {:#?}", KEY_MAPPING);
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


                // match event.physical_key {
                //     PhysicalKey::Code(KeyCode::Digit1) => {
                //         if event.state
                //         self.cpu.keypad[0].pressed = true;
                //     }

                // }

                if let PhysicalKey::Code(code) = event.physical_key {
                    for (idx, keycode) in KEY_MAPPING.iter().enumerate() {
                        if *keycode == code {
                            match event.state {
                                ElementState::Pressed => {
                                    self.cpu.keypad[idx].pressed = true;
                                    println!("Key {:?} pressed -> CHIP-8 Key {:x}", code, self.cpu.keypad[idx].scancode);
                                    // println!("{:x} is registered as pressed; truth: {}", self.cpu.keypad[idx].scancode, self.cpu.keypad[idx].pressed);                            
                                },
                                ElementState::Released => {
                                    self.cpu.keypad[idx].pressed = false; 
                                    // println!("Key {:?} released -> CHIP-8 Key {:x}", event.physical_key, self.cpu.keypad[idx].scancode);
                                    // println!("{:x} is registered as not pressed; truth: {}", self.cpu.keypad[idx].scancode, !self.cpu.keypad[idx].pressed);
                                },
                            }
                        }
                    }
                }
                
                
            },
            _ => (),
        }
    }
}

fn main() {

    // Handle window logic
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut emu = Emulator::default();
    event_loop.run_app(&mut emu);


}